use lam::languages::prolog::ast::{
    PrologClause,
    PrologGoal,
    PrologTerm
};
use lam::languages::prolog::parser::PrologParser;
use std::process::Command;

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
    // Variables (starting with an uppercase letter) are parsed as Var.
    match &query[0].args[0] {
        PrologTerm::Var(name) => assert_eq!(name, "X"),
        _ => panic!("Expected variable"),
    }
}

#[test]
fn test_parse_rule() {
    // Rule: mother(X, Y) :- parent(X, Y), female(X).
    let input = "mother(X, Y) :- parent(X, Y), female(X).";
    let mut parser = PrologParser::new(input);
    let clause = parser.parse_clause().unwrap();
    assert_eq!(clause.head.predicate, "mother");
    assert_eq!(clause.head.args,
               vec![
                   PrologTerm::Var("X".to_string()),
                   PrologTerm::Var("Y".to_string())
               ]);
    let body = clause.body.expect("Expected a rule body");
    assert_eq!(body.len(), 2);
    // First goal: parent(X, Y)
    assert_eq!(body[0].predicate, "parent");
    assert_eq!(body[0].args,
               vec![
                   PrologTerm::Var("X".to_string()),
                   PrologTerm::Var("Y".to_string())
               ]);
    // Second goal: female(X)
    assert_eq!(body[1].predicate, "female");
    assert_eq!(body[1].args,
               vec![
                   PrologTerm::Var("X".to_string())
               ]);
}

#[test]
fn test_parse_directive() {
    // Directive: ":- main, halt." should be parsed as a directive with head "directive".
    let input = ":- main, halt.";
    let mut parser = PrologParser::new(input);
    let clause = parser.parse_clause().unwrap();
    assert_eq!(clause.head.predicate, "directive");
    let body = clause.body.expect("Expected directive body");
    assert_eq!(body.len(), 2);
    assert_eq!(body[0].predicate, "main");
    assert_eq!(body[0].args, Vec::<PrologTerm>::new());
    assert_eq!(body[1].predicate, "halt");
    assert_eq!(body[1].args, Vec::<PrologTerm>::new());
}

#[test]
fn test_parse_infix_term() {
    // Test that an infix '-' operator is parsed correctly: write(X-Y).
    let input = "write(X-Y).";
    let mut parser = PrologParser::new(input);
    let clause = parser.parse_clause().unwrap();
    assert_eq!(clause.head.predicate, "write");
    // The argument should be parsed as a compound term: -(Var("X"), Var("Y"))
    let expected = PrologTerm::Compound("-".to_string(), vec![
        PrologTerm::Var("X".to_string()),
        PrologTerm::Var("Y".to_string())
    ]);
    assert_eq!(clause.head.args, vec![expected]);
}

#[test]
fn test_parse_program_with_comments() {
    let input = r#"
    % Facts about parents
    parent(john, mary).
    parent(john, tom).
    parent(jane, mary).
    parent(jane, tom).
    parent(mary, ann).
    parent(tom, peter).

    % Facts about gender
    female(jane).
    female(mary).
    female(ann).
    male(john).
    male(tom).
    male(peter).

    % Rule for mother
    mother(X, Y) :- parent(X, Y), female(X).

    % Simple display rule
    main :- mother(X, Y), write(X-Y), nl, fail.
    main.

    :- main, halt.
    "#;
    let mut parser = PrologParser::new(input);
    let clauses = parser.parse_program().unwrap();

    // We expect 6 (parents) + 6 (gender) + 1 (mother rule) + 2 (main rules) + 1 (directive) = 16 clauses.
    assert_eq!(clauses.len(), 16);

    // Test first fact: parent(john, mary).
    let clause1 = &clauses[0];
    assert_eq!(clause1.head.predicate, "parent");
    assert_eq!(clause1.head.args,
               vec![
                   PrologTerm::Atom("john".to_string()),
                   PrologTerm::Atom("mary".to_string())
               ]);

    // Find the mother rule.
    let mother_clause = clauses.iter().find(|clause| clause.head.predicate == "mother")
        .expect("Mother clause not found");
    assert_eq!(mother_clause.head.args,
               vec![
                   PrologTerm::Var("X".to_string()),
                   PrologTerm::Var("Y".to_string())
               ]);
    let body = mother_clause.body.as_ref().expect("Expected body for mother rule");
    assert_eq!(body.len(), 2);
    assert_eq!(body[0].predicate, "parent");
    assert_eq!(body[0].args,
               vec![
                   PrologTerm::Var("X".to_string()),
                   PrologTerm::Var("Y".to_string())
               ]);
    assert_eq!(body[1].predicate, "female");
    assert_eq!(body[1].args,
               vec![
                   PrologTerm::Var("X".to_string())
               ]);

    // Find the directive clause.
    let directive_clause = clauses.iter().find(|clause| clause.head.predicate == "directive")
        .expect("Directive clause not found");
    let dir_body = directive_clause.body.as_ref().expect("Expected body for directive");
    assert_eq!(dir_body.len(), 2);
    assert_eq!(dir_body[0].predicate, "main");
    assert_eq!(dir_body[1].predicate, "halt");
}

#[test]
fn test_family_tree_output() {
    // Run the prolog binary with the sample program.
    let output = Command::new("cargo")
        .args(&["run", "--bin", "prolog", "examples/prolog/family_tree.pl"])
        .output()
        .expect("Failed to execute prolog binary");

    // Convert stdout from bytes to a String.
    let stdout = String::from_utf8_lossy(&output.stdout);

    // For debugging purposes you can print the output:
    // println!("Output:\n{}", stdout);

    // Check that the expected solution lines appear.
    // (Depending on your implementation, extra debug or logging lines may be printed.
    //  In that case, you might simply check that each expected substring is present.)
    assert!(
        stdout.contains("jane-mary"),
        "Output did not contain 'jane-mary':\n{}",
        stdout
    );
    assert!(
        stdout.contains("jane-tom"),
        "Output did not contain 'jane-tom':\n{}",
        stdout
    );
    assert!(
        stdout.contains("mary-ann"),
        "Output did not contain 'mary-ann':\n{}",
        stdout
    );
}