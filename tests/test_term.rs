use lam::term::Term; // adjust the crate name accordingly

#[test]
fn test_create_constant() {
    let term = Term::Const(42);
    assert_eq!(term, Term::Const(42));
}

#[test]
fn test_create_variable() {
    let term = Term::Var("X".to_string());
    assert_eq!(term, Term::Var("X".to_string()));
}