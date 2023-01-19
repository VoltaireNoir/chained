//! Chained provides an ergonomic API for **lazily chaining function calls** with the help of [``Chained``] and [``IntoChained``] traits and the [``chained``] macro.
//! The core data types and traits are modeled after Rust's [iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html) and [map](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.map) method.
//! But instead of working with collections, the traits and data types in this crate are designed to work with single values.
//! Just like iterators in Rust, chains are also **lazy** by default. Nothing is evaluated until you explicitly call [``Chained::eval``].
//!
//! > **CAUTION: This crate is currently experimental. Future updates might bring breaking changes.**
//!
//! This crate is inspired by both [pipe-trait](https://crates.io/crates/pipe-trait) and [pipeline](https://crates.io/crates/pipeline) crates.
//! If you do not require lazy evaluation and just want a simple way to chain function calls or method calls, the aforementioned crates might serve you better.
//!
//! *For full macro syntax examples, see [chained]. For working with borrowed values, see all methods of the trait [IntoChained].*
//! # Usage Examples
//! ```
//! use chained::*;
//! use std::fs;
//!
//! fn main() {
//!     let count_chars = |s: String| s.chars().count();
//!
//!     // Chaining function calls with regular method syntax
//!     fs::read_to_string("myfile")
//!         .unwrap()
//!         .into_chained() // Takes ownership of the String
//!         .chain(count_chars)
//!         .chain(|count| println!("File has {count} chars"))
//!         .eval(); // The closures are evaluated after eval() is called
//!
//!     // Writing the same code more concisely using the macro
//!     // Note: `>>` in the beginning tells the macro to call eval() at the end of the chain
//!     chained!(>> fs::read_to_string("myfile").unwrap()
//!              => count_chars
//!              => |count| println!("File has {count} chars")
//!     );
//!     // You can also use commas as separators in the macro
//!     let print = |c| println!("File has {c} chars");
//!     chained!(>> fs::read_to_string("myfile").unwrap(), count_chars, print);
//!
//!     // Making use of lazy evaluation
//!     // Note: Since '>>' is not specified, eval() is not automatically called on this chain
//!     let lazy = chained!(fs::read_to_string("myfile").unwrap(), count_chars);
//!     let still_lazy = squared_sqrt(lazy);
//!     // All chained functions are evaluated only after eval() is called
//!     still_lazy.chain(|x| println!("{x}")).eval();
//! }
//!
//! // Use impl Chained just like you'd use impl Iterator
//! fn squared_sqrt(x: impl Chained<Item = usize>) -> impl Chained<Item = f32> {
//!     let squared = |x: usize| x.pow(2);
//!     let sqrt = |x| (x as f32).sqrt();
//!     x.chain(squared).chain(sqrt)
//! }
//! ```
use std::{
    convert::{AsMut, AsRef},
    ops::{Deref, DerefMut},
};

#[macro_export]
/// Write function chains more concisely with the chained macro.
///
/// The macro supports both `,` commas and `=>` fat arrows as delimiters or separators, but they can't be mixed together.
/// ```
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
/// *Remember: No symbols in the beginning*
/// ```
/// let lazy = chained!(10, |x| x+1, |x| x*x);
/// let result = lazy.eval();
/// ```
/// If you're starting with an initial value but want to evaluate now
///
/// *Remember: use >> in the beginning*
/// ```
/// let result = chained!(>> 10, |x| x+1, |x| x*x);
/// ```
/// If you already have a chain and want to chain more functions and evaluate later
///
/// *Remember: use => in the beginning*
/// ```
/// let still_lazy = chained!(=> lazy, |x| x - 1);
/// ```
/// If you want to add functions to an existing chain and evaluate now
///
/// *Remember: use >>> in the beginning*
/// ```
/// let result = chained!(>>> lazy, |x| x - 1);
/// ```
///
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
/// assert_eq!(20, Link::new(10).chain(|a| a + a).eval());
/// ```
/// However, using the [chained] macro is still the recommended way to chain functions when you are starting with an initial value.
/// ```
/// // Produces the same code as the above example
/// assert_eq!(20, chained!(>> 10, |a| a + a));
/// ```
///
/// Link merely takes ownership of T, and doesn't perform any operations when [Link::eval] is called.
/// To take the value T out of Link, simply call [Link::eval]
/// ```
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
