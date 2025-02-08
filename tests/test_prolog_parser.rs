#[cfg(test)]
mod tests {
    use lam::prolog::ast::{Clause, Term};
    use lam::prolog::parser::{parse_program, ParseError, parse_term};

    #[test]
    fn test_parse_fact() {
        let input = "parent(john, mary).";
        let clauses = parse_program(input).expect("Should parse fact");
        assert_eq!(clauses.len(), 1);

        match &clauses[0] {
            Clause::Fact { head } => {
                if let Term::Compound(functor, args) = head {
                    assert_eq!(functor, "parent");
                    assert_eq!(args.len(), 2);
                    assert_eq!(args[0], Term::Atom("john".into()));
                    assert_eq!(args[1], Term::Atom("mary".into()));
                } else {
                    panic!("Expected a compound term in fact head");
                }
            }
            _ => panic!("Expected a fact clause"),
        }
    }

    #[test]
    fn test_parse_rule() {
        let input = "ancestor(X, Y) :- parent(X, Y).";
        let clauses = parse_program(input).expect("Should parse rule");
        assert_eq!(clauses.len(), 1);

        match &clauses[0] {
            Clause::Rule { head, body } => {
                // Check the head.
                if let Term::Compound(functor, args) = head {
                    assert_eq!(functor, "ancestor");
                    assert_eq!(args, &vec![Term::Var("X".into()), Term::Var("Y".into())]);
                } else {
                    panic!("Expected a compound term in rule head");
                }
                // Check that the body contains one goal.
                assert_eq!(body.len(), 1);
                if let Term::Compound(functor, args) = &body[0] {
                    assert_eq!(functor, "parent");
                    assert_eq!(args, &vec![Term::Var("X".into()), Term::Var("Y".into())]);
                } else {
                    panic!("Expected a compound term in rule body");
                }
            }
            _ => panic!("Expected a rule clause"),
        }
    }

    #[test]
    fn test_missing_period() {
        let input = "parent(john, mary)";
        let err = parse_program(input);
        assert!(err.is_err());
        if let Err(ParseError::UnexpectedToken(msg)) = err {
            assert!(msg.contains("Clause must end with a period"));
        } else {
            panic!("Expected an UnexpectedToken error");
        }
    }

    #[test]
    fn test_parse_number() {
        let term = parse_term("42").expect("Should parse a number");
        assert_eq!(term, Term::Number(42));
    }

    #[test]
    fn test_parse_variable() {
        let term = parse_term("X").expect("Should parse a variable");
        assert_eq!(term, Term::Var("X".into()));
    }

    #[test]
    fn test_parse_atom() {
        let term = parse_term("john").expect("Should parse an atom");
        assert_eq!(term, Term::Atom("john".into()));
    }

    #[test]
    fn test_parse_compound() {
        let term = parse_term("father(john, X)").expect("Should parse a compound term");
        if let Term::Compound(functor, args) = term {
            assert_eq!(functor, "father");
            assert_eq!(args.len(), 2);
            assert_eq!(args[0], Term::Atom("john".into()));
            assert_eq!(args[1], Term::Var("X".into()));
        } else {
            panic!("Expected a compound term");
        }
    }
}
