# Language Reference

## Keywords

| Context   | Keyword  | Description                                    |
| --------- | -------- | ---------------------------------------------- |
| Modules   | `module` | Declare modules or implementations             |
| Modules   | `import` | Import items from other modules                |
| Modules   | `export` | Make item available to rest of current package |
| Modules   | `public` | Make item available to external packages       |
| Types     | `type`   | Alias types                                    |
| Types     | `trait`  | Declare traits                                 |
| Functions | `func`   | Declare functions                              |
| Functions | `do`     | Declare arguments in a block literal           |
| Functions | `block`  | Block type literal                             |
| Patterns  | `let`    | Declare variables                              |
| Patterns  | `mut`    | Make place mutable                             |
| Patterns  | `set`    | Assign value to place                          |
| Patterns  | `match`  | Pattern match an value                         |

## Punctuation

| Name | Symbol | Description                                |
| ---- | ------ | ------------------------------------------ |
|      | `=`    | Assignment, labeled parameters             |
|      | `.`    | Pipeline                                   |
|      | `;`    | Statement ending                           |
|      | `,`    | General purpose separator                  |
|      | `:`    | Type ascription                            |
|      | `()`   | Parameters, arguments, grouped expressions |
|      | `{}`   | Code blocks                                |
|      | `<>`   | Type generic                               |
|      | `!`    | Label declaration, label call              |
|      | `?`    | Implicit argument, backpassing             |
|      | `->`   | Return type ascription, match branches     |
|      | `&`    | Shared references                          |
|      | `$`    | Unique References                          |
