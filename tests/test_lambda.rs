// File: tests/test_lambda.rs

use lam::lambda::{substitute, beta_reduce};
use lam::term::Term;

/// Test capture-avoiding substitution in a simple scenario where no renaming is needed.
/// 
/// Term: λ0. App(Var(0), Var(2))
/// Substitute variable 2 with Const(42).
/// Expected: λ0. App(Var(0), Const(42))
#[test]
fn test_substitution_no_capture() {
    let term = Term::Lambda(0, Box::new(Term::App(
        Box::new(Term::Var(0)),
        Box::new(Term::Var(2))
    )));
    let replacement = Term::Const(42);
    let substituted = substitute(&term, 2, &replacement);
    let expected = Term::Lambda(0, Box::new(Term::App(
        Box::new(Term::Var(0)),
        Box::new(Term::Const(42))
    )));
    assert_eq!(substituted, expected);
}

/// Test capture-avoiding substitution when the replacement’s free variables conflict with the lambda’s bound variable.
/// 
/// Term: λ0. Var(1)
/// Substitute variable 1 with Var(0).
/// Since Var(0) is free in the replacement and 0 is bound in the lambda,
/// alpha-renaming must occur. Assuming our fresh-variable generator produces 2,
/// the expected result is: λ2. Var(0)
#[test]
fn test_substitution_with_capture_avoidance() {
    let term = Term::Lambda(0, Box::new(Term::Var(1)));
    let replacement = Term::Var(0);
    let substituted = substitute(&term, 1, &replacement);
    let expected = Term::Lambda(2, Box::new(Term::Var(0)));
    assert_eq!(substituted, expected);
}

/// Test beta reduction for a simple application.
/// 
/// Term: (λ0. Var(0)) Const(42)
/// Expected beta reduction yields: Const(42)
#[test]
fn test_beta_reduction_simple() {
    let identity = Term::Lambda(0, Box::new(Term::Var(0)));
    let app = Term::App(Box::new(identity), Box::new(Term::Const(42)));
    let reduced = beta_reduce(&app);
    assert_eq!(reduced, Term::Const(42));
}

/// Test beta reduction in a nested lambda application.
/// 
/// Term: (λ0. (λ1. App(Var(0), Var(1)))) Const(7)
/// Expected beta reduction yields: λ1. App(Const(7), Var(1))
#[test]
fn test_beta_reduction_nested() {
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

/// Test substitution on advanced term variants (Prob, Constraint, Modal, Temporal, HigherOrder).
/// 
/// Term: Compound("test", [Prob(Var(5)), Constraint("eq", [Var(5), Const(10)]), 
///       Modal("necessarily", Var(5)), Temporal("always", Var(5)), HigherOrder(Var(5))])
/// Substitute variable 5 with Const(100).
/// Expected: Each occurrence of Var(5) is replaced by Const(100)
#[test]
fn test_substitution_on_advanced_variants() {
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

/// Test beta reduction on a term that includes advanced variants.
/// 
/// Although advanced variants (Prob, Constraint, etc.) are not reduced further,
/// beta reduction should recursively process them and leave them intact if not applicable.
/// 
/// Here we create an application inside a Prob variant.
/// Term: Prob(App((λ0. Var(0)) Const(55)))
/// Expected: Prob(Const(55))
#[test]
fn test_beta_reduction_with_advanced_variant() {
    let lambda = Term::Lambda(0, Box::new(Term::Var(0)));
    let app = Term::App(Box::new(lambda), Box::new(Term::Const(55)));
    let term = Term::Prob(Box::new(app));
    let reduced = beta_reduce(&term);
    let expected = Term::Prob(Box::new(Term::Const(55)));
    assert_eq!(reduced, expected);
}
