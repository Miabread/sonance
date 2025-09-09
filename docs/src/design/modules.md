# Modules

```sonance
module my_module {
    ...
}
```

Basic modules with no body can be used as a file heading. A file may have zero or one. This allows no identifier, instead using the file name. This is so you may attach attributes or documentation to a module declared by a file.

```sonance
module;
```

## Type modules

Modules can be attached to a type by following the module identifer with the type identifier.

```sonance
module my_module MyType {
    ...
}
```

Modulles can be "inherent" to a type by using the `type` keyword instead of a module identifier.

```sonance
module type MyType {
    ...
}
```

## Trait modules

Modules can implement a trait for a given type using `->`.

```sonance
module type MyType -> MyTrait {
    ...
}
```

This can be done inherent or as a named module.

```sonance
module my_implmentation MyType -> MyTrait {
    ...
}
```
