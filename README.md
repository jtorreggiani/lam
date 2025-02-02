# Logic Abstract Machine

The Logical Abstract Machine (LAM) is a stack-based, abstract machine that can design to evaluate logical expressions. It is inspired by the Warren Abstract Machine (WAM) used in Prolog implementations, but it is not intended specifically for Prolog.

🚧 LAM is under active development and should be considered a work in progress.

## Introduction

The aim for LAM is to provide a robust core for implementing logic programming languages that is optimized for higher order logics, probabilistic logic, other advanced logic programming paradigms. The LAM is designed to be an efficient and flexible abstract machine that can be easily extended and adapted to different programming languages and paradigms.

## Design

LAM has been in Rust based on the principles presented in the the book [Warren's Abstract Machine: A Tutorial Reconstruction](https://direct.mit.edu/books/monograph/4253/Warren-s-Abstract-MachineA-Tutorial-Reconstruction) by Hassan Ait-Kaci. It currently has basic support for unification, backtracking, and arithmetic evaluation.

## File Structure

```
.
├── Cargo.lock
├── Cargo.toml
├── README.md
├── docs
│   └── GRAMMAR.md
├── src
│   ├── arithmetic.rs
│   ├── lambda.rs
│   ├── lib.rs
│   ├── machine
│   │   ├── choice_point.rs
│   │   ├── frame.rs
│   │   ├── instruction.rs
│   │   ├── machine.rs
│   │   ├── mod.rs
│   │   └── trail.rs
│   ├── main.rs
│   └── term.rs
└── tests
    ├── test_arithmetic.rs
    ├── test_backtracking_constants.rs
    ├── test_backtracking_variables.rs
    ├── test_benchmark.rs
    ├── test_build_compound.rs
    ├── test_cut.rs
    ├── test_dynamic_clause_management.rs
    ├── test_environment.rs
    ├── test_get_structure.rs
    ├── test_higher_order.rs
    ├── test_indexed_call.rs
    ├── test_machine.rs
    ├── test_path_inference.rs
    ├── test_tail_call.rs
    ├── test_term.rs
    └── test_unification.rs
```

## Usage

To build LAM you need Rust and Cargo installed. You can build the project by running:

```bash
cargo build
```

To run the tests:

```bash
cargo test
```
