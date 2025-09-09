# Backpassing

Function based
- [Gleam's `use` expressions](https://gleam.run/news/v0.25-introducing-use-expressions/)
- [Koka's `with` statement](https://koka-lang.github.io/koka/doc/book.html#sec-with)
- [Roc's "backpassing"](https://www.roc-lang.org/tutorial#backpassing) (which unfortunately appears to be a removed feature)

Monad based
- [Haskell's `do` notation](https://en.wikibooks.org/wiki/Haskell/do_notation)
- [Iris's `!` notation](https://idris2.readthedocs.io/en/latest/tutorial/interfaces.html#notation)

## Uses With Containers

```sonance
func maybe_sum(x: Option[U32], y: Option[U32]) -> Option[U32] {
    x.flat_map do(i) {
        y.flat_map do(j) {
            Some(i + j)
        }
    }
}
```

```sonance
func maybe_sum(x: Option[U32], y: Option[U32]) -> Option[U32] {
    let i = x.flat_map?;
    let j = y.flat_map?;

    Some(i + j)
}
```

## Backpassing `match` Special Form

- [Rust's `let else` construct](https://doc.rust-lang.org/rust-by-example/flow_control/let_else.html)

```sonance
match(foo) {
  True?,
  False -> print("nope"),
};

print("yep");
```

Expression returns a tuple.

### With values

```sonance
match(foo) {
  Pass(value)?,
  Fail(error) -> print("oh no: \(error)"),
};

print("yippee: \(value)");
```
