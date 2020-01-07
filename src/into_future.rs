pub trait IntoFuture {
    /// The output that the future will produce on completion.
    type Output;
    /// Which kind of future are we turning this into?
    type Future: std::future::Future<Output = Self::Output>;

    /// Creates a future from a value.
    fn into_future(self) -> Self::Future;
}
