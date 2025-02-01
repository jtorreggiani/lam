// tests/test_machine.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

#[test]
fn test_put_const_instruction() {
    // Prolog equivalent:
    // ?- X = 42.
    // X = 42.
    
    let code = vec![Instruction::PutConst { register: 0, value: 42 }];
    let mut machine = Machine::new(2, code);

    // Before running, both registers should be None.
    assert_eq!(machine.registers[0], None);
    assert_eq!(machine.registers[1], None);

    // Execute one instruction.
    let cont = machine.step();
    assert!(cont);

    // After execution, register 0 should contain the constant 42.
    assert_eq!(machine.registers[0], Some(Term::Const(42)));
    // Register 1 remains unchanged.
    assert_eq!(machine.registers[1], None);

    // There should be no further instruction.
    assert!(!machine.step());
}

#[test]
fn test_put_var_instruction() {
    // Prolog equivalent:
    // ?- X = X.
    // true.
    
    let code = vec![Instruction::PutVar { register: 1, name: "X".to_string() }];
    let mut machine = Machine::new(2, code);

    // Run the machine.
    machine.run();

    // Register 1 should now contain a variable "X".
    assert_eq!(machine.registers[1], Some(Term::Var("X".to_string())));
}

#[test]
fn test_get_const_unification_success() {
    // Prolog equivalent:
    // ?- X = 42, X = 42.
    // X = 42.
    
    let code = vec![
        Instruction::PutConst { register: 0, value: 42 },
        Instruction::GetConst { register: 0, value: 42 },
    ];
    let mut machine = Machine::new(1, code);

    machine.run();

    // Unification should succeed and register 0 remains as 42.
    assert_eq!(machine.registers[0], Some(Term::Const(42)));
}

#[test]
fn test_get_const_unification_failure() {
    // Prolog equivalent:
    // ?- X = 42, X = 100.
    // false.
    
    let code = vec![
        Instruction::PutConst { register: 0, value: 42 },
        Instruction::GetConst { register: 0, value: 100 },
    ];
    let mut machine = Machine::new(1, code);

    machine.run();

    // Our current implementation simply prints an error on unification failure,
    // and leaves the register unchanged.
    assert_eq!(machine.registers[0], Some(Term::Const(42)));
}

#[test]
fn test_get_var_unification() {
    // Prolog equivalent:
    // ?- X = X, Y = X.
    // X = X,
    // Y = X.
    
    let code = vec![
        Instruction::PutVar { register: 0, name: "X".to_string() },
        Instruction::GetVar { register: 0, name: "Y".to_string() },
    ];
    let mut machine = Machine::new(1, code);

    machine.run();

    // The register should remain unchanged (still holds variable "X").
    assert_eq!(machine.registers[0], Some(Term::Var("X".to_string())));

    // The substitution environment should record that "Y" is bound to "X".
    let binding = machine.substitution.get("Y").cloned();
    assert_eq!(binding, Some(Term::Var("X".to_string())));
}

#[test]
fn test_call_and_proceed_with_lookup() {
    // We construct a program with 6 instructions arranged as follows:
    //
    // 0: PutConst   reg0, 10         ; Main code: set reg0 to 10.
    // 1: Call       predicate "dummy_pred"  ; Call using lookup.
    // 2: (This instruction is skipped, as control jumps to predicate code.)
    // 3: PutConst   reg1, 20         ; Predicate code: set reg1 to 20.
    // 4: Proceed                     ; End of predicate: return.
    // 5: PutConst   reg2, 30         ; Main continuation: set reg2 to 30.
    let code = vec![
        Instruction::PutConst { register: 0, value: 10 },
        Instruction::Call { predicate: "dummy_pred".to_string() },
        Instruction::PutConst { register: 99, value: 0 }, // dummy instruction (won't execute)
        Instruction::PutConst { register: 1, value: 20 },
        Instruction::Proceed,
        Instruction::PutConst { register: 2, value: 30 },
    ];
    
    let mut machine = Machine::new(3, code);
    // Register the predicate "dummy_pred" to start at index 3.
    machine.register_predicate("dummy_pred".to_string(), 3);
    
    // Run the entire program.
    machine.run();
    
    // Check expected register values.
    assert_eq!(machine.registers[0], Some(Term::Const(10)));
    assert_eq!(machine.registers[1], Some(Term::Const(20)));
    assert_eq!(machine.registers[2], Some(Term::Const(30)));
    
    // The control stack should be empty at the end.
    assert_eq!(machine.control_stack.len(), 0);
}