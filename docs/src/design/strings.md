# Strings

## Escape Codes

See Rust lol.

## String Interpolation

Interpolation can be done with a special escape sequence `\(...)` where the `...` is an arbitrary expression.

```sonance
print("\(a) plus \(b) is \(a + b)");
```

## Multiline Strings

Both after the opening quote and before closing quote must be only whitespace, which is stripped.
Starting whitespace of the closing quote is commonly stripped from all middle lines, syntax error if a line has less starting whitespace.
Newlines can be escaped using `\` at the end of the line.

```sonance
let example = "
    foo\
    bar
    ";

assert_equal(example, "foobar");
```

## Raw Strings

Raw strings are created by prefixing the opening quote and suffixing the closing quote with a set number of `#` characters.
Within raw strings, normal special characters are interpreted literally, including sequences of lesser amounts of `#`.
To use escape sequences, prefix the `\` character with the same amount of `#` as the opening and closing quotes.

```sonance
let example ###"
    "you can put whatever here"
    ##" not affected
    ###\n escape codes
    ###\(value) interpolation works
"###;
```
