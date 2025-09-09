# Modules

## Basic Modules

```sonance
module my_module {
    ...
}
```

Basic modules with no body can be used as a file heading. A file may have zero or one. This allows no identifier, instead using the file name. This is so you may attach attributes or documentation to a module declared by a file.

```sonance
module;
```

## Type Modules

Types themselves can act as modules, and you may declare and access items inside of them. To declare items as apart of a type, use the `type` keyword along with the type name instead of a module identifier. This may only be done within the file a type is defined, and cannot be used on type aliases.

```sonance
module type MyType {
    const num = 123;
}

print(MyType.num)
```

## Trait Modules

Modules can implement a trait for a given type using `->`. This must provide implementations of any items the trait requires, and will automatically provide them whenever the type is used in a context that requires that trait. Like inherent type modules, they contain the `type` keyword instead of a module identifier and must be declared in the same file as the type or the trait.

```sonance
module type MyType -> MyTrait {
    ...
}

func use_trait[T: MyTrait](input: T) {
    ...
}


let value: MyType = ...;
use_trait(value);
```

## Named Trait Modules

What if you needed to declare a trait module outside of where the type or trait is defined? Or if you want to provide an alternative to an existing trait module? In that case, you may provide a module name when creating a trait module. In this case, those items are only used when specifically requested. This also prevents the "HashMap problem" familiar to Rust's justification of it's orphan rules system.

```sonance
module foo MyType -> MyTrait {
    ...
}

func use_trait[T: MyTrait](input: T) {
    ...
}

let value: MyType = ...;
use_trait(foo(value)); // Apply the named trait module before passing it
```
