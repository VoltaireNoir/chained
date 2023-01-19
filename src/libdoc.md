Chained provides an ergonomic API for **lazily chaining function calls** with the help of [``Chained``] and [``IntoChained``] traits and the [``chained``] macro.
The core data types and traits are modeled after Rust's [iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html) and [map](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.map) method.
But instead of working with collections, the traits and data types in this crate are designed to work with single values.
Just like iterators in Rust, chains are also **lazy** by default. Nothing is evaluated until you explicitly call [``Chained::eval``].

> **CAUTION: This crate is currently experimental. Future updates might bring breaking changes.**

This crate is inspired by both [pipe-trait](https://crates.io/crates/pipe-trait) and [pipeline](https://crates.io/crates/pipeline) crates.
If you do not require lazy evaluation and just want a simple way to chain function calls or method calls, the aforementioned crates might serve you better.

*For full macro syntax examples, see [chained]. For working with borrowed values, see all methods of the trait [IntoChained].*
# Usage Examples
```
use chained::*;
use std::fs;

fn main() -> Result<(),std::io::Error> {
let count_chars = |s: String| s.chars().count();

// Chaining function calls with regular method syntax
fs::read_to_string("myfile")?
.into_chained() // Takes ownership of the String
.chain(count_chars)
.chain(|count| println!("File has {count} chars"))
.eval(); // The closures are evaluated after eval() is called

// Writing the same code more concisely using the macro
// Note: `>>` in the beginning tells the macro to call eval() at the end of the chain
chained!(>> fs::read_to_string("myfile").unwrap()
=> count_chars
=> |count| println!("File has {count} chars")
);
// You can also use commas as separators in the macro
let print = |c| println!("File has {c} chars");
chained!(>> fs::read_to_string("myfile").unwrap(), count_chars, print);

// Making use of lazy evaluation
// Note: Since '>>' is not specified, eval() is not automatically called on this chain
let lazy = chained!(fs::read_to_string("myfile").unwrap(), count_chars);
let still_lazy = squared_sqrt(lazy);
// All chained functions are evaluated only after eval() is called
still_lazy.chain(|x| println!("{x}")).eval();
# Ok(())
}

// Use impl Chained just like you'd use impl Iterator
fn squared_sqrt(x: impl Chained<Item = usize>) -> impl Chained<Item = f32> {
let squared = |x: usize| x.pow(2);
let sqrt = |x| (x as f32).sqrt();
x.chain(squared).chain(sqrt)
}
```
