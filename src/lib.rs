#![doc = include_str!("libdoc.md")]
use std::{
    convert::{AsMut, AsRef},
    ops::{Deref, DerefMut},
};

/// Write function chains more concisely with the chained macro.
///
/// The general syntax looks like this `chained!(<optional mod symbols> <initial value> <delimeter> <function/closure>)`.
///
/// The macro supports both `,` commas and `=>` fat arrows as delimiters or separators, but they can't be mixed together.
/// ```
/// # use chained::*;
/// chained!(0, |x| x+1);
/// // and
/// chained!(0
///          => |x| x+1
/// );
/// // are both valid and produce the same code
/// ```
///
/// # Usage
/// If you're starting with an initial value and want to evaluate later (lazy)
///
/// *Remember: No modifier symbols in the beginning*
/// ```
/// # use chained::*;
/// let lazy = chained!(10, |x| x+1, |x| x*x);
/// assert_eq!(121, lazy.eval());
/// ```
/// If you're starting with an initial value but want to evaluate now
///
/// *Remember: use >> in the beginning*
/// ```
/// # use chained::*;
/// let result = chained!(>> 10, |x| x+1, |x| x*x);
/// assert_eq!(121, result);
/// ```
/// If you already have a chain and want to chain more functions and evaluate later
///
/// *Remember: use => in the beginning*
/// ```
/// # use chained::*;
/// let lazy = chained!(69, |x| x + 1);
/// let still_lazy = chained!(=> lazy, |x| x - 1);
/// assert_eq!(69, still_lazy.eval());
/// ```
/// If you want to add functions to an existing chain and evaluate now
///
/// *Remember: use >>> in the beginning*
/// ```
/// # use chained::*;
/// let lazy = chained!(69, |x| x + 1);
/// let result = chained!(>>> lazy, |x| x - 1);
/// assert_eq!(69, result);
/// ```
///
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
    (=> $val: expr, $($fn: expr),+) => {
            $val
            $(.chain($fn))+
    };
    (=> $val: expr => $($fn: expr)=>+) => {
            $val
            $(.chain($fn))+
    };
    (>> $val: expr, $($fn: expr),*) => {
        Link::new($val)
            $(.chain($fn))*
            .eval()
    };
    (>> $val: expr => $($fn: expr)=>*) => {
        Link::new($val)
            $(.chain($fn))*
            .eval()
    };
    (>>> $val: expr, $($fn: expr),+) => {
            $val
            $(.chain($fn))*
            .eval()
    };
    (>>> $val: expr => $($fn: expr)=>+) => {
            $val
            $(.chain($fn))*
            .eval()
    };
}

/// The trait that is the heart and soul of this crate.
pub trait Chained {
    type Item;
    fn eval(self) -> Self::Item;

    fn chain<F, T>(self, fun: F) -> Chain<Self, F, T>
    where
        Self: Sized,
        F: FnOnce(Self::Item) -> T,
    {
        Chain { val: self, fun }
    }
}

/// The trait that let's you turn a type `T` into `Link<T>`, which implements the [Chained] trait that let's you chain functions by calling the [chain][Chained::chain] method.
///
/// It's important to remember that if you want to own the value, use [into_chained][IntoChained::into_chained] or [to_chained][IntoChained::to_chained] (clones self). The other other methods let you work with borrowed values.
pub trait IntoChained {
    fn into_chained(self) -> Link<Self>
    where
        Self: Sized,
    {
        Link::new(self)
    }

    fn to_chained(&self) -> Link<Self>
    where
        Self: Clone,
    {
        Link::new(self.clone())
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

    fn chained_as_ref<T: ?Sized>(&self) -> Link<&T>
    where
        Self: AsRef<T>,
    {
        Link::new(self.as_ref())
    }

    fn chained_as_mut<T: ?Sized>(&mut self) -> Link<&T>
    where
        Self: AsMut<T>,
    {
        Link::new(self.as_mut())
    }
}

impl<T> IntoChained for T {}

/// The base type which implements the [Chained] trait. It holds the initial value and is always the starting point of a chain.
///
/// You can manually use it too if you'd like to avoid using the [chained] macro or calling methods like [into_chained][IntoChained::into_chained].
/// ```
/// # use chained::*;
/// assert_eq!(20, Link::new(10).chain(|a| a + a).eval());
/// ```
/// However, using the [chained] macro is still the recommended way to chain functions when you are starting with an initial value.
/// ```
/// # use chained::*;
/// // Produces the same code as the above example
/// assert_eq!(20, chained!(>> 10, |a| a + a));
/// ```
///
/// Link merely takes ownership of T, and doesn't perform any operations when [Link::eval] is called.
/// To take the value T out of Link, simply call [Link::eval]
/// ```
/// # use chained::*;
/// let x = Link::new("Hello");
/// let y = x.eval();
/// assert_eq!("Hello", y);
/// ```
#[derive(Clone, Debug)]
pub struct Link<T>(T);

impl<T> Link<T> {
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
    fn eval(self) -> Self::Item {
        self.0
    }
}

/// The type that is returned when the [Chained::chain] method is called.
///
/// Chain implements the [Chained] trait and stores the previous chain or value, and a function.
/// This struct is analogous to the Map struct which is returned by the iterator when map is called.
#[derive(Clone)]
pub struct Chain<C: Chained, F, T>
where
    F: FnOnce(C::Item) -> T,
{
    val: C,
    fun: F,
}

impl<C, F, T, B> Chained for Chain<C, F, T>
where
    C: Chained<Item = B>,
    F: FnOnce(C::Item) -> T,
{
    type Item = T;
    fn eval(self) -> Self::Item {
        (self.fun)(self.val.eval())
    }
}
