pub trait Transaction<'b, In> {
    fn transaction<F, Out>(&'b self, tx: F) -> Out
    where
        F: for<'a> Fn(&'a mut In) -> Out;
}
