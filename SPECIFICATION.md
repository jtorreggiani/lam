# LAM Formal Specification

## Overview

The Logical Abstract Machine (LAM) is a register-based abstract machine for logic programming. Its state consists of the following components:

- **Registers:** A fixed-size vector (of size R) storing partial terms (constants, variables, compound terms, etc.).
- **Control Stack:** A stack of frames that record return addresses for predicate calls.
- **Environment Stack:** A stack of frames that hold local variable bindings.
- **Choice Stack:** A stack of choice points for backtracking. Each choice point records:
  - The current registers.
  - The current substitution.
  - The current control stack.
  - The union-find trail length.
  - The call level (typically the current length of the control stack).
- **Predicate Table & Index Table:** Structures mapping predicate names (and keys) to clause addresses.
- **Substitution:** A mapping (implicitly maintained via the union-find structure) that records variable bindings.
- **Union-Find Structure:** Implements unification using a trailing mechanism for efficient rollback.
- **Program Counter (PC):** An integer pointer into the program’s instruction list.

We denote the overall machine state as:

    S = (Registers, Control Stack, Environment Stack, Choice Stack, 
         Predicate Table, Index Table, Substitution, Union-Find, PC)

Each instruction I defines a state transition:

    S --[I]--> S'

---

## Instruction Semantics

Below is the specification of each instruction in the LAM instruction set.

### 1. PutConst { register, value }
- Precondition:
  - register < R   (where R is the number of registers)
- Effect:
  - Sets Registers[register] to Const(value).
  - Other machine state components remain unchanged.
  - The program counter (PC) is incremented normally.

---

### 2. PutVar { register, var_id, name }
- Precondition:
  - register < R.
  - var_id uniquely identifies the variable.
  - name is a human-readable identifier (used for debugging).
- Effect:
  - Sets Registers[register] to Var(var_id).
  - Updates the variable names mapping: variable_names[var_id] is set to name.
  - Other state components remain unchanged.

---

### 3. GetConst { register, value }
- Precondition:
  - register < R and Registers[register] is initialized.
  - The term in Registers[register] must be unifiable with Const(value).
- Effect:
  - Attempts to unify the term in Registers[register] with Const(value).
  - On success, variable bindings (via the union-find mechanism) are updated.
  - On failure, an error is returned (triggering backtracking if possible).
  - PC is incremented normally.

---

### 4. GetVar { register, var_id, name }
- Precondition:
  - register < R.
- Effect:
  - If Registers[register] is uninitialized, sets it to Var(var_id) and records variable_names[var_id] = name.
  - If the register holds a term, attempts to unify that term with Var(var_id), updating the union-find structure.
  - PC increments normally.

---

### 5. Call { predicate }
- Precondition:
  - predicate must exist either as a built-in or in the predicate table.
- Effect:
  - If the predicate is built-in:
      - Invokes the corresponding built-in function.
  - If the predicate is user-defined:
      1. Pushes a control frame onto the control stack (saving the current PC as the return address).
      2. Saves a choice point capturing the current registers, substitution, control stack, union-find trail length, and call level.
      3. Sets the PC to the first clause address for predicate.

---

### 6. Proceed
- Precondition:
  - At least one frame exists in the control stack.
- Effect:
  - Pops the top frame from the control stack.
  - Sets the PC to the return PC stored in that frame.
  - Other state components remain unchanged.

---

### 7. Choice { alternative }
- Precondition:
  - alternative is a valid clause address.
- Effect:
  - Saves a choice point containing the current registers, substitution, control stack, union-find trail length, and call level, with the alternative clause address stored.
  - PC remains unchanged until a failure triggers backtracking.

---

### 8. Allocate { n }
- Precondition:
  - n >= 0 is the number of local variables required.
- Effect:
  - Pushes a new environment frame onto the environment stack with n uninitialized slots.
  - Other state components are unaffected.

---

### 9. Deallocate
- Precondition:
  - The environment stack is non-empty.
- Effect:
  - Pops the top environment frame from the environment stack.
  - Other state remains unchanged.

---

### 10. ArithmeticIs { target, expression }
- Precondition:
  - target < R.
  - The arithmetic expression is syntactically valid and any referenced registers are initialized.
- Effect:
  - Evaluates the arithmetic expression using the current register values.
  - Stores the result as Const(result) in Registers[target].
  - Other state components remain unchanged.

---

### 11. SetLocal { index, value }
- Precondition:
  - The environment stack is non-empty.
  - index is a valid index into the top environment frame.
- Effect:
  - Updates the current environment frame by setting the slot at position index to Some(value).
  - Other state components remain unchanged.

---

### 12. GetLocal { index, register }
- Precondition:
  - The environment stack is non-empty.
  - index is a valid index in the top environment frame.
  - register < R.
- Effect:
  - Retrieves the term from the top environment frame at position index.
  - If Registers[register] already holds a term, attempts to unify it with the retrieved term.
  - Otherwise, copies the term into Registers[register].
  - Other state components remain unchanged.

---

### 13. Fail
- Precondition:
  - There exists at least one choice point in the choice stack.
- Effect:
  - Triggers backtracking by popping the most recent choice point.
  - Restores registers, substitution, control stack, and union-find trail (using undo_trail) to the saved state.
  - If alternative clause addresses are available in the choice point, selects one (and may push an updated choice point if alternatives remain) and sets the PC accordingly.
  - If no choice point is available, returns a failure error.

---

### 14. GetStructure { register, functor, arity }
- Precondition:
  - register < R and Registers[register] is initialized.
  - The term in the register is expected to be a compound term.
- Effect:
  - Checks if the term in Registers[register] is a compound term with functor equal to functor and exactly arity arguments.
  - Succeeds if the check passes; otherwise, it fails (triggering backtracking).
  - No change is made to the state on success.

---

### 15. IndexedCall { predicate, index_register }
- Precondition:
  - index_register < R and Registers[index_register] is initialized.
  - The term in Registers[index_register] is used as a key.
  - The predicate must have an index entry in the index table.
- Effect:
  - Looks up the list of clause addresses for predicate using the key from Registers[index_register].
  - If a matching clause address is found, saves a choice point (capturing the current state) and sets the PC to that clause address.
  - If no matching clause is found, the instruction fails.

---

### 16. MultiIndexedCall { predicate, index_registers }
- Precondition:
  - Every register in index_registers is within bounds and initialized.
- Effect:
  - Constructs a composite key from the terms in the specified registers.
  - Looks up this key in the index table for predicate.
  - If matching clause addresses are found, saves a choice point and sets the PC to the first clause address.
  - Otherwise, the instruction fails.

---

### 17. TailCall { predicate }
- Precondition:
  - The environment stack is non-empty (i.e., there is an environment frame).
  - predicate must exist as a built-in or in the predicate table.
- Effect:
  - Environment Deallocation:
      - Pops the current environment frame, deallocating it.
  - Control Flow:
      - If predicate is built-in, invokes its function.
      - If predicate is user-defined and alternative clauses exist, saves a choice point; then sets the PC to the first clause address for predicate.
  - No new control frame is pushed; the tail call reuses the caller’s control frame.

---

### 18. AssertClause { predicate, address }
- Precondition:
  - predicate is provided as a string.
  - address is the code address of the new clause.
- Effect:
  - Appends address to the predicate table entry for predicate.
  - If an index entry exists for predicate, then for each key in that entry the new clause address is appended to the associated list.
  - Other state components remain unchanged.

---

### 19. RetractClause { predicate, address }
- Precondition:
  - predicate exists in the predicate table.
  - address is present in the predicate table for predicate.
- Effect:
  - Removes address from the predicate table entry for predicate.
  - Updates the index table by removing address from all keys associated with predicate.
  - If the clause address is not found, returns an error.
  - No other state is modified.

---

### 20. Cut
- Precondition:
  - The machine may have one or more choice points.
- Effect:
  - Removes from the choice stack all choice points that were created at the current call level or deeper 
    (i.e., those with a call level greater than or equal to the current control stack length).
  - This pruning prevents backtracking to alternative clauses from the current predicate call.
  - Other state (registers, environment, etc.) remains unchanged.

---

### 21. BuildCompound { target, functor, arg_registers }
- Precondition:
  - target < R.
  - Every register in arg_registers is within bounds and holds an initialized term.
- Effect:
  - Reads the terms from the registers listed in arg_registers (in order).
  - Constructs a compound term: Compound(functor, args), where args is the vector of retrieved terms.
  - Stores the resulting compound term in Registers[target].
  - Other state remains unchanged.

---

### 22. PutStr { register, value }
- Precondition:
  - register < R.
- Effect:
  - Stores Str(value) in Registers[register].
  - Other state remains unchanged.

---

### 23. GetStr { register, value }
- Precondition:
  - register < R and Registers[register] is initialized.
- Effect:
  - Attempts to unify the term in Registers[register] with Str(value).
  - On success, updates the register with the resolved term.
  - On failure, returns an error (triggering backtracking if possible).

---

### 24. Move { src, dst }
- Precondition:
  - src < R and dst < R.
- Effect:
  - Copies the content of Registers[src] to Registers[dst].
  - Other state components remain unchanged.

---

### 25. Halt
- Precondition:
  - None.
- Effect:
  - Stops execution by setting the PC to the end of the program.
  - No further instructions are executed.

---

## Invariants and Side Conditions

- **Register Bound Invariant:**  
  Every instruction that accesses a register must satisfy:  
  register < R.

- **Unification Invariant:**  
  For any two terms t1 and t2, if unification succeeds then there exists a substitution sigma such that:
  
      sigma(t1) = sigma(t2)
  
  The union-find mechanism (with trailing and rollback via undo_trail) guarantees this invariant.

- **Backtracking Invariant:**  
  When a Fail instruction triggers backtracking, the machine’s state (registers, substitution, control stack, union-find trail) is restored to the state saved in the most recent choice point.

- **Tail-Call Invariant:**  
  Executing a tail call deallocates the current environment frame. That is, if epsilon is the current environment frame, then after a tail call epsilon is removed and the tail-called predicate reuses the caller’s control frame.

- **Dynamic Clause Management:**  
  Every clause asserted dynamically appears in the predicate table and, if applicable, in the index table. When a clause is retracted, it is removed from both.

---

## Further Notes

- **Extensibility:**  
  This instruction set is designed to be extended. Additional built-in predicates (such as those for arithmetic comparisons, list processing, or constraints) can be added following the same formal specification style.

- **Error Handling:**  
  When a precondition is violated (e.g., register out-of-bounds, uninitialized term, unification failure), the machine returns a corresponding error (a variant of MachineError). Such errors trigger backtracking when applicable.

- **Assumptions:**  
  The specification assumes that unification is implemented via a union-find mechanism with trailing, and that choice points capture a complete snapshot of the machine state for proper backtracking.

- **Rendering:**  
  This document uses plain text and ASCII notation for formulas. For a version with rendered math, consider using GitHub Pages with MathJax/KaTeX or converting this document to PDF.
