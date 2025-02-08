// tests/test_machine/test_lambda_calculus.rs

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use lam::machine::term::Term;
    use lam::machine::lambda::{free_vars, generate_fresh_var, substitute, beta_reduce};

    #[test]
    fn test_free_vars_var() {
        // For Var(0), free_vars should return {0}.
        let term = Term::Var(0);
        let mut expected = HashSet::new();
        expected.insert(0);
        assert_eq!(free_vars(&term), expected);
    }

    #[test]
    fn test_free_vars_const_and_str() {
        // Constants and strings should have an empty set of free variables.
        let term_const = Term::Const(42);
        let term_str = Term::Str("hello".to_string());
        assert!(free_vars(&term_const).is_empty());
        assert!(free_vars(&term_str).is_empty());
    }

    #[test]
    fn test_free_vars_compound() {
        // For Compound("f", [Var(0), Const(10)]), free_vars should be {0}.
        let term = Term::Compound("f".to_string(), vec![Term::Var(0), Term::Const(10)]);
        let mut expected = HashSet::new();
        expected.insert(0);
        assert_eq!(free_vars(&term), expected);
    }

    #[test]
    fn test_free_vars_lambda() {
        // For Lambda(0, Box::new(Var(0))), free_vars should be empty.
        let term = Term::Lambda(0, Box::new(Term::Var(0)));
        assert!(free_vars(&term).is_empty());

        // For Lambda(0, Box::new(Var(1))), free_vars should be {1}.
        let term2 = Term::Lambda(0, Box::new(Term::Var(1)));
        let mut expected = HashSet::new();
        expected.insert(1);
        assert_eq!(free_vars(&term2), expected);
    }

    #[test]
    fn test_free_vars_application() {
        // For App(Box::new(Var(0)), Box::new(Var(1))), free_vars should be {0, 1}.
        let term = Term::App(Box::new(Term::Var(0)), Box::new(Term::Var(1)));
        let mut expected = HashSet::new();
        expected.insert(0);
        expected.insert(1);
        assert_eq!(free_vars(&term), expected);
    }

    #[test]
    fn test_generate_fresh_var() {
        // For a term with free variables {0, 1} and a replacement with free variables {1, 2},
        // the union is {0, 1, 2}. The generated fresh variable should not be in this set.
        let term = Term::Compound("f".to_string(), vec![Term::Var(0), Term::Var(1)]);
        let replacement = Term::Compound("g".to_string(), vec![Term::Var(1), Term::Var(2)]);
        let fresh = generate_fresh_var(&term, &replacement);
        let union: HashSet<_> = free_vars(&term).union(&free_vars(&replacement)).cloned().collect();
        assert!(!union.contains(&fresh));
    }

    #[test]
    fn test_substitute_simple() {
        // Substitute Var(1) with Const(42) in f(Var(1), Const(10)) should yield f(Const(42), Const(10)).
        let term = Term::Compound("f".to_string(), vec![Term::Var(1), Term::Const(10)]);
        let result = substitute(&term, 1, &Term::Const(42));
        let expected = Term::Compound("f".to_string(), vec![Term::Const(42), Term::Const(10)]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_beta_reduce_identity() {
        // (Lambda(0, Box::new(Var(0))) applied to Const(99)) should beta–reduce to Const(99).
        let identity = Term::Lambda(0, Box::new(Term::Var(0)));
        let application = Term::App(Box::new(identity), Box::new(Term::Const(99)));
        let result = beta_reduce(&application);
        assert_eq!(result, Term::Const(99));
    }

    #[test]
    fn test_beta_reduce_complex() {
        // Lambda(0, Box::new(Compound("f", [Var(0), Const(1)]))) applied to Const(2)
        // should reduce to Compound("f", [Const(2), Const(1)]).
        let lambda_term = Term::Lambda(
            0,
            Box::new(Term::Compound("f".to_string(), vec![Term::Var(0), Term::Const(1)]))
        );
        let application = Term::App(Box::new(lambda_term), Box::new(Term::Const(2)));
        let result = beta_reduce(&application);
        let expected = Term::Compound("f".to_string(), vec![Term::Const(2), Term::Const(1)]);
        assert_eq!(result, expected);
    }
}
