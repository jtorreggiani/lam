use lam::machine::core::Machine;
use lam::machine::instruction::Instruction;
use lam::term::Term;

#[test]
fn test_indexed_call() {
    let code = vec![
        // Main program: perform an indexed call.
        Instruction::IndexedCall { predicate: "p".to_string(), index_register: 0 },
        // Clause for predicate p when key is 1 (should not be chosen in this test).
        Instruction::PutConst { register: 0, value: 10 },
        Instruction::Fail,
        // Clause for predicate p when key is 2.
        Instruction::PutConst { register: 0, value: 20 },
        Instruction::Proceed,
    ];
    
    let mut machine = Machine::new(1, code);
    
    // Set up the index table:
    // Clause 1: for key [Const(1)], at address 1.
    machine.register_indexed_clause("p".to_string(), vec![Term::Const(1)], 1);
    // Clause 2: for key [Const(2)], at address 3.
    machine.register_indexed_clause("p".to_string(), vec![Term::Const(2)], 3);
    
    // Set register 0 to Const(2), which is the index key we want to use.
    machine.registers[0] = Some(Term::Const(2));
    
    let _ = machine.run();
    
    // We expect that the IndexedCall will select Clause 2, setting register 0 to 20.
    assert_eq!(machine.registers[0], Some(Term::Const(20)));
}
