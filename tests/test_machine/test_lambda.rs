// tests/test_lambda.rs

#[cfg(test)]
mod tests {
    use lam::machine::lambda::{substitute, beta_reduce};
    use lam::machine::term::Term;

    // Convenience constructors for testing.
    fn var(id: usize) -> Term {
        Term::Var(id)
    }
    fn const_term(n: i32) -> Term {
        Term::Const(n)
    }
    fn str_term(s: &str) -> Term {
        Term::Str(s.to_string())
    }
    fn compound(f: &str, args: Vec<Term>) -> Term {
        Term::Compound(f.to_string(), args)
    }
    fn lambda(param: usize, body: Term) -> Term {
        Term::Lambda(param, Box::new(body))
    }
    fn app(fun: Term, arg: Term) -> Term {
        Term::App(Box::new(fun), Box::new(arg))
    }

    // ---------------------------
    // Tests for `substitute`
    // ---------------------------

    // If the term is a variable and it matches the substitution variable,
    // it is replaced by the replacement.
    #[test]
    fn test_substitute_var_matches() {
        let t = var(3);
        let replacement = const_term(42);
        let result = substitute(&t, 3, &replacement);
        assert_eq!(result, replacement);
    }

    // If the term is a variable but does not match, it remains unchanged.
    #[test]
    fn test_substitute_var_no_match() {
        let t = var(5);
        let replacement = const_term(42);
        let result = substitute(&t, 3, &replacement);
        assert_eq!(result, t);
    }

    // Constants and strings are unaffected.
    #[test]
    fn test_substitute_const_and_str() {
        let t1 = const_term(100);
        let t2 = str_term("hello");
        let replacement = var(10);
        let result1 = substitute(&t1, 1, &replacement);
        let result2 = substitute(&t2, 1, &replacement);
        assert_eq!(result1, t1);
        assert_eq!(result2, t2);
    }

    // Substitution recurses into compound terms.
    #[test]
    fn test_substitute_compound() {
        let t = compound("f", vec![var(1), const_term(2)]);
        let replacement = const_term(99);
        let result = substitute(&t, 1, &replacement);
        let expected = compound("f", vec![replacement.clone(), const_term(2)]);
        assert_eq!(result, expected);
    }

    // In a lambda, if the lambda's parameter equals the substitution variable,
    // the lambda is left unchanged.
    #[test]
    fn test_substitute_lambda_param_equal() {
        let t = lambda(3, var(4));
        let replacement = const_term(50);
        let result = substitute(&t, 3, &replacement);
        assert_eq!(result, t);
    }

    // When the lambda's parameter is different and does not appear in the free variables
    // of the replacement, substitution recurses normally.
    #[test]
    fn test_substitute_lambda_no_capture() {
        let t = lambda(2, var(1));
        let replacement = compound("g", vec![const_term(7)]);
        let result = substitute(&t, 1, &replacement);
        let expected = lambda(2, replacement.clone());
        assert_eq!(result, expected);
    }

    // When the lambda's parameter is different but appears in the free variables of the replacement,
    // alpha–conversion (capture avoidance) is triggered.
    #[test]
    fn test_substitute_lambda_with_capture_avoidance() {
        // Create a lambda with parameter 0 and body var(1).
        let t = lambda(0, var(1));
        // The replacement will be a compound term that has free variable 0.
        let replacement = compound("f", vec![var(0)]);
        // Substitute for variable 1. Since lambda parameter 0 appears in replacement's free vars,
        // the function should generate a fresh variable (expected to be 2 if union of free vars is {0,1})
        // and perform alpha conversion.
        let result = substitute(&t, 1, &replacement);
        let expected = lambda(2, replacement.clone());
        assert_eq!(result, expected);
    }

    // Substitution should recurse into applications.
    #[test]
    fn test_substitute_app() {
        let t = app(var(1), var(2));
        let replacement = const_term(100);
        let result = substitute(&t, 2, &replacement);
        let expected = app(var(1), replacement.clone());
        assert_eq!(result, expected);
    }

    // Substitution inside a Prob term.
    #[test]
    fn test_substitute_prob() {
        let t = Term::Prob(Box::new(var(3)));
        let replacement = const_term(123);
        let result = substitute(&t, 3, &replacement);
        let expected = Term::Prob(Box::new(replacement.clone()));
        assert_eq!(result, expected);
    }

    // Substitution inside a Constraint term.
    #[test]
    fn test_substitute_constraint() {
        let t = Term::Constraint("eq".to_string(), vec![var(2), const_term(50)]);
        let replacement = const_term(999);
        let result = substitute(&t, 2, &replacement);
        let expected = Term::Constraint("eq".to_string(), vec![replacement.clone(), const_term(50)]);
        assert_eq!(result, expected);
    }

    // Substitution inside a Modal term.
    #[test]
    fn test_substitute_modal() {
        let t = Term::Modal("necessarily".to_string(), Box::new(var(4)));
        let replacement = const_term(777);
        let result = substitute(&t, 4, &replacement);
        let expected = Term::Modal("necessarily".to_string(), Box::new(replacement.clone()));
        assert_eq!(result, expected);
    }

    // Substitution inside a Temporal term.
    #[test]
    fn test_substitute_temporal() {
        let t = Term::Temporal("always".to_string(), Box::new(var(5)));
        let replacement = const_term(888);
        let result = substitute(&t, 5, &replacement);
        let expected = Term::Temporal("always".to_string(), Box::new(replacement.clone()));
        assert_eq!(result, expected);
    }

    // Substitution inside a HigherOrder term.
    #[test]
    fn test_substitute_higher_order() {
        let t = Term::HigherOrder(Box::new(var(6)));
        let replacement = const_term(101);
        let result = substitute(&t, 6, &replacement);
        let expected = Term::HigherOrder(Box::new(replacement.clone()));
        assert_eq!(result, expected);
    }

    // ---------------------------
    // Tests for `beta_reduce`
    // ---------------------------

    // Beta reduction of an application where the function is a lambda:
    // (λ0. var(0)) applied to Const(99) should reduce to Const(99).
    #[test]
    fn test_beta_reduce_simple() {
        let lambda_identity = lambda(0, var(0));
        let app_term = app(lambda_identity, const_term(99));
        let result = beta_reduce(&app_term);
        assert_eq!(result, const_term(99));
    }

    // Beta reduction on an application where the function is not a lambda.
    // In this case, beta_reduce should recursively reduce both parts.
    #[test]
    fn test_beta_reduce_nested_app() {
        let t = app(var(1), const_term(5));
        let result = beta_reduce(&t);
        // Since var and const are irreducible, result should equal the original term.
        assert_eq!(result, t);
    }

    // Beta reduction on a lambda: it should recursively reduce its body.
    #[test]
    fn test_beta_reduce_lambda_body() {
        let t = lambda(0, app(var(0), const_term(3)));
        let result = beta_reduce(&t);
        // In this case, the body is already in normal form.
        assert_eq!(result, t);
    }

    // Beta reduction on a compound term: each argument is reduced.
    #[test]
    fn test_beta_reduce_compound() {
        let t = compound("f", vec![app(var(1), const_term(2)), const_term(10)]);
        let result = beta_reduce(&t);
        let expected = compound("f", vec![app(var(1), const_term(2)), const_term(10)]);
        assert_eq!(result, expected);
    }

    // Beta reduce for types that require recursive calls:
    // Prob, Constraint, Modal, Temporal, and HigherOrder.
    #[test]
    fn test_beta_reduce_wrapper_types() {
        let t_prob = Term::Prob(Box::new(app(lambda(0, var(0)), const_term(7))));
        let t_constraint = Term::Constraint("eq".to_string(), vec![app(lambda(0, var(0)), const_term(8))]);
        let t_modal = Term::Modal("possibly".to_string(), Box::new(app(lambda(0, var(0)), const_term(9))));
        let t_temporal = Term::Temporal("sometime".to_string(), Box::new(app(lambda(0, var(0)), const_term(10))));
        let t_higher = Term::HigherOrder(Box::new(app(lambda(0, var(0)), const_term(11))));

        let res_prob = beta_reduce(&t_prob);
        let res_constraint = beta_reduce(&t_constraint);
        let res_modal = beta_reduce(&t_modal);
        let res_temporal = beta_reduce(&t_temporal);
        let res_higher = beta_reduce(&t_higher);

        // Each (λ0. var(0)) applied to a constant should reduce to the constant.
        assert_eq!(res_prob, Term::Prob(Box::new(const_term(7))));
        assert_eq!(res_constraint, Term::Constraint("eq".to_string(), vec![const_term(8)]));
        assert_eq!(res_modal, Term::Modal("possibly".to_string(), Box::new(const_term(9))));
        assert_eq!(res_temporal, Term::Temporal("sometime".to_string(), Box::new(const_term(10))));
        assert_eq!(res_higher, Term::HigherOrder(Box::new(const_term(11))));
    }
}
