#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::term::Term;

    #[test]
    fn test_compound_unification() {
        // Create a compound term f(42, X) in register 0, where X is Var(1).
        let term1 = Term::Compound("f".to_string(), vec![Term::Const(42), Term::Var(1)]);
        
        let mut machine = Machine::new(1, vec![]);
        machine.registers[0] = Some(term1);
        
        // Unify a new variable Y (with id 2) with the second argument.
        if let Some(Term::Compound(_, ref args)) = machine.registers[0].clone() {
            machine.unify(&Term::Var(2), &args[1]).expect("Unification should succeed");
        } else {
            panic!("Register 0 does not contain a compound term");
        }

        assert_eq!(machine.uf.resolve(&Term::Var(2)), Term::Var(1));
    }
}
