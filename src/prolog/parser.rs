// src/prolog/parser.rs
//! A minimal Prolog parser.
//!
//! This parser assumes that each clause is on its own line and that each clause ends
//! with a period. Lines starting with "%", "//", or "?-" (queries) are ignored.
//!
//! It supports facts and rules, where rules are written using ":-".
//! Single-quoted strings are parsed as atoms. Variables start with an uppercase letter
//! or an underscore. Numbers are parsed as integers. Compound terms use a functor
//! followed by a parenthesized, comma-separated list of arguments.

use crate::prolog::ast::{Clause, Term};

/// Errors that can occur during parsing.
#[derive(Debug)]
pub enum ParseError {
    /// An expected token was missing or the syntax was incorrect.
    UnexpectedToken(String),
    /// The input was empty.
    IncompleteInput,
}

/// Parses an entire Prolog program from the given input string.
///
/// Assumptions:
/// - Each clause is on its own line.
/// - Lines starting with "?-" (queries) are ignored.
/// - Each clause must end with a period.
/// - A clause containing ":-" is interpreted as a rule; otherwise, it is a fact.
pub fn parse_program(input: &str) -> Result<Vec<Clause>, ParseError> {
    let mut clauses = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();
        // Skip empty lines and comment lines.
        if trimmed.is_empty() || trimmed.starts_with('%') || trimmed.starts_with("//") {
            continue;
        }
        // Ignore query lines.
        if trimmed.starts_with("?-") {
            continue;
        }
        // Each clause must end with a period.
        if !trimmed.ends_with('.') {
            return Err(ParseError::UnexpectedToken(
                "Clause must end with a period".to_string(),
            ));
        }
        // Remove the trailing period.
        let clause_str = trimmed[..trimmed.len() - 1].trim();
        if let Some(idx) = clause_str.find(":-") {
            let head_str = clause_str[..idx].trim();
            let body_str = clause_str[idx + 2..].trim();
            let head = parse_term(head_str)?;
            // Instead of naïvely splitting on commas, use split_goals which respects parentheses.
            let body_parts = split_goals(body_str);
            let mut body = Vec::new();
            for part in body_parts {
                let term = parse_term(part)?;
                body.push(term);
            }
            clauses.push(Clause::Rule { head, body });
        } else {
            let head = parse_term(clause_str)?;
            clauses.push(Clause::Fact { head });
        }
    }
    Ok(clauses)
}

/// Splits the input string on commas that are not inside parentheses.
/// This ensures that commas inside compound terms are not treated as separators.
fn split_goals(s: &str) -> Vec<&str> {
    let mut goals = Vec::new();
    let mut start = 0;
    let mut paren_count = 0;
    for (i, c) in s.char_indices() {
        match c {
            '(' => paren_count += 1,
            ')' => {
                if paren_count > 0 {
                    paren_count -= 1;
                }
            }
            ',' if paren_count == 0 => {
                goals.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        goals.push(&s[start..]);
    }
    // Trim each goal and filter out any empty strings.
    goals.into_iter().map(|g| g.trim()).filter(|g| !g.is_empty()).collect()
}

/// Parses a single term from the given input string.
///
/// Supported term types:
/// - **Single-quoted strings:** e.g. `'Hello world'` is parsed as `Atom("Hello world")`.
/// - **Variables:** A token that starts with an uppercase letter or underscore.
/// - **Numbers:** Parsed as integers.
/// - **Compound terms:** A functor followed by a parenthesized, comma‑separated list of arguments.
/// - **Atoms:** All other tokens.
pub fn parse_term(input: &str) -> Result<Term, ParseError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(ParseError::IncompleteInput);
    }
    // Handle single-quoted strings.
    if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        return Ok(Term::Atom(s[1..s.len()-1].to_string()));
    }
    // Variables: tokens starting with uppercase or underscore.
    let first = s.chars().next().unwrap();
    if first.is_uppercase() || first == '_' {
        return Ok(Term::Var(s.to_string()));
    }
    // Numbers: tokens starting with a digit.
    if first.is_digit(10) {
        return s.parse::<i32>()
            .map(Term::Number)
            .map_err(|_| ParseError::UnexpectedToken(format!("Invalid number: {}", s)));
    }
    // Compound term: check for '('.
    if let Some(pos) = s.find('(') {
        let functor = s[..pos].trim().to_string();
        // Assume the closing parenthesis is the last character.
        let args_str = s[pos+1..].trim_end_matches(')').trim();
        let args: Result<Vec<_>, _> = if args_str.is_empty() {
            Ok(vec![])
        } else {
            // Use split_goals to split arguments while ignoring commas inside nested terms.
            let parts = split_goals(args_str);
            parts.into_iter().map(|arg| parse_term(arg)).collect()
        };
        return Ok(Term::Compound(functor, args?));
    }
    // Otherwise, treat the token as an atom.
    Ok(Term::Atom(s.to_string()))
}
