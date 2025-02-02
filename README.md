# Logic Abstract Machine

The Logical Abstract Machine (LAM) is a stack-based, abstract machine that can design to evaluate logical expressions. It is inspired by the Warren Abstract Machine (WAM) used in Prolog implementations.

🚧 This project is under active development and should be considered a work in progress.

## Background

The WAM was developed by David H. D. Warren in 1983 while he was at the University of Edinburgh. It's considered one of the most influential developments in the implementation of logic programming languages, particularly Prolog.

The key aspects of the WAM include:

Register Architecture: The WAM defines a specialized register-based architecture optimized for Prolog execution. It includes registers for argument passing, temporary variables, and environment management.
Memory Areas: It organizes memory into several distinct areas:

- Code area (for storing compiled programs)
- Heap (for storing structured terms)
- Stack (for environment and choice point frames)
- Trail (for variable bindings that may need to be undone during backtracking)

Instruction Set: The WAM defines a set of abstract machine instructions specifically designed for Prolog operations, including:

- Get instructions (for argument passing)
- Put instructions (for constructing terms)
- Unify instructions (for pattern matching)
- Control instructions (for procedure calls and returns)

Term Representation: It uses a tagged architecture to represent Prolog terms efficiently, with different tag bits indicating the type of term (variable, constant, structure, etc.).

Backtracking Mechanism: The WAM implements Prolog's backtracking through choice points, which store the machine state at points where alternative clauses could be tried.

The WAM has served as the basis for many Prolog implementations and has influenced the design of other logic programming systems. Its efficiency comes from several optimizations:

- Compile-time analysis to reduce runtime overhead
- Specialized instructions for common Prolog operations
- Efficient memory management strategies
- Smart register allocation
