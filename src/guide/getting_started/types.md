# Types

## Custom Types

```
type MyType {
    VariantA,
    VariantB,
}
```

## Records

```
type MyRecord(foo: U32, bar: Boolean);
```

## Anonymous Records

```
fn sincos(x: F64) -> type(F64, F64) {
    type(x.sin, x.cos)
}
```

## Type Modules

```
module type MyType {
    ...
}
```


## Type Aliases

```
type Alias = MyType;
```
