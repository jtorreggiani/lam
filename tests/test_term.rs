// tests/test_term.rs

use lam::term::Term;

#[test]
fn test_create_const() {
    let term = Term::Const(10);
    let term2 = Term::Const(10);
    assert_eq!(term, term2);
    
    // Test inequality
    assert_ne!(Term::Const(10), Term::Const(20));
    
    // Test against different term types
    assert_ne!(Term::Const(10), Term::Var("10".to_string()));
    assert_ne!(Term::Const(10), Term::Compound("10".to_string(), vec![]));
    
    // Test boundary values
    let max = Term::Const(i32::MAX);
    let min = Term::Const(i32::MIN);
    assert_ne!(max, min);
}

#[test]
fn test_create_var() {
    // Test equality with different string allocations
    let term1 = Term::Var("X".to_string());
    let term2 = Term::Var("X".to_string());
    assert_eq!(term1, term2);
    
    // Test case sensitivity
    assert_ne!(Term::Var("x".to_string()), Term::Var("X".to_string()));
    
    // Test empty variable name
    let empty_var = Term::Var("".to_string());
    assert_ne!(empty_var, Term::Var("X".to_string()));
    
    // Test special characters in variable names
    let special_var = Term::Var("X_1".to_string());
    assert_eq!(special_var, Term::Var("X_1".to_string()));
}

#[test]
fn test_create_compound() {
    // Test empty argument list
    let empty_compound = Term::Compound("f".to_string(), vec![]);
    assert_eq!(empty_compound, Term::Compound("f".to_string(), vec![]));
    assert_ne!(empty_compound, Term::Compound("g".to_string(), vec![]));
    
    // Test argument order matters
    let term1 = Term::Compound("f".to_string(), vec![
        Term::Const(1),
        Term::Const(2),
    ]);
    
    let term2 = Term::Compound("f".to_string(), vec![
        Term::Const(2),
        Term::Const(1),
    ]);
    
    assert_ne!(term1, term2);
    
    // Test functor case sensitivity
    let case1 = Term::Compound("f".to_string(), vec![Term::Const(1)]);
    let case2 = Term::Compound("F".to_string(), vec![Term::Const(1)]);
    assert_ne!(case1, case2);
}

#[test]
fn test_nested_compound() {
    // Test deeply nested terms
    let nested = Term::Compound("f".to_string(), vec![
        Term::Compound("g".to_string(), vec![
            Term::Compound("h".to_string(), vec![
                Term::Var("X".to_string())
            ])
        ])
    ]);
    
    // Test equality with identical structure
    let nested2 = Term::Compound("f".to_string(), vec![
        Term::Compound("g".to_string(), vec![
            Term::Compound("h".to_string(), vec![
                Term::Var("X".to_string())
            ])
        ])
    ]);
    
    assert_eq!(nested, nested2);
    
    // Test inequality with different inner structure
    let different_nested = Term::Compound("f".to_string(), vec![
        Term::Compound("g".to_string(), vec![
            Term::Compound("h".to_string(), vec![
                Term::Var("Y".to_string())  // Different variable name
            ])
        ])
    ]);
    
    assert_ne!(nested, different_nested);
}

#[test]
fn test_clone() {
    // Test cloning of complex nested structure
    let original = Term::Compound("f".to_string(), vec![
        Term::Var("X".to_string()),
        Term::Compound("g".to_string(), vec![Term::Const(42)]),
    ]);
    
    let cloned = original.clone();
    assert_eq!(original, cloned);
    
    // Ensure deep cloning works
    match cloned {
        Term::Compound(_, args) => {
            assert_eq!(args.len(), 2);
        },
        _ => panic!("Cloned term lost its structure!")
    }
}