#[cfg(test)]
mod tests {
    use lam::machine::lambda::{substitute, beta_reduce};
    use lam::machine::term::Term;

    #[test]
    fn test_substitution() {
        // f(x) with x represented as Var(0) should become f(42) after substituting 0 → Const(42)
        let term = Term::Compound("f".to_string(), vec![Term::Var(0)]);
        let result = substitute(&term, 0, &Term::Const(42));
        let expected = Term::Compound("f".to_string(), vec![Term::Const(42)]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_beta_reduce_identity() {
        // (λx. x) 42 should beta-reduce to 42.
        let identity = Term::Lambda(0, Box::new(Term::Var(0)));
        let app = Term::App(Box::new(identity), Box::new(Term::Const(42)));
        let result = beta_reduce(&app);
        assert_eq!(result, Term::Const(42));
    }

    #[test]
    fn test_beta_reduce_complex() {
        // λx. f(x, 1) applied to 2 should reduce to f(2, 1)
        let lambda_term = Term::Lambda(
            0,
            Box::new(Term::Compound("f".to_string(), vec![Term::Var(0), Term::Const(1)])),
        );
        let app = Term::App(Box::new(lambda_term), Box::new(Term::Const(2)));
        let result = beta_reduce(&app);
        let expected = Term::Compound("f".to_string(), vec![Term::Const(2), Term::Const(1)]);
        assert_eq!(result, expected);
    }
}
