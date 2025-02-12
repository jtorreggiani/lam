// src/prolog/parser.rs
//! A minimal Prolog parser.
//!
//! This parser now supports multi–line clauses while enforcing that full
//! statements always end with a period. It does so by:
//! 1. Removing comment lines (lines starting with `%`, `//`, or `?-`).
//! 2. Joining the remaining lines into one long string.
//! 3. Verifying that the joined string ends with a period (if not, an error is raised).
//! 4. Splitting the string into clauses using a custom function that splits on
//!    periods ('.') that are not part of a decimal number.
//!
//! Each non–empty clause is then trimmed and parsed. The parser supports:
//! - Facts and rules (using ":-" for rules)
//! - Infix operator "=" (at the top level) by producing a compound term with functor "=" and two arguments.
//! - Single–quoted strings (parsed as atoms), numbers, variables (starting with uppercase or underscore),
//!   compound terms, and atoms.

use crate::prolog::ast::{Clause, Term};

/// Errors that can occur during parsing.
#[derive(Debug)]
pub enum ParseError {
    /// An expected token was missing or the syntax was incorrect.
    UnexpectedToken(String),
    /// The input was empty.
    IncompleteInput,
}

/// Splits a string on commas that are not inside parentheses or quotes.
/// This ensures that commas inside compound terms are not treated as separators.
fn split_goals(s: &str) -> Vec<&str> {
    let mut goals = Vec::new();
    let mut start = 0;
    let mut paren_count = 0;
    let mut in_quote = false;
    for (i, c) in s.char_indices() {
        match c {
            '\'' => in_quote = !in_quote,
            '(' if !in_quote => paren_count += 1,
            ')' if !in_quote => {
                if paren_count > 0 {
                    paren_count -= 1;
                }
            }
            ',' if paren_count == 0 && !in_quote => {
                goals.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        goals.push(&s[start..]);
    }
    goals.into_iter().map(|g| g.trim()).filter(|g| !g.is_empty()).collect()
}

/// Splits a string on the first occurrence of the infix operator `op` that is not inside parentheses or quotes.
/// Returns Some((left, right)) if found, otherwise None.
fn split_infix_operator(s: &str, op: char) -> Option<(String, String)> {
    let mut nest = 0;
    let mut in_quote = false;
    for (i, c) in s.char_indices() {
        match c {
            '\'' => in_quote = !in_quote,
            '(' if !in_quote => nest += 1,
            ')' if !in_quote => {
                if nest > 0 {
                    nest -= 1;
                }
            }
            c if c == op && nest == 0 && !in_quote => {
                let left = s[..i].trim().to_string();
                let right = s[i + 1..].trim().to_string();
                return Some((left, right));
            }
            _ => {}
        }
    }
    None
}

/// Splits a string on the period ('.') character that is not part of a decimal number,
/// and that occurs at the outermost level (i.e. not inside quotes or parentheses).
///
/// A period is considered part of a decimal if it is preceded and followed by a digit.
fn split_clauses(s: &str) -> Vec<String> {
    let mut clauses = Vec::new();
    let mut start = 0;
    let mut in_quote = false;
    let mut paren_count = 0;
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for i in 0..len {
        let c = chars[i];
        match c {
            '\'' => in_quote = !in_quote,
            '(' if !in_quote => paren_count += 1,
            ')' if !in_quote => {
                if paren_count > 0 {
                    paren_count -= 1;
                }
            }
            '.' if !in_quote && paren_count == 0 => {
                // Check if this period is part of a decimal.
                let is_decimal = if i > 0 && i + 1 < len {
                    chars[i - 1].is_digit(10) && chars[i + 1].is_digit(10)
                } else {
                    false
                };
                if !is_decimal {
                    let clause = s[start..i].trim();
                    if !clause.is_empty() {
                        clauses.push(clause.to_string());
                    }
                    start = i + 1;
                }
            }
            _ => {}
        }
    }
    if start < len {
        let clause = s[start..].trim();
        if !clause.is_empty() {
            clauses.push(clause.to_string());
        }
    }
    clauses
}

/// Splits the input string into clauses. This function first removes comment and query lines,
/// joins the remaining lines into one string, and then splits that string into clauses using `split_clauses`.
/// It then enforces that the program text must end with a period.
pub fn split_program_into_clauses(input: &str) -> Result<Vec<String>, ParseError> {
    let mut program = String::new();
    for line in input.lines() {
        let trimmed = line.trim();
        // Skip lines that are comments or queries.
        if trimmed.starts_with('%') || trimmed.starts_with("//") || trimmed.starts_with("?-") {
            continue;
        }
        program.push_str(trimmed);
        program.push(' ');
    }
    let program = program.trim().to_string();
    if program.is_empty() {
        return Err(ParseError::IncompleteInput);
    }
    // Enforce that the program ends with a period.
    if !program.ends_with('.') {
        return Err(ParseError::UnexpectedToken("Clause must end with a period".to_string()));
    }
    Ok(split_clauses(&program))
}

/// Parses a single term from the given input string.
///
/// Supported term types:
/// - **Single-quoted strings:** e.g. `'Hello world'` is parsed as `Atom("Hello world")`.
/// - **Variables:** A token that starts with an uppercase letter or underscore.
/// - **Numbers:** Parsed as integers. (Decimal numbers containing a period will later be supported.)
/// - **Compound terms:** A functor followed by a parenthesized, comma-separated list of arguments.
/// - **Infix operator "=":** If the input contains a top–level "=" not inside quotes or parentheses,
///   it is parsed as a compound term with functor "=" and two arguments.
/// - **Atoms:** All other tokens.
pub fn parse_term(input: &str) -> Result<Term, ParseError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(ParseError::IncompleteInput);
    }
    // If not a quoted string, check for a top–level infix "=".
    if !s.starts_with('\'') {
        if let Some((left, right)) = split_infix_operator(s, '=') {
            let left_term = parse_term(&left)?;
            let right_term = parse_term(&right)?;
            return Ok(Term::Compound("=".to_string(), vec![left_term, right_term]));
        }
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
            let parts = split_goals(args_str);
            parts.into_iter().map(|arg| parse_term(arg)).collect()
        };
        return Ok(Term::Compound(functor, args?));
    }
    // Otherwise, treat the token as an atom.
    Ok(Term::Atom(s.to_string()))
}

/// Parses an entire Prolog program from the given input string.
///
/// This parser supports multi–line clauses by first removing comment and query lines,
/// then joining the remaining lines into a single string, splitting that string into clauses
/// using the period ('.') as a delimiter (ignoring periods in decimals), and then parsing each clause.
/// 
/// Facts are clauses that do not contain ":-"; rules are clauses that contain ":-".
pub fn parse_program(input: &str) -> Result<Vec<Clause>, ParseError> {
    let clauses_str = split_program_into_clauses(input)?;

    let mut clauses = Vec::new();
    for clause in clauses_str {
        // Check if the clause contains ":-" to decide if it is a rule.
        if clause.contains(":-") {
            let parts: Vec<&str> = clause.split(":-").collect();
            if parts.len() != 2 {
                return Err(ParseError::UnexpectedToken(format!("Invalid clause syntax: {}", clause)));
            }
            let head_str = parts[0].trim();
            let body_str = parts[1].trim();
            let head = parse_term(head_str)?;
            let body_parts = split_goals(body_str);
            let mut body = Vec::new();
            for part in body_parts {
                let term = parse_term(part)?;
                body.push(term);
            }
            clauses.push(Clause::Rule { head, body });
        } else {
            let head = parse_term(&clause)?;
            clauses.push(Clause::Fact { head });
        }
    }
    Ok(clauses)
}
