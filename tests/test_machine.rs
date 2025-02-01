use lam::machine::{Instruction, Machine};
use lam::term::Term;

#[test]
fn test_put_const_instruction() {
    // Create a machine with 2 registers and a program that puts a constant in register 0.
    let code = vec![Instruction::PutConst { register: 0, value: 42 }];
    let mut machine = Machine::new(2, code);

    // Before running, registers should be None.
    assert_eq!(machine.registers[0], None);
    assert_eq!(machine.registers[1], None);

    // Execute one instruction.
    let cont = machine.step();
    assert!(cont);

    // After execution, register 0 should contain the constant.
    assert_eq!(machine.registers[0], Some(Term::Const(42)));
    // And register 1 should remain unchanged.
    assert_eq!(machine.registers[1], None);

    // There is no further instruction.
    assert!(!machine.step());
}

#[test]
fn test_put_var_instruction() {
    // Create a machine with 2 registers and a program that puts a variable in register 1.
    let code = vec![Instruction::PutVar { register: 1, name: "X".to_string() }];
    let mut machine = Machine::new(2, code);

    // Execute the instruction.
    machine.run();

    // Register 1 should now contain a variable "X".
    assert_eq!(machine.registers[1], Some(Term::Var("X".to_string())));
}
