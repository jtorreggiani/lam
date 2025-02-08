use crate::prolog::ast::{Clause, Term};

/// Represents errors that can occur during parsing.
#[derive(Debug)]
pub enum ParseError {
    /// A required token (such as the terminating period) was missing.
    UnexpectedToken(String),
    /// The input was empty or incomplete.
    IncompleteInput,
}

/// Parses a complete Prolog program (a series of clauses).
///
/// Each clause must be on its own line and end with a period.
/// Comments (lines starting with "%" or "//") and blank lines are ignored.
///
/// # Examples
///
/// ```prolog
/// parent(john, mary).
/// ancestor(X, Y) :- parent(X, Y).
/// ```
pub fn parse_program(input: &str) -> Result<Vec<Clause>, ParseError> {
    let mut clauses = Vec::new();

    for line in input.lines() {
        let line = line.trim();
        // Skip empty lines and comment lines.
        if line.is_empty() || line.starts_with('%') || line.starts_with("//") {
            continue;
        }
        // Each clause must end with a period.
        if !line.ends_with('.') {
            return Err(ParseError::UnexpectedToken(
                "Clause must end with a period.".into(),
            ));
        }
        // Remove the trailing period.
        let clause_str = line[..line.len() - 1].trim();
        if clause_str.is_empty() {
            continue;
        }
        // Determine if the clause is a rule or a fact.
        if let Some(index) = clause_str.find(":-") {
            // Rule clause.
            let head_str = clause_str[..index].trim();
            let body_str = clause_str[index + 2..].trim();
            let head = parse_term(head_str)?;
            // Instead of simply splitting on commas, we use a helper that only splits
            // on commas not enclosed in parentheses.
            let body_parts = split_goals(body_str);
            let body: Result<Vec<_>, _> = body_parts
                .into_iter()
                .map(|s| parse_term(s.trim()))
                .collect();
            clauses.push(Clause::Rule {
                head,
                body: body?,
            });
        } else {
            // Fact clause.
            let head = parse_term(clause_str)?;
            clauses.push(Clause::Fact { head });
        }
    }

    Ok(clauses)
}

/// Splits a rule body string into separate goal substrings by splitting on commas
/// that are not enclosed within parentheses.
fn split_goals(body: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut paren_count = 0;

    for (i, ch) in body.char_indices() {
        match ch {
            '(' => paren_count += 1,
            ')' => {
                if paren_count > 0 {
                    paren_count -= 1;
                }
            }
            ',' if paren_count == 0 => {
                // Found a comma outside of any parentheses.
                parts.push(&body[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    // Push the final segment.
    if start < body.len() {
        parts.push(&body[start..]);
    }
    parts.into_iter().filter(|s| !s.trim().is_empty()).collect()
}

/// Parses a single Prolog term.
///
/// This minimal parser supports:
/// - **Variables:** tokens that start with an uppercase letter or an underscore.
/// - **Numbers:** integer literals.
/// - **Atoms:** tokens that start with a lowercase letter.
/// - **Compound terms:** functor followed by a parenthesized, comma‑separated list of arguments.
///
/// # Examples
///
/// - `"X"` → Variable `"X"`  
/// - `"42"` → Number `42`  
/// - `"john"` → Atom `"john"`  
/// - `"parent(john, mary)"` → Compound term with functor `"parent"` and arguments `[Atom("john"), Atom("mary")]`
pub fn parse_term(input: &str) -> Result<Term, ParseError> {
    if input.is_empty() {
        return Err(ParseError::IncompleteInput);
    }
    let first_char = input.chars().next().unwrap();
    if first_char.is_uppercase() || first_char == '_' {
        // Variable.
        Ok(Term::Var(input.to_string()))
    } else if first_char.is_digit(10) {
        // Parse an integer.
        input
            .parse::<i32>()
            .map(Term::Number)
            .map_err(|_| ParseError::UnexpectedToken(format!("Invalid number: {}", input)))
    } else {
        // Atom or compound term.
        if let Some(pos) = input.find('(') {
            let functor = input[..pos].trim().to_string();
            let args_str = input[pos + 1..].trim_end_matches(')').trim();
            let args = if args_str.is_empty() {
                Vec::new()
            } else {
                args_str
                    .split(',')
                    .map(|s| parse_term(s.trim()))
                    .collect::<Result<Vec<_>, _>>()?
            };
            Ok(Term::Compound(functor, args))
        } else {
            // Plain atom.
            Ok(Term::Atom(input.to_string()))
        }
    }
}
