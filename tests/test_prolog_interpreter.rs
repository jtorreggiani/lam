use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::Builder;

#[test]
fn test_main_write_hello_world() {
    // Create a temporary file with a .pl extension
    let mut file = Builder::new()
        .suffix(".pl")
        .tempfile()
        .expect("Could not create temp file");
    writeln!(
        file,
        "main :-
  write('Hello world'),
  nl,
  halt."
    )
    .expect("Could not write to temp file");

    let path = file.path();

    // Run the lamc binary on the temporary file with the --execute flag.
    let mut cmd = Command::cargo_bin("lamc").expect("Could not find lamc binary");
    cmd.arg(path).arg("--execute")
       .assert()
       .stdout(predicate::str::contains("Hello world").and(predicate::str::contains("\n")));
}

#[test]
fn test_main_with_variable_assignment() {
    // This program assigns X to 'Hello world' then writes X.
    let mut file = Builder::new()
        .suffix(".pl")
        .tempfile()
        .expect("Could not create temp file");
    writeln!(
        file,
        "main :-
  X = 'Hello world',
  write(X),
  nl,
  halt."
    )
    .expect("Could not write to temp file");

    let path = file.path();

    let mut cmd = Command::cargo_bin("lamc").expect("Could not find lamc binary");
    cmd.arg(path).arg("--execute")
       .assert()
       .stdout(predicate::str::contains("Hello world").and(predicate::str::contains("\n")));
}

#[test]
fn test_main_parent() {
    // This program defines a fact and then calls main/0 which writes the result.
    let mut file = Builder::new()
        .suffix(".pl")
        .tempfile()
        .expect("Could not create temp file");
    writeln!(
        file,
        "parent(john, mary).

main :-
  parent(john, X),
  write(X),
  nl,
  halt."
    )
    .expect("Could not write to temp file");

    let path = file.path();

    let mut cmd = Command::cargo_bin("lamc").expect("Could not find lamc binary");
    cmd.arg(path).arg("--execute")
       .assert()
       .stdout(predicate::str::contains("mary").and(predicate::str::contains("\n")));
}
