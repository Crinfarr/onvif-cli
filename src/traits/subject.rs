pub trait Subject<T> {
    fn observe(&self)->Option<T>;
    fn consume(&mut self)->Option<T>;
}