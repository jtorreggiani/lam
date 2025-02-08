# Logic Abstract Machine (LAM)
Version 0.2

The Logic Abstract Machine (LAM) is a stack-based abstract machine for logic programming. Inspired by the Warren Abstract Machine (WAM) for Prolog, LAM has evolved into a robust and extensible core that supports several logic programming paradigms including unification with backtracking, arithmetic evaluation, lambda calculus, and a simple Prolog interpreter.

## Overview

LAM provides:

### Unification and Backtracking
Uses a union–find data structure with trailing and path compression for efficient unification. The machine supports backtracking via choice points and a comprehensive trail mechanism.

### Arithmetic Evaluation
An arithmetic parser and evaluator support standard operators (+, -, *, /) with correct precedence and parentheses.

### Lambda Calculus Support
Lambda abstraction, beta reduction, and capture–avoiding substitution are implemented to support higher–order reasoning.

### Prolog Interpreter
A basic Prolog interpreter supports facts, rules, queries, and even dynamic clause management (assert/retract) with indexing for faster lookup.

### Dynamic Clause Management
Clauses can be asserted and retracted at runtime. Indexing is available to speed up clause lookup.

### Extensive Testing and Coverage
A comprehensive test suite covers arithmetic, unification, backtracking, lambda calculus, and more. Test coverage is measured using tools such as cargo-tarpaulin.

### Detailed Formal Specification
The SPECIFICATION.md file documents the abstract machine's state, instruction semantics, invariants, and design assumptions.

## Project Structure

```
.
├── Cargo.toml
├── Cargo.lock
├── README.md
├── SPECIFICATION.md
├── src
│   ├── main.rs
│   ├── lib.rs
│   ├── machine
│   │   ├── arithmetic.rs
│   │   ├── choice_point.rs
│   │   ├── core.rs
│   │   ├── error_handling.rs
│   │   ├── execution.rs
│   │   ├── frame.rs
│   │   ├── instruction.rs
│   │   ├── lambda.rs
│   │   ├── mod.rs
│   │   ├── term.rs
│   │   └── unification.rs
│   └── languages
│       ├── lam.rs
│       └── prolog
│           ├── ast.rs
│           ├── interpreter.rs
│           └── parser.rs
└── tests
    ├── test_machine
    │   ├── test_arithmetic.rs
    │   ├── test_backtracking_constants.rs
    │   ├── test_backtracking_variables.rs
    │   ├── test_benchmark.rs
    │   ├── test_build_compound.rs
    │   ├── test_cut.rs
    │   ├── test_dynamic_clause_management.rs
    │   ├── test_environment.rs
    │   ├── test_error_conditions.rs
    │   ├── test_get_structure.rs
    │   ├── test_higher_order.rs
    │   ├── test_indexed_call.rs
    │   ├── test_lambda.rs
    │   ├── test_machine.rs
    │   ├── test_path_inference.rs
    │   ├── test_ping.rs
    │   ├── test_tail_call.rs
    │   ├── test_term.rs
    │   ├── test_unification.rs
    │   ├── test_unification_performance.rs
    │   ├── test_dynamic_clause_indexing.rs
    │   └── test_properties.rs
    └── test_languages
```

## Building and Running

### Building the Project

Ensure that you have Rust and Cargo installed. Then run:

```bash
cargo build
```

For an optimized production build, run:

```bash
cargo build --release
```

### Running Tests

Run the full test suite with:

```bash
cargo test
```

To generate a test coverage report using cargo-tarpaulin:

```bash
cargo tarpaulin --out Html
```

Then open the generated HTML report to view detailed coverage information.

## Running the LAM Interpreter

LAM Programs can be run by supplying a file with LAM instructions:

```bash
cargo run --bin lam <program.lam>
```

## Current Capabilities

### Unification & Backtracking
Efficient unification with a union–find structure supporting path compression and backtracking via choice points and a trail.

### Arithmetic Evaluation
Parsing and evaluation of arithmetic expressions with proper operator precedence.

### Lambda Calculus
Support for lambda abstractions, beta reduction, and capture–avoiding substitution.

### Prolog Parsing & Execution
A recursive–descent parser for Prolog with support for facts, rules, and queries. Dynamic clause management (assert/retract) is included.

### Dynamic Clause Management & Indexing
Runtime assertion and retraction of clauses with clause indexing for efficient lookup.

### Robust Testing
A comprehensive test suite (with unit and integration tests) verifies correctness, and test coverage is measured to ensure reliability.

## Formal Specification

For a complete formal specification of the LAM abstract machine (including state, instruction semantics, invariants, and design decisions), please refer to the SPECIFICATION.md file.

## Future Improvements

The roadmap for LAM includes several areas for further development:

### Reduce Cloning Overhead
Refactor parts of the code (especially in union–find and state management) to use borrowing or smart pointers (e.g. Rc, RefCell) and potentially persistent data structures to reduce cloning and allocation overhead.

### Structured Logging & Introspection
Enhance logging by integrating a structured logging framework (such as tracing or slog). This will allow machine state (registers, substitution, choice points) to be output in a structured format (e.g. JSON) for easier debugging and interactive analysis.

### Parser Refactoring
Consider using a combinator library (like nom) to refactor the recursive–descent parser for both LAM and Prolog. This may improve error reporting and code reuse.

### Interactive Debugger
Develop an interactive debugger (or REPL) that allows step–by–step execution of LAM programs, with detailed inspection of registers, control and environment stacks, union–find trail, etc.

### Performance Optimization
Benchmark and profile the machine using more sophisticated tools (e.g. Criterion) to guide optimizations in unification, backtracking, and instruction dispatch.

### Language Extensions
Expand support for additional logic programming features (such as probabilistic logic, constraint logic programming, or higher–order logic) and further enrich the Prolog interpreter.

## Contributing

Contributions are very welcome! If you wish to contribute:

* Please open an issue or submit a pull request
* Ensure your changes include appropriate tests to maintain high test coverage
* Follow the existing code style and document any new features or modifications