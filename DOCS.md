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
5. Resolve all the names
6. Repeat until all names are resolved
7. Type check
8. Transpile