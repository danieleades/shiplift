//! Types for working with docker TTY streams

use crate::{Compat, Result};
use bytes::BytesMut;
use futures_util::{
    io::{AsyncRead, AsyncWrite},
    stream::{Stream, StreamExt, TryStreamExt},
};
use pin_project::pin_project;
use std::io;
use tokio_util::codec::length_delimited::LengthDelimitedCodec;

/// An enum representing a chunk of TTY text streamed from a Docker container.
///
/// For convenience, this type can deref to the contained `Vec<u8>`.
#[derive(Debug, Clone)]
pub enum TtyChunk {
    StdIn(BytesMut),
    StdOut(BytesMut),
    StdErr(BytesMut),
}

impl std::ops::Deref for TtyChunk {
    type Target = BytesMut;
    fn deref(&self) -> &Self::Target {
        match self {
            TtyChunk::StdIn(bytes) | TtyChunk::StdOut(bytes) | TtyChunk::StdErr(bytes) => bytes,
        }
    }
}

impl std::ops::DerefMut for TtyChunk {
    fn deref_mut(&mut self) -> &mut BytesMut {
        match self {
            TtyChunk::StdIn(bytes) | TtyChunk::StdOut(bytes) | TtyChunk::StdErr(bytes) => bytes,
        }
    }
}

fn decode(reader: impl AsyncRead) -> impl Stream<Item = Result<TtyChunk>> {
    let reader = Compat::new(reader);

    LengthDelimitedCodec::builder()
        .length_field_offset(4)
        .length_field_length(4)
        .num_skip(0)
        .new_read(reader)
        .map(|chunk| {
            let bytes = chunk?;
            let tty_chunk = match bytes[0] {
                0 => TtyChunk::StdIn(bytes),
                1 => TtyChunk::StdOut(bytes),
                2 => TtyChunk::StdErr(bytes),
                n => panic!("invalid stream number from docker daemon: '{}'", n),
            };

            Ok(tty_chunk)
        })
}

pub(crate) fn decode_chunks<S>(hyper_chunk_stream: S) -> impl Stream<Item = Result<TtyChunk>>
where
    S: Stream<Item = Result<Vec<u8>>>,
{
    let reader = Box::pin(hyper_chunk_stream.map_err(|e| io::Error::new(io::ErrorKind::Other, e)))
        .into_async_read();

    decode(reader)
}

type TtyReader<'a> = Pin<Box<dyn Stream<Item = Result<TtyChunk>> + 'a>>;
type TtyWriter<'a> = Pin<Box<dyn AsyncWrite + 'a>>;

/// TTY multiplexer returned by the `attach` method.
///
/// This object can emit a stream of `TtyChunk`s and also implements `AsyncWrite` for streaming bytes to Stdin.
#[pin_project]
pub struct Multiplexer<'a> {
    #[pin]
    reader: TtyReader<'a>,
    #[pin]
    writer: TtyWriter<'a>,
}

use std::{
    pin::Pin,
    task::{Context, Poll},
};

impl<'a> Stream for Multiplexer<'a> {
    type Item = Result<TtyChunk>;
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.project().reader.poll_next(cx)
    }
}

impl<'a> AsyncWrite for Multiplexer<'a> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().writer.poll_write(cx, buf)
    }
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().writer.poll_flush(cx)
    }
    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().writer.poll_close(cx)
    }
}

impl<'a> Multiplexer<'a> {
    /// Split the `Multiplexer` into the component `Stream` and `AsyncWrite` parts
    pub fn split(
        self
    ) -> (
        impl Stream<Item = Result<TtyChunk>> + 'a,
        impl AsyncWrite + 'a,
    ) {
        (self.reader, self.writer)
    }
}
