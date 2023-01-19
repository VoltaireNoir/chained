![](chained.png)
##### A Rust library that provides an ergonomic API for lazily chaining functions.

| :exclamation:  Chained is currently experimental. Future updates might bring breaking changes   |
|----------------------------------------------------------------------------------------------------|

> This crate is inspired by both [pipe-trait](https://crates.io/crates/pipe-trait) and [pipeline](https://crates.io/crates/pipeline) crates.
> If you do not require lazy evaluation and just want a simple way to chain function calls or method calls, the aforementioned crates might serve you better.

# Usage Examples
```rust
use chained::*;
use std::env;

fn main() {
    let count_chars = |s: String| s.chars().count();

    // Chaining function calls with regular method syntax
    env::args()
        .collect::<String>()
        .into_chained() // Takes ownership of the string, returns a Link type
        .chain(count_chars) // Now you can call chain and pass a Fn/Closure as an argument
        .chain(|count| println!("File has {count} chars"))
        .eval(); // The closures are evaluated only after eval() is called

    // Writing the same code more concisely using the macro
    // Note: `>>` in the beginning tells the macro to call eval() at the end of the chain
    chained!(>> env::args().collect::<String>()
             => count_chars
             => |count| println!("Args have a total of {count} chars")
    );
    // You can also use commas as separators in the macro
    let print = |c| println!("Args have a total of {c} chars");
    chained!(>> env::args().collect::<String>(), count_chars, print);

    // Making use of lazy evaluation
    // Note: Since '>>' is not specified, eval() is not automatically called on this chain
    let lazy = chained!(env::args().collect::<String>(), count_chars);
    let still_lazy = squared_sqrt(lazy);
    // All chained functions are evaluated only after eval() is called
    still_lazy.chain(|x| println!("{x}")).eval();
}

fn squared_sqrt(x: impl Chained<Item = usize>) -> impl Chained<Item = f32> {
    let squared = |x: usize| x.pow(2);
    let sqrt = |x| (x as f32).sqrt();
    x.chain(squared).chain(sqrt)
}
```
