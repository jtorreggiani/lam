#[cfg(test)]
mod tests {
    use lam::machine::arithmetic::{Expression, evaluate, parse_expression};
    use lam::machine::error_handling::MachineError;

    // In these tests, our arithmetic expressions only use constants
    // so we can pass an empty vector (or a vector of Nones) as registers.

    // ============================
    // Tests for `evaluate`
    // ============================

    #[test]
    fn test_evaluate_constant() {
        let expr = Expression::Const(42);
        let result = evaluate(&expr, &vec![]);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_evaluate_addition() {
        let expr = Expression::Add(
            Box::new(Expression::Const(3)),
            Box::new(Expression::Const(4)),
        );
        let result = evaluate(&expr, &vec![]);
        assert_eq!(result.unwrap(), 7);
    }

    #[test]
    fn test_evaluate_subtraction() {
        let expr = Expression::Sub(
            Box::new(Expression::Const(10)),
            Box::new(Expression::Const(3)),
        );
        let result = evaluate(&expr, &vec![]);
        assert_eq!(result.unwrap(), 7);
    }

    #[test]
    fn test_evaluate_multiplication() {
        let expr = Expression::Mul(
            Box::new(Expression::Const(6)),
            Box::new(Expression::Const(7)),
        );
        let result = evaluate(&expr, &vec![]);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_evaluate_division() {
        let expr = Expression::Div(
            Box::new(Expression::Const(20)),
            Box::new(Expression::Const(5)),
        );
        let result = evaluate(&expr, &vec![]);
        assert_eq!(result.unwrap(), 4);
    }

    #[test]
    fn test_evaluate_division_by_zero() {
        let expr = Expression::Div(
            Box::new(Expression::Const(10)),
            Box::new(Expression::Const(0)),
        );
        let result = evaluate(&expr, &vec![]);
        match result {
            Err(MachineError::UnificationFailed(msg)) => {
                assert_eq!(msg, "Division by zero".to_string())
            },
            _ => panic!("Expected division by zero error"),
        }
    }

    // ============================
    // Tests for `parse_expression` + evaluation
    // ============================

    #[test]
    fn test_parse_expression_constant() {
        let input = "42";
        let expr = parse_expression(input).unwrap();
        // Expect an AST representing a constant 42.
        assert_eq!(expr, Expression::Const(42));
        let result = evaluate(&expr, &vec![]);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_parse_expression_simple_add() {
        let input = "3+4";
        let expr = parse_expression(input).unwrap();
        let expected = Expression::Add(
            Box::new(Expression::Const(3)),
            Box::new(Expression::Const(4)),
        );
        assert_eq!(expr, expected);
        let result = evaluate(&expr, &vec![]);
        assert_eq!(result.unwrap(), 7);
    }

    #[test]
    fn test_parse_expression_precedence() {
        // Without parentheses, multiplication should bind stronger than addition.
        // 3+4*2 should be parsed as 3+(4*2)=11.
        let input = "3+4*2";
        let expr = parse_expression(input).unwrap();
        let result = evaluate(&expr, &vec![]);
        assert_eq!(result.unwrap(), 11);
    }

    #[test]
    fn test_parse_expression_with_parentheses() {
        // Parentheses override default precedence: (3+4)*2 = 14.
        let input = "(3+4)*2";
        let expr = parse_expression(input).unwrap();
        let result = evaluate(&expr, &vec![]);
        assert_eq!(result.unwrap(), 14);
    }

    #[test]
    fn test_parse_expression_unary_minus() {
        // The parser should treat a leading minus as unary minus.
        // For example, "-3+5" is parsed as (0-3)+5 which equals 2.
        let input = "-3+5";
        let expr = parse_expression(input).unwrap();
        let result = evaluate(&expr, &vec![]);
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_parse_expression_complex() {
        // A more complex expression: (10-2)*(1+3) = 32.
        let input = "(10-2)*(1+3)";
        let expr = parse_expression(input).unwrap();
        let result = evaluate(&expr, &vec![]);
        assert_eq!(result.unwrap(), 32);
    }

    // ============================
    // Error cases in parsing
    // ============================

    #[test]
    fn test_parse_expression_error_invalid_character() {
        // Input contains an unexpected character.
        let input = "3+@4";
        let err = parse_expression(input).unwrap_err();
        // The error string should mention "Unexpected character"
        assert!(err.contains("Unexpected character"), "Error was: {}", err);
    }

    #[test]
    fn test_parse_expression_error_missing_parenthesis() {
        // Missing a closing parenthesis.
        let input = "(3+4*2";
        let err = parse_expression(input).unwrap_err();
        // The error should mention a missing closing parenthesis.
        assert!(err.contains("Missing closing parenthesis") || err.contains("unexpected end"), "Error was: {}", err);
    }

    #[test]
    fn test_parse_expression_error_extra_tokens() {
        // Extra tokens remain after parsing a valid expression.
        let input = "3+4 5";
        let err = parse_expression(input).unwrap_err();
        // The error should indicate an unexpected token.
        assert!(err.contains("Unexpected token") || err.contains("extra"), "Error was: {}", err);
    }
}
