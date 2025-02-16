# Logic Abstract Machine (LAM)

[![CI](https://github.com/jtorreggiani/lam/actions/workflows/ci.yml/badge.svg)](https://github.com/jtorreggiani/lam/actions) ![Coverage](./coverage/coverage.svg)
[![License: BDS](https://img.shields.io/badge/License-BSD-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-blue.svg)](https://www.rust-lang.org/)


> **A modern abstract machine for logic programming and automated reasoning in AI systems.**

LAM aims to be a high-performance abstract machine written in Rust that serves as the core for next-generation logic programming languages. Designed from the ground up with AI-intensive applications in mind, LAM supports unification, backtracking, arithmetic evaluation, lambda calculus, and dynamic clause management. Its architecture enables efficient automated reasoning—making it ideal for research, production AI systems, and advanced knowledge representation.

⚠️ **Note:** This project is a work in progress and is not yet production-ready. Contributions, feedback, and ideas are welcome!

---

## Table of Contents

- [Introduction](#introduction)
- [Key Features](#key-features)
- [Architecture Overview](#architecture-overview)
  - [Core Machine](#core-machine)
  - [Unification & Backtracking](#unification--backtracking)
  - [Arithmetic & Lambda Calculus](#arithmetic--lambda-calculus)
  - [Dynamic Clause Management](#dynamic-clause-management)
- [Installation & Build Instructions](#installation--build-instructions)
- [Usage](#usage)
  - [Running LAM Programs](#running-lam-programs)
  - [Interactive Debugging & Logging](#interactive-debugging--logging)
- [Project Structure](#project-structure)
- [Testing & Coverage](#testing--coverage)
- [Roadmap & Future Improvements](#roadmap--future-improvements)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)

---

## Introduction

The **Logic Abstract Machine (LAM)** aims to be comprehensive execution engine for logic programming languages. At its heart, LAM supports:

- **Automated Reasoning:** Unification using a union–find algorithm with trailing for backtracking, ensuring efficient variable binding and rollback.
- **Arithmetic Evaluation:** A robust parser and evaluator for arithmetic expressions with proper operator precedence.
- **Lambda Calculus:** First-class support for lambda abstraction, beta reduction, and capture–avoiding substitution, paving the way for higher–order reasoning.
- **Dynamic Clause Management:** Runtime assertion and retraction of clauses with indexing for fast clause lookup.

LAM right now is a research tool but seeks to become a foundation for developing new logic programming languages that can serve as the backbone of modern AI systems. Its design emphasizes modularity, performance, and extensibility, to be well suited for both academic exploration and industrial-scale reasoning tasks.

---

## Key Features

- **Efficient Unification & Backtracking:**  
  Utilizes an optimized union–find algorithm with path compression and trailing to support rapid unification and safe backtracking.
  
- **Advanced Arithmetic Evaluation:**  
  Parses and evaluates complex arithmetic expressions (supporting `+`, `-`, `*`, `/`, and parentheses) with guaranteed correct precedence.
  
- **Native Lambda Calculus Support:**  
  Implements lambda abstractions, beta reductions, and capture–avoiding substitution to enable higher–order logic and functional reasoning.
  
- **Dynamic Clause Management:**  
  Supports runtime assertion and retraction of clauses with indexing for rapid clause retrieval.
  
- **Modular and Extensible Design:**  
  Written in Rust for memory safety and concurrency, LAM’s modular architecture allows seamless integration with other AI components.
  
- **Comprehensive Testing & Documentation:**  
  Over 100% test coverage with unit and integration tests, plus a formal specification detailing the machine’s semantics and invariants.

---

## Architecture Overview

### Core Machine

At its core, LAM is a register-based abstract machine that maintains:

- **Registers:** A fixed-size vector for storing partial terms (constants, variables, compounds, etc.).
- **Control & Environment Stacks:** For managing predicate calls, return addresses, and local variable bindings.
- **Choice Points:** For backtracking, each capturing a complete snapshot of the machine state (registers, substitutions, union–find trail, etc.).
- **Predicate & Index Tables:** For fast clause lookup and dynamic clause management.

### Unification & Backtracking

LAM’s unification engine is built upon a union–find structure enhanced with:
  
- **Path Compression:** Minimizes lookup times during repeated unifications.
- **Trailing Mechanism:** Records state changes to support efficient rollback during backtracking.
- **Choice Point Management:** Enables robust backtracking when unification fails, ensuring automated reasoning even in complex logic scenarios.

### Arithmetic & Lambda Calculus

- **Arithmetic Module:**  
  Supports parsing and evaluating expressions with standard arithmetic operators, handling operator precedence and parentheses seamlessly.
  
- **Lambda Calculus Support:**  
  Facilitates lambda abstractions, applications, and beta reductions, complete with capture–avoiding substitution. This module empowers LAM to handle higher–order logic programming.

### Dynamic Clause Management

- **Runtime Clause Assertion & Retraction:**  
  Clauses can be dynamically added or removed at runtime. This feature enables flexible knowledge bases that can evolve during execution.
  
- **Clause Indexing:**  
  An indexing mechanism speeds up clause lookup, ensuring that even large dynamic databases are handled efficiently.

---

## Installation & Build Instructions

### Prerequisites

- **Rust & Cargo:** Ensure you have the latest stable version of [Rust](https://www.rust-lang.org/tools/install).

### Building from Source

Clone the repository and build the project:

```bash
git clone https://github.com/jtorreggiani/lam.git
cd lam
cargo build
```

Please fork the repository if you plan to contribute.

For an optimized release build:

```bash
cargo build --release
```

## Usage

To run execute LAM instructions, use the following command:

```bash
cargo run examples/hello_world.lam
```

Please note the LAM instruction parser is still in development and may not be able to parse all instructions.

---

## Project Structure

```plaintext
.
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
├── SPECIFICATION.md
├── benches
├── bin
├── docs
├── examples
│   └── hello_world.lam
├── package.json
├── src
│   ├── lib.rs
│   ├── machine
│   │   ├── arithmetic.rs
│   │   ├── choice_point.rs
│   │   ├── core.rs
│   │   ├── error_handling.rs
│   │   ├── execution.rs
│   │   ├── frame.rs
│   │   ├── instruction.rs
│   │   ├── instruction_parser.rs
│   │   ├── lambda.rs
│   │   ├── mod.rs
│   │   ├── term.rs
│   │   └── unification.rs
│   └── main.rs
└── tests
```

---

## Testing & Coverage

LAM is rigorously tested with a comprehensive suite that covers:

- **Arithmetic Evaluation**
- **Unification & Backtracking**
- **Lambda Calculus Operations**
- **Dynamic Clause Management**
- **Error Handling and Edge Cases**

Run all tests with:

```bash
cargo test
```

For a test coverage report (using [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)):

```bash
cargo tarpaulin --out Html
```

Then open the generated HTML report in your browser.

---

## Roadmap & Future Improvements

### Short-Term Goals

- **Implement a basic prolog interpreter:**
  Develop a Prolog interpreter using LAM as the core engine.
- **Benchmark & Profiling:**
  Profile the LAM prolog implementation and compare to existing prolog interpreters to identify bottlenecks.
- **Release a stable version:**
  Release a stable version of LAM with a basic prolog interpreter.

### Long-Term Vision

- **Language Extensions:**  
  Extend LAM to support probabilistic logic, constraint logic programming, and higher–order reasoning.
- **Integration with AI Systems:**  
  Build libraries and interfaces to seamlessly integrate LAM with modern AI frameworks.
- **Performance Optimizations:**  
  Profile and optimize the execution engine using tools like [Criterion](https://github.com/bheisler/criterion.rs) to achieve state–of–the–art performance.

---

## Contributing

Contributions are highly welcome! Please follow these guidelines:

1. **Fork the Repository:** Create your own branch for features or bug fixes.
2. **Write Tests:** Ensure that new features and bug fixes include comprehensive tests.
3. **Documentation:** Update the README and inline documentation as necessary.
4. **Pull Requests:** Open a pull request for review. All contributions must adhere to the existing coding style and include tests.

For major changes, please open an issue first to discuss your ideas.

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

LAM draws inspiration from seminal works such as the Warren Abstract Machine (WAM) and leverages modern programming paradigms to serve the evolving needs of AI and logic programming. Special thanks to the Rust community for their excellent tools and libraries that made this project possible.
