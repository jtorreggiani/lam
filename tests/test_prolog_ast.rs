#[cfg(test)]
mod tests {
    use lam::prolog::ast::{Clause, Query, Term};

    #[test]
    fn test_term_equality_atoms() {
        let atom1 = Term::Atom("john".into());
        let atom2 = Term::Atom("john".into());
        let atom3 = Term::Atom("mary".into());
        assert_eq!(atom1, atom2);
        assert_ne!(atom1, atom3);
    }

    #[test]
    fn test_term_equality_variables() {
        let var1 = Term::Var("X".into());
        let var2 = Term::Var("X".into());
        let var3 = Term::Var("Y".into());
        assert_eq!(var1, var2);
        assert_ne!(var1, var3);
    }

    #[test]
    fn test_compound_term() {
        let term = Term::Compound(
            "parent".into(),
            vec![Term::Atom("john".into()), Term::Atom("mary".into())],
        );
        // Check that the functor is "parent" and there are two arguments.
        if let Term::Compound(functor, args) = &term {
            assert_eq!(functor, "parent");
            assert_eq!(args.len(), 2);
            assert_eq!(args[0], Term::Atom("john".into()));
            assert_eq!(args[1], Term::Atom("mary".into()));
        } else {
            panic!("Expected a compound term");
        }
    }

    #[test]
    fn test_fact_clause() {
        let fact = Clause::Fact {
            head: Term::Compound(
                "parent".into(),
                vec![Term::Atom("john".into()), Term::Atom("mary".into())],
            ),
        };
        // For a fact, pattern match and check the head.
        match fact {
            Clause::Fact { head } => {
                if let Term::Compound(functor, ref args) = head {
                    assert_eq!(functor, "parent");
                    assert_eq!(args.len(), 2);
                } else {
                    panic!("Expected a compound term for fact head");
                }
            }
            _ => panic!("Expected a fact clause"),
        }
    }

    #[test]
    fn test_rule_clause() {
        let rule = Clause::Rule {
            head: Term::Compound(
                "ancestor".into(),
                vec![Term::Var("X".into()), Term::Var("Y".into())],
            ),
            body: vec![Term::Compound(
                "parent".into(),
                vec![Term::Var("X".into()), Term::Var("Y".into())],
            )],
        };
        match rule {
            Clause::Rule { head, body } => {
                // Check head.
                if let Term::Compound(functor, ref args) = head {
                    assert_eq!(functor, "ancestor");
                    assert_eq!(args.len(), 2);
                } else {
                    panic!("Expected a compound term for rule head");
                }
                // Check body has one goal.
                assert_eq!(body.len(), 1);
                if let Term::Compound(functor, ref args) = &body[0] {
                    assert_eq!(functor, "parent");
                    assert_eq!(args.len(), 2);
                } else {
                    panic!("Expected a compound term in rule body");
                }
            }
            _ => panic!("Expected a rule clause"),
        }
    }

    #[test]
    fn test_query_structure() {
        let query = Query {
            goals: vec![Term::Compound(
                "ancestor".into(),
                vec![Term::Var("X".into()), Term::Var("Y".into())],
            )],
        };
        // Check that the query has one goal and that it is a compound term with functor "ancestor".
        assert_eq!(query.goals.len(), 1);
        if let Term::Compound(functor, ref args) = &query.goals[0] {
            assert_eq!(functor, "ancestor");
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected a compound term as query goal");
        }
    }
}
