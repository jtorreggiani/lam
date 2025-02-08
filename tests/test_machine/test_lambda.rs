#[cfg(test)]
mod tests {
    use lam::machine::lambda::{substitute, beta_reduce};
    use lam::machine::term::Term;

    #[test]
    fn test_substitution_no_capture() {
        // λ0. (App(Var(0), Var(2))) with substitution 2 → Const(42)
        let term = Term::Lambda(0, Box::new(Term::App(
            Box::new(Term::Var(0)),
            Box::new(Term::Var(2))
        )));
        let result = substitute(&term, 2, &Term::Const(42));
        let expected = Term::Lambda(0, Box::new(Term::App(
            Box::new(Term::Var(0)),
            Box::new(Term::Const(42))
        )));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_substitution_with_capture_avoidance() {
        // λ0. Var(1) with substitution 1 → Var(0) must avoid capture.
        let term = Term::Lambda(0, Box::new(Term::Var(1)));
        let replacement = Term::Var(0);
        let substituted = substitute(&term, 1, &replacement);
        // Expect that the bound variable is alpha-renamed (e.g., to 2).
        let expected = Term::Lambda(2, Box::new(Term::Var(0)));
        assert_eq!(substituted, expected);
    }

    #[test]
    fn test_beta_reduction_simple() {
        // (λ0. Var(0)) applied to Const(42) should reduce to Const(42).
        let identity = Term::Lambda(0, Box::new(Term::Var(0)));
        let app = Term::App(Box::new(identity), Box::new(Term::Const(42)));
        let reduced = beta_reduce(&app);
        assert_eq!(reduced, Term::Const(42));
    }

    #[test]
    fn test_beta_reduction_nested() {
        // (λ0. (λ1. (App(Var(0), Var(1))))) applied to Const(7)
        let inner_lambda = Term::Lambda(1, Box::new(Term::App(
            Box::new(Term::Var(0)),
            Box::new(Term::Var(1))
        )));
        let outer_lambda = Term::Lambda(0, Box::new(inner_lambda));
        let app = Term::App(Box::new(outer_lambda), Box::new(Term::Const(7)));
        let reduced = beta_reduce(&app);
        let expected = Term::Lambda(1, Box::new(Term::App(
            Box::new(Term::Const(7)),
            Box::new(Term::Var(1))
        )));
        assert_eq!(reduced, expected);
    }

    #[test]
    fn test_substitution_on_advanced_variants() {
        // Test substitution in a compound term containing Prob, Constraint, Modal, Temporal, HigherOrder.
        let term = Term::Compound("test".to_string(), vec![
            Term::Prob(Box::new(Term::Var(5))),
            Term::Constraint("eq".to_string(), vec![Term::Var(5), Term::Const(10)]),
            Term::Modal("necessarily".to_string(), Box::new(Term::Var(5))),
            Term::Temporal("always".to_string(), Box::new(Term::Var(5))),
            Term::HigherOrder(Box::new(Term::Var(5))),
        ]);
        let substituted = substitute(&term, 5, &Term::Const(100));
        let expected = Term::Compound("test".to_string(), vec![
            Term::Prob(Box::new(Term::Const(100))),
            Term::Constraint("eq".to_string(), vec![Term::Const(100), Term::Const(10)]),
            Term::Modal("necessarily".to_string(), Box::new(Term::Const(100))),
            Term::Temporal("always".to_string(), Box::new(Term::Const(100))),
            Term::HigherOrder(Box::new(Term::Const(100))),
        ]);
        assert_eq!(substituted, expected);
    }

    #[test]
    fn test_beta_reduction_with_advanced_variant() {
        // Prob(App((λ0. Var(0)) Const(55))) should reduce to Prob(Const(55)).
        let lambda = Term::Lambda(0, Box::new(Term::Var(0)));
        let app = Term::App(Box::new(lambda), Box::new(Term::Const(55)));
        let term = Term::Prob(Box::new(app));
        let reduced = beta_reduce(&term);
        let expected = Term::Prob(Box::new(Term::Const(55)));
        assert_eq!(reduced, expected);
    }
}
