# A library for **lazily chaining functions**
The data types and traits are modeled after Rust's [iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html) trait and its [map](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.map) method.
But instead of working with collections, the traits and data types in this crate are designed to work with single values.
Just like iterators in Rust, chains are also **lazy** by default. Nothing is evaluated until you explicitly call [``Chained::eval``], which is analogous to calling `.next()` on an iterator.

The [``Chained``] trait serves as the backbone of this library, while the [``InterChained``] trait has a blanket implementation for all types and let's you start function chains anywhere, which is especially useful in already existing method chains.
The [``chained``] macro, on the other hand, let's you write elegant and concise function chains while internally making use of the same traits and types which this library provides.

> **CAUTION! Chained is currently experimental. Future updates might bring breaking changes.**

This crate is inspired by both [pipe-trait](https://crates.io/crates/pipe-trait) and [pipeline](https://crates.io/crates/pipeline) crates.
If you do not require lazy evaluation and just want a simple way to chain function calls or method calls, the aforementioned crates might serve you better.

*For full macro syntax examples, see [chained]. For working with borrowed values, see all methods of the trait [InterChained].*
# Usage examples
```
use chained::*;

use std::{env,fmt::Display};

fn main() {
    let count_chars = |s: String| s.chars().count();
    
    fn print_count(count: impl Display) {
        println!("Args have a total of {count} chars");
    }

    // Chaining function calls with regular method syntax
    env::args()
        .collect::<String>()
        .into_chained(count_chars) // Owns value, chains the given Fn/Closure and returns a Chain type
        .chain(print_count) // Now you can call chain to add more Fns/Closures 
        .eval(); // Everything is evaluated only after eval() is called

    // Writing the same code more concisely using the macro
    // Note: `>>` in the beginning tells the macro to call eval() at the end of the chain
    chained!(>> env::args().collect::<String>()
             => count_chars
             => print_count
    );
    // You can also use commas as separators in the macro
    // This produces the same code as above
    chained!(>> env::args().collect::<String>(), count_chars, print_count);

    // Making use of lazy evaluation
    // Note: Since '>>' is not specified in the macro, eval() is not automatically called on this chain
    let lazy = chained!(env::args().collect::<String>(), count_chars);
    let still_lazy = squared_sqrt(lazy); // Take a look at the fn defined below
    // All chained functions are evaluated only after eval() is called
    still_lazy.chain(|x| println!("{x}")).eval();
}

// Use impl Chained just like you'd use impl Iterator
fn squared_sqrt(x: impl Chained<Item = usize>) -> impl Chained<Item = f32> {
    let squared = |x: usize| x.pow(2);
    let sqrt = |x| (x as f32).sqrt();
    x.chain(squared).chain(sqrt)
}
```

# A note on object safety
While the [``Chained``] trait appears to be [object safe](https://doc.rust-lang.org/reference/items/traits.html#object-safety) on the surface, as you can turn an existing type that implements the [``Chained``] trait into a trait object, but any chain that is turned into a trait object will be rendered useless as you cannot call either [chain][Chained::chain] or [eval][Chained::eval] on them.

Rust's Iterator map method, however, works on trait objects even if it requires `Self` to be `Sized`. This is made possible by re-implementing the Iterator trait on `Box<I>` and `&mut I` where `I: Iterator + ?Sized`.
This works because the Iterator's most important method `next()` is object safe, as it takes `&mut self` and returns `Option<Self::Item>`.
[``Chained``], on the other hand, takes ownership of 'self' in both its methods which stops us from using such workarounds.

Making Chained object safe would require significant API changes, and I'm not sure if it's worth it. But I'm very much open to suggestions if the users of this library (if there will be any) deem that trait safety is important. Feel free to open an issue if you have a suggestion or create a PR if you'd like to help solve this directly through [collaboration](https://github.com/VoltaireNoir/chained/).
