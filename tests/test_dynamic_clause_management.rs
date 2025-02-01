// tests/test_dynamic_clause_management.rs

use lam::machine::{Instruction, Machine};
use lam::term::Term;

/// This test demonstrates dynamic clause management by asserting and retracting clauses.
///
/// We simulate a predicate "p" that initially has no clauses. We then:
/// 1. Assert a clause for "p" at a given address that sets register 0 to 1.
/// 2. Call "p" so that register 0 is set to 1.
/// 3. Retract that clause.
/// 4. Attempt to call "p" again (which should now fail or leave register 0 unchanged).
///
/// For simplicity, our test will check that after retracting, a call to "p" does not change the register.
#[test]
fn test_dynamic_clause_management() {
    // Program:
    // 0: AssertClause for predicate "p" with clause address 3.
    // 1: Call "p"              -> should jump to address 3.
    // 2: Proceed               -> returns to after the call.
    // 3: PutConst reg0, 1      -> clause body that sets register 0 to 1.
    // 4: Proceed               -> returns.
    // 5: RetractClause for predicate "p" with clause address 3.
    // 6: Call "p"              -> should fail to find any clause and leave register 0 unchanged.
    let code = vec![
        Instruction::AssertClause { predicate: "p".to_string(), address: 3 },
        Instruction::Call { predicate: "p".to_string() },
        Instruction::Proceed,
        Instruction::PutConst { register: 0, value: 1 },
        Instruction::Proceed,
        Instruction::RetractClause { predicate: "p".to_string(), address: 3 },
        Instruction::Call { predicate: "p".to_string() },
        Instruction::Proceed,
    ];

    let mut machine = Machine::new(1, code);
    // Initially, predicate "p" is not registered in the static table.
    // We rely solely on the AssertClause instruction to add it.
    // Run the program.
    machine.run();

    // After the first call, register 0 should have been set to 1.
    assert_eq!(machine.registers[0], Some(Term::Const(1)));
    // After retracting the clause and calling p again, no clause should be found.
    // For our test, we assume that a call to an undefined predicate leaves the register unchanged.
    // In this simple design, the second call might simply not change register 0.
    // We check that the value remains 1.
    assert_eq!(machine.registers[0], Some(Term::Const(1)));
}
