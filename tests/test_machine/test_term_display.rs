#[cfg(test)]
mod tests {
    use lam::machine::term::Term;

    #[test]
    fn test_display_const() {
        let term = Term::Const(42);
        // For a constant, Display should print just the number.
        assert_eq!(term.to_string(), "42");
    }

    #[test]
    fn test_display_str() {
        let term = Term::Str("hello".to_string());
        // For a string constant, Display prints the string without quotes.
        assert_eq!(term.to_string(), "hello");
    }

    #[test]
    fn test_display_var() {
        let term = Term::Var(7);
        // For a variable, Display prints "Var(<id>)"
        assert_eq!(term.to_string(), "Var(7)");
    }

    #[test]
    fn test_display_compound_empty_args() {
        let term = Term::Compound("f".to_string(), vec![]);
        // A compound with no arguments should be printed as "f()"
        assert_eq!(term.to_string(), "f()");
    }

    #[test]
    fn test_display_compound_with_args() {
        let term = Term::Compound("f".to_string(), vec![Term::Const(1), Term::Var(2)]);
        // A compound with two arguments should print them comma–separated.
        assert_eq!(term.to_string(), "f(1, Var(2))");
    }

    #[test]
    fn test_display_compound_nested() {
        // Create a nested compound term: f(g(10), "test")
        let inner = Term::Compound("g".to_string(), vec![Term::Const(10)]);
        let term = Term::Compound("f".to_string(), vec![inner, Term::Str("test".to_string())]);
        assert_eq!(term.to_string(), "f(g(10), test)");
    }

    #[test]
    fn test_display_other_variant_falls_back_to_debug() {
        // For a variant not explicitly handled by the Display implementation,
        // the code should fall back to Debug formatting.
        // We'll use the Lambda variant as an example.
        let term = Term::Lambda(0, Box::new(Term::Var(0)));
        let output = term.to_string();
        // Since the Debug format for Lambda is not specified by Display,
        // we simply check that the output contains the word "Lambda".
        assert!(
            output.contains("Lambda"),
            "Expected Debug formatting to contain 'Lambda', got: {}",
            output
        );
    }
}
