use lam::machine::core::Machine;
use lam::machine::term::Term;

#[test]
fn test_compound_unification() {
    // Set up a compound term f(42, X) in register 0, where X has id 1.
    let term1 = Term::Compound(
        "f".to_string(),
        vec![Term::Const(42), Term::Var(1)],
    );
    
    let mut machine = Machine::new(1, vec![]);
    machine.registers[0] = Some(term1);
    
    // Unify a new variable Y (with id 2) with the second argument.
    if let Some(compound_term) = machine.registers[0].clone() {
        if let Term::Compound(_, args) = compound_term {
            let success = machine.unify(&Term::Var(2), &args[1]);
            assert!(success.is_ok(), "Unification of the compound subterm failed");
        } else {
            panic!("Register 0 does not contain a compound term");
        }
    } else {
        panic!("Register 0 is empty");
    }

    assert_eq!(machine.uf.resolve(&Term::Var(2)), Term::Var(1));
}
