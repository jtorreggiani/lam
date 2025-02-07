# Logic Abstract Machine

The Logic Abstract Machine (LAM) is a stack-based, abstract machine designed to evaluate logical expressions. It is inspired by the Warren Abstract Machine (WAM) used in Prolog implementations, but it is not intended specifically for Prolog.

🚧 LAM is under active development and should be considered a work in progress.

## Introduction

The aim for LAM is to provide a robust core for implementing logic programming languages that is optimized for higher order logics, probabilistic logic, other advanced logic programming paradigms. The LAM is designed to be an efficient and flexible abstract machine that can be easily extended and adapted to different programming languages and paradigms.

## Design

LAM has been built in Rust based on the principles presented in the book [Warren's Abstract Machine: A Tutorial Reconstruction](https://direct.mit.edu/books/monograph/4253/Warren-s-Abstract-MachineA-Tutorial-Reconstruction) by Hassan Ait-Kaci. It currently has basic support for unification, backtracking, and arithmetic evaluation.

## File Structure

```
.
├── Cargo.lock
├── Cargo.toml
├── README.md
├── benches
│   └── machine_bench.rs
├── bin
│   └── collect_source
├── docs
│   └── source.rs
├── examples
│   ├── instructions
│   └── prolog
├── src
│   ├── languages
│   │   ├── lam.rs
│   │   └── prolog.rs
│   ├── lib.rs
│   ├── machine
│   │   ├── arithmetic.rs
│   │   ├── choice_point.rs
│   │   ├── core.rs
│   │   ├── error_handling.rs
│   │   ├── execution.rs
│   │   ├── frame.rs
│   │   ├── instruction.rs
│   │   ├── lambda.rs
│   │   ├── mod.rs
│   │   ├── term.rs
│   │   └── unification.rs
│   └── main.rs
└── tests
    ├── test_languages
    ├── test_machine
    └── test_main.rs
```

## Usage

To build LAM you need Rust and Cargo installed. You can build the project by running:

```bash
cd lam
cargo build
```

To run the tests:

```bash
cargo test
```

## Working with the LAM instruction set

You can run the LAM instructions by providing a file with the instructions. For example, to run the hello world example in `examples/lam/hello_world.lam`:

```
PutStr 0 "Hello world"
Call write
```

```bash
cargo run --bin lam examples/lam/hello_world.lam
```

## Running Prolog interpreter

Since the LAM was derived from the WAM, prolog is the easiest language to implement. To run the test prolog interpreter:

```bash
cargo run --bin prolog
```