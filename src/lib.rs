use std::ops::{Deref, DerefMut};

#[macro_export]
macro_rules! chained {
    ($val: expr, $($fn: expr),*) => {
        Link::new($val)
            $(.chain($fn))*
    };
    ($val: expr => $($fn: expr)=>*) => {
        Link::new($val)
            $(.chain($fn))*
    };
    (>> $val: expr, $($fn: expr),*) => {
        Link::new($val)
            $(.chain($fn))*
            .consume()
    };
    (>> $val: expr => $($fn: expr)=>*) => {
        Link::new($val)
            $(.chain($fn))*
            .consume()
    };
}

pub trait Chained
where
    Self: Sized,
{
    type Item;
    fn consume(self) -> Self::Item;

    fn chain<F, T>(self, fun: F) -> Chain<Self, F, T>
    where
        F: FnOnce(Self::Item) -> T,
    {
        Chain { val: self, fun }
    }
}

pub trait IntoChained
where
    Self: Sized,
{
    fn into_chained(self) -> Link<Self> {
        Link::new(self)
    }

    fn chained(&self) -> Link<&Self> {
        Link::new(self)
    }

    fn chained_mut(&mut self) -> Link<&mut Self> {
        Link::new(self)
    }

    fn chained_deref<T>(&self) -> Link<&Self::Target>
    where
        Self: Deref<Target = T>,
        <Self as Deref>::Target: Sized,
    {
        Link::new(self.deref())
    }

    fn chained_deref_mut<T>(&mut self) -> Link<&mut Self::Target>
    where
        Self: DerefMut<Target = T>,
        <Self as Deref>::Target: Sized,
    {
        Link::new(self.deref_mut())
    }
}

impl<T: Sized> IntoChained for T {}

#[derive(Clone, Debug)]
pub struct Link<T: Sized>(T);

impl<T: Sized> Link<T> {
    pub fn new(val: T) -> Self {
        Link(val)
    }
}

impl<T> From<T> for Link<T> {
    fn from(value: T) -> Self {
        Link::new(value)
    }
}

impl<T> Chained for Link<T> {
    type Item = T;
    fn consume(self) -> Self::Item {
        self.0
    }
}

#[derive(Clone)]
pub struct Chain<C: Chained, F, T>
where
    T: Sized,
    F: FnOnce(C::Item) -> T,
{
    val: C,
    fun: F,
}

impl<C, F, T, B> Chained for Chain<C, F, T>
where
    Self: Sized,
    C: Chained<Item = B>,
    F: FnOnce(C::Item) -> T,
{
    type Item = T;
    fn consume(self) -> Self::Item {
        (self.fun)(self.val.consume())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order() {
        let add = |a| a + a;
        let sub = |a| a - a;
        let mul = |a| a * a;
        assert_eq!(0, chained!(>> 1, add, sub, mul))
    }
}
