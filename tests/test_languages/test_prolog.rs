use lam::languages::prolog::ast::{
    PrologClause,
    PrologGoal,
    PrologTerm
};

use lam::languages::prolog::parser::PrologParser;

#[test]
fn test_fact_ast() {
    // Fact: parent('Homer', 'Bart').
    let fact = PrologClause {
        head: PrologGoal {
            predicate: "parent".to_string(),
            args: vec![
                PrologTerm::Atom("Homer".to_string()),
                PrologTerm::Atom("Bart".to_string()),
            ],
        },
        body: None,
    };
    // Check that the AST is built as expected.
    assert_eq!(fact.head.predicate, "parent");
    assert!(fact.body.is_none());
}

#[test]
fn test_parse_fact_unquoted_atoms() {
    let input = "parent(homer, bart).";
    let mut parser = PrologParser::new(input);
    let fact = parser.parse_fact().unwrap();
    assert_eq!(fact.head.predicate, "parent");
    assert_eq!(fact.head.args,
                vec![
                    PrologTerm::Atom("homer".to_string()),
                    PrologTerm::Atom("bart".to_string())
                ]);
}

#[test]
fn test_parse_fact_quoted_atoms() {
    let input = "parent('Homer', 'Bart').";
    let mut parser = PrologParser::new(input);
    let fact = parser.parse_fact().unwrap();
    // In our design, quoted atoms are parsed as Atom.
    assert_eq!(fact.head.args,
                vec![
                    PrologTerm::Atom("Homer".to_string()),
                    PrologTerm::Atom("Bart".to_string())
                ]);
}

#[test]
fn test_parse_query() {
    let input = "likes(X, pizza).";
    let mut parser = PrologParser::new(input);
    let query = parser.parse_query().unwrap();
    assert_eq!(query.len(), 1);
    assert_eq!(query[0].predicate, "likes");
    // Variable names start with an uppercase letter.
    match &query[0].args[0] {
        PrologTerm::Var(name) => assert_eq!(name, "X"),
        _ => panic!("Expected variable"),
    }
}