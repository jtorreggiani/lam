use lam::machine::term::Term;
use lam::machine::lambda::{substitute, beta_reduce};

#[test]
fn test_substitution() {
    // Let term be f(x), represented as Compound("f", [Var(0)]) where 0 is the id for x.
    let term = Term::Compound("f".to_string(), vec![Term::Var(0)]);
    let result = substitute(&term, 0, &Term::Const(42));
    let expected = Term::Compound("f".to_string(), vec![Term::Const(42)]);
    assert_eq!(result, expected);
}

#[test]
fn test_beta_reduce_identity() {
    // Identity function: Lambda(0, Var(0))
    let identity = Term::Lambda(0, Box::new(Term::Var(0)));
    // Application: (λx. x) 42
    let app = Term::App(Box::new(identity), Box::new(Term::Const(42)));
    let result = beta_reduce(&app);
    assert_eq!(result, Term::Const(42));
}

#[test]
fn test_beta_reduce_complex() {
    // Lambda term: λx. f(x, 1) where x has id 0.
    let lambda_term = Term::Lambda(
        0,
        Box::new(Term::Compound(
            "f".to_string(),
            vec![Term::Var(0), Term::Const(1)],
        )),
    );
    // Application: (λx. f(x, 1)) 2
    let app = Term::App(Box::new(lambda_term), Box::new(Term::Const(2)));
    let result = beta_reduce(&app);
    let expected = Term::Compound("f".to_string(), vec![Term::Const(2), Term::Const(1)]);
    assert_eq!(result, expected);
}
