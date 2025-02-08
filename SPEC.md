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
For each instruction, we define its effect on the machine state:

### PutConst(register, value)
- **Pre–condition:** `register < R` (where \( R \) is the number of registers)
- **Effect:** \( \text{registers}[register] \gets \text{Const}(value) \)

### Call(predicate)
- **Pre–condition:** `predicate` exists in the predicate table.
- **Effect:**
  1. Push a frame with return PC onto the control stack.
  2. Save a choice point with the current registers, substitution, control stack, union–find trail length, and call level.
  3. Set \( \text{PC} \) to the first clause address for `predicate`.

<!-- Continue for other instructions such as Cut, TailCall, etc. -->

## Unification Invariants
Let \( t_1 \) and \( t_2 \) be terms. If unification succeeds, then there exists a substitution \( \sigma \) such that:
\[
\sigma(t_1) = \sigma(t_2)
\]
The union–find mechanism ensures that after a binding, the function `resolve` returns the canonical form. Backtracking via `undo_trail` reverts changes in the union–find structure.

## Dynamic Clause Management and Indexing
- **AssertClause(predicate, address):**  
  Adds `address` to the predicate table entry for `predicate` and, if an index exists, to each key’s list in the index table.
- **RetractClause(predicate, address):**  
  Removes `address` from the predicate table and index table.

## Tail-Call Optimization
Tail calls deallocate the current environment frame. Formally, if \( \epsilon \) is the current environment frame and a tail call is executed, then \( \epsilon \) is removed, and the tail-called predicate reuses the caller's control frame.

## Further Notes
(Additional details, assumptions, and potential extensions may be documented here.)
