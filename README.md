# The Vine Programming Language

Vine is a programming language with focus on simplicity and type safety. (it's written in Rust btw)

> [!WARNING]  
> This project is in a very early stage of development.

## How To Build This Project

Because this is a Rust project, go ahead and [install Rust](https://www.rust-lang.org/learn/get-started#installing-rust).

Now you are ready to _cd_ into any crate in `crates/` and try to run the crate via `cargo run`. This project needs nightly, but it should automatically install.

## Progress Bar / TODO

- [ ] Implement frontend
  - [X] Iterate through bytes of source file
  - [X] Lex (implement lexer)
  - [X] Parse
  - [ ] Resolve
  - [ ] Type-check
  - [ ] Control-Flow-Graph
  - [ ] IR (?)
- [ ] Implement backend
  - [ ] _Vine Virtual Machine_ backend
  - [ ] JavaScript backend
- [ ] Implement CLI
- [ ] Documentation

## Maintained Crates

The purpose of some crates in this project has vanished due to some changes in other crates. These crates are not used and therefore currently not maintained:

- `set`
- `queue`