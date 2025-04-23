# Blocks


## Example: `scope`

```sonance
func scope<T>(body: block() -> T) -> T {
    body!()
}
```

## Example: `Option<T>`

```sonance
condition
    .then { print("was true") }
    .else { print("was false") }
```

```sonance
func then<T>(self: Boolean, body: block() -> T) -> Option<T> {
    match(self) {
        True -> Some(body!()),
        False -> None,
    }
}

func else<T>(self: Option<T>, body: block() -> T) -> T {
    match(self) {
        Some(value) -> value,
        None -> body!(),
    }
}
```

## Example: `while`

```sonance
let mut i = 1;

while { i < 6 } {
    print(i);
    i = i + 1;
}
```

```sonance
func while(condition: block() -> Boolean, body: block()) {
    loop {
        condition!().then {
            body!();
        };
    };
}
```
