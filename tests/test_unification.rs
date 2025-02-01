// tests/unification.rs

use lam::machine::Machine;
use lam::term::Term;

#[test]
fn test_compound_unification() {
    // Set up a compound term f(42, X) in register 0.
    let term1 = Term::Compound(
        "f".to_string(),
        vec![Term::Const(42), Term::Var("X".to_string())],
    );
    
    let mut machine = Machine::new(1, vec![]);
    machine.registers[0] = Some(term1);
    
    // Instead of borrowing machine.registers[0] immutably, clone its value.
    if let Some(compound_term) = machine.registers[0].clone() {
        if let Term::Compound(_, args) = compound_term {
            // We want to unify the second argument (args[1], which is Var("X"))
            // with a new variable Y.
            // To get a binding of "Y" -> Var("X"), we call unify with Y as the left-hand term.
            let success = machine.unify(&Term::Var("Y".to_string()), &args[1]);
            assert!(success, "Unification of the compound subterm failed");
        } else {
            panic!("Register 0 does not contain a compound term");
        }
    } else {
        panic!("Register 0 is empty");
    }
    
    // Check that the substitution environment has the binding "Y" -> Var("X")
    let binding = machine.substitution.get("Y").cloned();
    assert_eq!(binding, Some(Term::Var("X".to_string())));
}
