# Documentation

## Primitive Types

- `num`
- `char`
- `str`
- `nil`
- `bool`
- `object`

## Composite Types

- `String`

## Markup

Markup elements start `<`. The token before `<` must be one of the following:

- `=`
- `==`
- `=>`
- `<`
- `<=`
- `<<`
- `<<=`
- `>`
- `>=`
- `>>`
- `>>=`
- `+`
- `+=`
- `-`
- `-=`
- `*`
- `*=`
- `**`
- `**=`
- `/`
- `/=`
- Line comment `//`
- Doc comment `///`
- `%`
- `%=`
- `|`
- `|=`
- `||`
- `||=`
- `&`
- `&=`
- `&&`
- `&&=`
- `^`
- `^=`
- `[`
- `{`
- `,`
- `;`
- `:`
- `!=`

## Parsing Steps

1. Load source file into memory and validate it as UTF-8
2. Iterate through the source file while yielding tokens
3. Build an AST while consuming the tokens
4. Validate the AST
5. Replace all identifiers and paths with references to items.
6. Repeat until all names are resolved
7. Type check
8. Control-Flow-Graph
9. Transpile

## Characteristics Of Functions

Functions can have attributes, altering their behavior at runtime or compile-time. Parenthesised attributes should be inferred by the compiler and applied platform dependently.

- Pure: The function (a) always returns the same value for a set of parameters and (b) has no side effects.
- (Constant): The function only uses constant expressions and invocations of constant functions.
- (Inline): The function should be inlined.

## Types

These are the primitive types in Vine:

- `num`
- `str`
- 

Those types are passed by value.

### Compound Types

There is just the object type. The empty object `()` has no members and therefore zero size.