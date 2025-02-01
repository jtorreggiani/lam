// tests/test_higher_order.rs

use lam::term::Term;
use lam::lambda::{substitute, beta_reduce};

/// Test that substitution works correctly for lambda terms.
///
/// In this test, we substitute in the term f(x) for variable x.
#[test]
fn test_substitution() {
    // Let term be f(x), represented as Compound("f", [Var("x")])
    let term = Term::Compound("f".to_string(), vec![Term::Var("x".to_string())]);
    // Substitute x with Const(42)
    let result = substitute(&term, "x", &Term::Const(42));
    let expected = Term::Compound("f".to_string(), vec![Term::Const(42)]);
    assert_eq!(result, expected);
}

/// Test beta-reduction for the identity lambda.
///
/// The identity function is defined as λx. x. Applying it to 42 should yield 42.
#[test]
fn test_beta_reduce_identity() {
    // Identity function: Lambda("x", Var("x"))
    let identity = Term::Lambda("x".to_string(), Box::new(Term::Var("x".to_string())));
    // Application: (λx. x) 42
    let app = Term::App(Box::new(identity), Box::new(Term::Const(42)));
    let result = beta_reduce(&app);
    assert_eq!(result, Term::Const(42));
}

/// Test beta-reduction for a non-trivial lambda expression.
///
/// Define a lambda term: λx. f(x, 1), and apply it to 2. Expected result is f(2,1).
#[test]
fn test_beta_reduce_complex() {
    // Lambda term: λx. Compound("f", [Var("x"), Const(1)])
    let lambda_term = Term::Lambda(
        "x".to_string(),
        Box::new(Term::Compound(
            "f".to_string(),
            vec![Term::Var("x".to_string()), Term::Const(1)],
        )),
    );
    // Application: (λx. f(x, 1)) 2
    let app = Term::App(Box::new(lambda_term), Box::new(Term::Const(2)));
    let result = beta_reduce(&app);
    let expected = Term::Compound("f".to_string(), vec![Term::Const(2), Term::Const(1)]);
    assert_eq!(result, expected);
}