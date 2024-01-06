# `!`-notation, brought to Rust

This crate provides `bang!` macro, similar to [`!`-notation in Idris2](https://idris2.readthedocs.io/en/latest/tutorial/interfaces.html#notation).
With it, the following expression:

``` rust
bang!(!x + !y + z)
```

Gets desugared into

``` rust
x.and_then(|x| {
    y.and_then(|y| {
        x + y + z
    })
})
```

## What problem does this solve?

Rust provides neat containers to make error-handling easy and convenient.
However, sometimes one, like myself, can find themselves in a peculiar situation.

Suppose there is some function that produces an `Option`:

``` rust
fn some_func(x1: T1, x2: T2, x3: T3) -> Option<u32>
```

And you even have the arguments prepared to call the function.
However, there's a catch: the arguments were also produced in some way that made them wrapped in `Option`.
So, we only have `x1: Option<T1>`, `x2: Option<T2>` and `x3: Option<T3>` in our hands.
This, of course, prompts us to write something along the lines of:

``` rust
x1.and_then(|x1| {
    x2.and_then(|x2| {
        x3.and_then(|x3| {
            some_func(x1, x2, x3)
        })
    })
});
```

This looks horrible and requires us to write a lot of pointless symbols!
Luckily, programming language [Idris2](https://github.com/idris-lang/Idris2) (which I greatly adore) provides a nice solution to this problem.
It has a nice piece of syntactic sugar called [!-notation](https://idris2.readthedocs.io/en/latest/tutorial/interfaces.html#notation).
In Idris, the expression prefixed with `!` is lifted as high as possible and bound to a fresh name.
Then the original expression is replaced with this newly bound name.
Following the Idris documentation, it transforms the following expression

``` idris
f !(g !(print y) !x)
```

into

``` idris
do y' <- print y
   x' <- x
   g' <- g y' x'
   f g'
```

This crate provides a procedural macro that performs a similar transformation in Rust. Simply import it:
``` rust
use bang_notation::bang;
```

And then we could replace the unreadable mess we had originally written with a concise:

``` rust
bang!(some_func(!x1, !x2, !x3))
```

It would get desugared to the chain of `and_then`s we initially came up with.

## What can you use this with?

In Idris, `!`-notation can be used with arbitrary monads.
This Rust version aims to be as general as possible.
It is supposed to work with any type that provides `and_then` method with a suitable signature.
If you found any example where this doesn't work, let me know!

## Is this any better than `?` operator?

Yes! AFAIK, currently `?` operator works only with `Option` and `Result` types.
There is an [experimental `Try` trait](https://doc.rust-lang.org/std/ops/trait.Try.html) that could be used to overload `?` operator,
but it provides semantics different to `!`-notation in this crate.
Unlike `?`, `!`-notation is applicable to arbitrary types that provide a suitable interface for `and_then`.
As an example, you might want to take a look at `list001` test that uses a custom-written `List` monad.

## In what order are values bound?

The expressions marked with `!` are bound left-to-right, in order they are written in the source code.
In case of nested expressions marked with `!`, the inner expressions are bound first.
After that, they are replaced with new names in the outer expression, and the outer expression is bound.
