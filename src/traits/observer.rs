use crate::traits::subject::Subject;
pub trait Observer<'a, T,A> where T: Subject<A> {
    fn watch(&'a mut self, subject:&'a mut T);
    fn update(&mut self) -> Vec<Option<A>>;
}