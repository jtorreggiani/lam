#[cfg(test)]
mod tests {
    use lam::machine::term::Term;

    #[test]
    fn test_create_const() {
        let term = Term::Const(10);
        let term2 = Term::Const(10);
        assert_eq!(term, term2);
        
        assert_ne!(Term::Const(10), Term::Const(20));
        assert_ne!(Term::Const(10), Term::Var(10)); // Var(10) is a variable with id 10.
        assert_ne!(Term::Const(10), Term::Compound("10".to_string(), vec![]));
        
        let max = Term::Const(i32::MAX);
        let min = Term::Const(i32::MIN);
        assert_ne!(max, min);
    }

    #[test]
    fn test_create_var() {
        let term1 = Term::Var(0);
        let term2 = Term::Var(0);
        assert_eq!(term1, term2);
        assert_ne!(Term::Var(0), Term::Var(1));
    }

    #[test]
    fn test_create_compound() {
        let empty_compound = Term::Compound("f".to_string(), vec![]);
        assert_eq!(empty_compound, Term::Compound("f".to_string(), vec![]));
        assert_ne!(empty_compound, Term::Compound("g".to_string(), vec![]));
        
        let term1 = Term::Compound("f".to_string(), vec![Term::Const(1), Term::Const(2)]);
        let term2 = Term::Compound("f".to_string(), vec![Term::Const(2), Term::Const(1)]);
        assert_ne!(term1, term2);
        
        let case1 = Term::Compound("f".to_string(), vec![Term::Const(1)]);
        let case2 = Term::Compound("F".to_string(), vec![Term::Const(1)]);
        assert_ne!(case1, case2);
    }

    #[test]
    fn test_nested_compound() {
        let nested = Term::Compound("f".to_string(), vec![
            Term::Compound("g".to_string(), vec![
                Term::Compound("h".to_string(), vec![Term::Var(0)])
            ])
        ]);
        
        let nested2 = Term::Compound("f".to_string(), vec![
            Term::Compound("g".to_string(), vec![
                Term::Compound("h".to_string(), vec![Term::Var(0)])
            ])
        ]);
        
        assert_eq!(nested, nested2);
        
        let different_nested = Term::Compound("f".to_string(), vec![
            Term::Compound("g".to_string(), vec![
                Term::Compound("h".to_string(), vec![Term::Var(1)])
            ])
        ]);
        
        assert_ne!(nested, different_nested);
    }

    #[test]
    fn test_clone() {
        let original = Term::Compound("f".to_string(), vec![
            Term::Var(0),
            Term::Compound("g".to_string(), vec![Term::Const(42)]),
        ]);
        
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}
