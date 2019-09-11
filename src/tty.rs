use crate::{Error, Result};
use byteorder::{BigEndian, ByteOrder, ReadBytesExt};
use bytes::BytesMut;
use futures::task::Context;
use log::trace;
use std::{
    future::Future,
    io::{self, Cursor},
    pin::Pin,
    task::Poll,
};
use tokio_codec::Decoder;
use tokio_io::{AsyncRead, AsyncWrite};

#[derive(Debug)]
pub struct Chunk {
    pub stream_type: StreamType,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum StreamType {
    StdIn,
    StdOut,
    StdErr,
}

/// A multiplexed stream.
pub struct Multiplexed {
    stdin: Box<dyn AsyncWrite>,
    chunks: Box<dyn futures::Stream<Item = Result<Chunk>>>,
}

pub struct MultiplexedBlocking {
    stdin: Box<dyn AsyncWrite>,
    chunks: Box<dyn Iterator<Item = Result<Chunk>>>,
}

/// Represent the current state of the decoding of a TTY frame
enum TtyDecoderState {
    /// We have yet to read a frame header
    WaitingHeader,
    /// We have read a header and extracted the payload size and stream type,
    /// and are now waiting to read the corresponding payload
    WaitingPayload(usize, StreamType),
}

pub struct TtyDecoder {
    state: TtyDecoderState,
}

impl Chunk {
    /// Interprets the raw bytes as a string.
    ///
    /// Returns `None` if the raw bytes do not represent
    /// a valid UTF-8 string.
    pub fn as_string(&self) -> Option<String> {
        String::from_utf8(self.data.clone()).ok()
    }

    /// Unconditionally interprets the raw bytes as a string.
    ///
    /// Inserts placeholder symbols for all non-character bytes.
    pub fn as_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.data).into_owned()
    }
}

impl TtyDecoder {
    pub fn new() -> Self {
        Self {
            state: TtyDecoderState::WaitingHeader,
        }
    }
}

impl Default for TtyDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for TtyDecoder {
    type Item = Result<Chunk>;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<Self::Item>> {
        loop {
            match self.state {
                TtyDecoderState::WaitingHeader => {
                    if src.len() < 8 {
                        trace!("Not enough data to read a header");
                        return Ok(None);
                    } else {
                        trace!("Reading header");
                        let header_bytes = src.split_to(8);
                        let payload_size: Vec<u8> = header_bytes[4..8].to_vec();
                        let stream_type = match header_bytes[0] {
                            0 => {
                                return Err(Error::InvalidResponse(
                                    "Unsupported stream of type stdin".to_string(),
                                ));
                            }
                            1 => StreamType::StdOut,
                            2 => StreamType::StdErr,
                            n => {
                                return Err(Error::InvalidResponse(format!(
                                    "Unsupported stream of type {}",
                                    n
                                )));
                            }
                        };

                        let length =
                            Cursor::new(&payload_size).read_u32::<BigEndian>().unwrap() as usize;
                        trace!(
                            "Read header: length = {}, stream_type = {:?}",
                            length,
                            stream_type
                        );
                        // We've successfully read a header, now we wait for the payload
                        self.state = TtyDecoderState::WaitingPayload(length, stream_type);
                        continue;
                    }
                }
                TtyDecoderState::WaitingPayload(len, stream_type) => {
                    if src.len() < len {
                        trace!(
                            "Not enough data to read payload. Need {} but only {} available",
                            len,
                            src.len()
                        );
                        return Ok(None);
                    } else {
                        trace!("Reading payload");
                        let data = src.split_to(len)[..].to_owned();
                        let tty_chunk = Chunk { stream_type, data };

                        // We've successfully read a full frame, now we go back to waiting for the next
                        // header
                        self.state = TtyDecoderState::WaitingHeader;
                        return Ok(Some(tty_chunk));
                    }
                }
            }
        }
    }
}

impl Multiplexed {
    /// Create a multiplexed stream.
    pub(crate) fn new<T>(stream: T) -> Multiplexed
    where
        T: AsyncRead + AsyncWrite + 'static,
    {
        let (reader, stdin) = stream.split();
        Multiplexed {
            chunks: Box::new(chunks(reader)),
            stdin: Box::new(stdin),
        }
    }

    pub fn wait(self) -> MultiplexedBlocking {
        MultiplexedBlocking {
            stdin: self.stdin,
            chunks: Box::new(self.chunks.wait()),
        }
    }
}

impl futures::Stream for Multiplexed {
    type Item = Result<Chunk>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Self::Item>> {
        self.chunks.poll()
    }
}

impl Iterator for MultiplexedBlocking {
    type Item = Result<Chunk>;

    fn next(&mut self) -> Option<Result<Chunk>> {
        self.chunks.next()
    }
}

macro_rules! delegate_io_write {
    ($ty:ty) => {
        impl io::Write for $ty {
            fn write(
                &mut self,
                buf: &[u8],
            ) -> std::result::Result<usize, io::Error> {
                self.stdin.write(buf)
            }

            fn flush(&mut self) -> std::result::Result<(), io::Error> {
                self.stdin.flush()
            }
        }
    };
}

delegate_io_write!(Multiplexed);
delegate_io_write!(MultiplexedBlocking);

pub fn chunks<S>(stream: S) -> impl futures::stream::Stream<Item = Result<Chunk>>
where
    S: AsyncRead,
{
    let stream = futures::stream::unfold(stream, |stream| {
        let header_future = ::tokio_io::io::read_exact(stream, vec![0; 8]);

        let fut = header_future.and_then(|(stream, header_bytes)| {
            let size_bytes = &header_bytes[4..];
            let data_length = BigEndian::read_u32(size_bytes);
            let stream_type = match header_bytes[0] {
                0 => StreamType::StdIn,
                1 => StreamType::StdOut,
                2 => StreamType::StdErr,
                n => panic!("invalid stream number from docker daemon: '{}'", n),
            };

            tokio::io::read_exact(stream, vec![0; data_length as usize])
                .map(move |(stream, data)| (Chunk { stream_type, data }, stream))
        });
        // FIXME: when updated to futures 0.2, the future itself returns the Option((Chunk,
        // stream)).
        // This is much better because it would allow us to swallow the unexpected eof and
        // stop the stream much cleaner than writing a custom stream filter.
        Some(fut)
    });

    util::stop_on_err(stream, |e| e.kind() != io::ErrorKind::UnexpectedEof)
        .map_err(crate::Error::from)
}

mod util {
    use futures::stream::Stream;
    use std::task::Poll;

    pub struct StopOnError<S, F> {
        stream: S,
        f: F,
    }

    pub fn stop_on_err<S, F>(
        stream: S,
        f: F,
    ) -> StopOnError<S, F>
    where
        S: Stream,
        F: FnMut(&S::Error) -> bool,
    {
        StopOnError { stream, f }
    }

    impl<S, F> Stream for StopOnError<S, F>
    where
        S: Stream,
        F: FnMut(&S::Item) -> bool,
    {
        type Item = S::Item;

        fn poll_next(
            self: Pin<&mut Self>,
            cx: &mut Context,
        ) -> Poll<Option<S::Item>> {
            match self.stream.poll() {
                Err(e) => {
                    if (self.f)(&e) {
                        Err(e)
                    } else {
                        Ok(Async::Ready(None))
                    }
                }
                a => a,
            }
        }
    }
}
