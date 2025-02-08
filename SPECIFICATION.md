# LAM Formal Specification

## Overview
The Logical Abstract Machine (LAM) is a register–based abstract machine for logic programming. Its state consists of:

- **Registers:** A fixed–size vector storing partial terms.
- **Control Stack:** A stack of frames containing return addresses.
- **Environment Stack:** A stack of local variable frames.
- **Choice Stack:** A stack of choice points for backtracking.
- **Predicate Table & Index Table:** Structures mapping predicate names (and keys) to clause addresses.
- **Union–Find Structure:** Implements unification with a trailing mechanism for efficient rollback.

## Instruction Semantics

### PutConst(register, value)
- **Pre–condition:** register < R (where R is the number of registers)
- **Effect:** registers[register] is set to Const(value)

### Call(predicate)
- **Pre–condition:** predicate exists in the predicate table.
- **Effect:**
  1. Push a frame with the return PC onto the control stack.
  2. Save a choice point capturing the current registers, substitution, control stack, union–find trail length, and call level.
  3. Set the PC to the first clause address for predicate.

<!-- Instead of LaTeX formulas, we describe invariants in words. -->

## Unification Invariants
For any two terms t1 and t2, if unification succeeds, then there exists a substitution σ such that σ(t1) equals σ(t2). The union–find mechanism ensures that, after binding, the function resolve returns the canonical form. Backtracking via undo_trail reverts any changes made to the union–find structure.

## Dynamic Clause Management and Indexing
- **AssertClause(predicate, address):**  
  Adds the clause address to the predicate table entry for predicate and, if an index exists, to every key’s list in the index table.
- **RetractClause(predicate, address):**  
  Removes the clause address from both the predicate table and the index table.

## Tail-Call Optimization
Tail calls deallocate the current environment frame. That is, if ε is the current environment frame and a tail call is executed, then ε is removed and the tail-called predicate reuses the caller's control frame.

## Further Notes
(Additional details, assumptions, and potential extensions are documented here.)
