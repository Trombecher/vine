# The Vine Programming Language

Vine is a programming language with focus on simplicity and type safety. (it's written in Rust btw.)

> [!WARNING]  
> This project is in a very early stage of development. No stability is guaranteed. Expect breaking API changes regularly.
> 
> Currently only the crates `lex`, `parse`, `error` and `warning` are actively maintained. Code from the crate `vine` is being migrated.

## How To Build This Project

Because this is a Rust project, go ahead and [install Rust](https://www.rust-lang.org/learn/get-started#installing-rust) and it's dependencies.

Then you need to install [Bun](https://bun.sh/). Why? Because the Rust code for the error codes needs to be generated from TypeScript.

To generate the codes run:

```shell
cd crates/error/generate
bun index.ts
```

There should now be an up-to-date `crates/error/src/generated_codes.rs` file.

Now you are ready to _cd_ into any crate in `crates/` and try to run the crate via `cargo run`. This project needs nightly, but it should automatically install.