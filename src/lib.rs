#![feature(test)]

use std::ops::{Deref, DerefMut};

#[macro_export]
macro_rules! pipeline {
    ($val: expr, $($fn: expr),*) => {
        Line($val)
            $(.pipe($fn))*
    };
    ($val: expr => $($fn: expr)=>*) => {
        Line($val)
            $(.pipe($fn))*
    };
    (>> $val: expr, $($fn: expr),*) => {
        Line($val)
            $(.pipe($fn))*
            .consume()
    };
    (>> $val: expr => $($fn: expr)=>*) => {
        Line($val)
            $(.pipe($fn))*
            .consume()
    };
}

trait Pipeline
where
    Self: Sized,
{
    type Item;
    fn consume(self) -> Self::Item;

    fn pipe<F, T>(self, fun: F) -> Pipe<Self, F, T>
    where
        F: FnOnce(Self::Item) -> T,
    {
        Pipe { val: self, fun }
    }
}

trait IntoPipeline
where
    Self: Sized,
{
    fn into_line(self) -> Line<Self> {
        Line(self)
    }

    fn line(&self) -> Line<&Self> {
        Line(self)
    }

    fn line_mut(&mut self) -> Line<&mut Self> {
        Line(self)
    }

    fn line_deref<T>(&self) -> Line<&Self::Target>
    where
        Self: Deref<Target = T>,
        <Self as Deref>::Target: Sized,
    {
        Line(self.deref())
    }

    fn line_deref_mut<T>(&mut self) -> Line<&mut Self::Target>
    where
        Self: DerefMut<Target = T>,
        <Self as Deref>::Target: Sized,
    {
        Line(self.deref_mut())
    }
}

impl<T: Sized> IntoPipeline for T {}

#[derive(Clone, Debug)]
struct Line<T: Sized>(T);

impl<T> Pipeline for Line<T> {
    type Item = T;
    fn consume(self) -> Self::Item {
        self.0
    }
}

#[derive(Clone)]
struct Pipe<C: Pipeline, F, T>
where
    T: Sized,
    F: FnOnce(C::Item) -> T,
{
    val: C,
    fun: F,
}

impl<C, F, T, B> Pipeline for Pipe<C, F, T>
where
    Self: Sized,
    C: Pipeline<Item = B>,
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
    extern crate test;
    use test::*;

    #[test]
    fn order() {
        let add = |a| a + a;
        let sub = |a| a - a;
        let mul = |a| a * a;
        assert_eq!(0, pipeline!(>> 1, add, sub, mul))
    }

    #[bench]
    fn math_reg(b: &mut Bencher) {
        let add = |a| a + a;
        let sub = |a| a - a;
        let mul = |a| a * a;

        b.iter(|| mul(mul(mul(sub(add(10))))))
    }

    #[bench]
    fn math_pipeline(b: &mut Bencher) {
        let add = |a| a + a;
        let sub = |a| a - a;
        let mul = |a| a * a;
        b.iter(|| {
            pipeline!(>> 10, add, sub, mul, mul, mul);
        })
    }
}
