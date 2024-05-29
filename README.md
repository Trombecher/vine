# The Vine Programming Language

Vine is a programming language with focus on simplicity and type safety.

## Build Errors

If you are encountering any build errors, it is probably due to missing error codes. Try generating them via:

```shell
cd src/error/generate
bun install
bun index.ts
```

Error codes are generated via TypeScript because it is easier to generate documentation.