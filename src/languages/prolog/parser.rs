// src/languages/prolog/parser.rs

use crate::languages::prolog::ast::{PrologClause, PrologGoal, PrologTerm};

/// A simple recursive–descent parser for a Prolog–like language.
pub struct PrologParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> PrologParser<'a> {
    /// Creates a new parser over the given input.
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    /// Skips any whitespace.
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
    }

    /// Returns the current character (if any).
    fn current_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// Parses an identifier: a sequence of alphanumeric characters and underscores.
    pub fn parse_identifier(&mut self) -> Result<String, String> {
        self.skip_whitespace();
        let start = self.pos;
        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
        if start == self.pos {
            Err("Expected identifier".to_string())
        } else {
            Ok(self.input[start..self.pos].to_string())
        }
    }

    /// Parses a double–quoted string.
    pub fn parse_string(&mut self) -> Result<String, String> {
        self.pos += 1; // Skip opening "
        let start = self.pos;
        while let Some(ch) = self.current_char() {
            if ch == '"' {
                let s = self.input[start..self.pos].to_string();
                self.pos += 1; // Skip closing "
                return Ok(s);
            }
            self.pos += ch.len_utf8();
        }
        Err("Unterminated string literal".to_string())
    }

    /// Parses a numeric constant.
    pub fn parse_number(&mut self) -> Result<i32, String> {
        self.skip_whitespace();
        let start = self.pos;
        while let Some(ch) = self.current_char() {
            if ch.is_digit(10) {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
        let num_str = &self.input[start..self.pos];
        num_str.parse::<i32>().map_err(|e| e.to_string())
    }

    /// Parses a Prolog term.
    pub fn parse_term(&mut self) -> Result<PrologTerm, String> {
        self.skip_whitespace();
        let term = if let Some(ch) = self.current_char() {
            if ch.is_digit(10) {
                self.parse_number().map(PrologTerm::Const)?
            } else if ch == '"' {
                self.parse_string().map(PrologTerm::Str)?
            } else if ch == '\'' {
                self.pos += 1; // Skip opening '
                let start = self.pos;
                while let Some(ch) = self.current_char() {
                    if ch == '\'' {
                        let atom = self.input[start..self.pos].to_string();
                        self.pos += 1; // Skip closing '
                        return Ok(PrologTerm::Atom(atom));
                    }
                    self.pos += ch.len_utf8();
                }
                return Err("Unterminated quoted atom".to_string());
            } else if ch.is_uppercase() {
                self.parse_identifier().map(PrologTerm::Var)?
            } else {
                self.parse_identifier().map(PrologTerm::Atom)?
            }
        } else {
            return Err("Unexpected end of input while parsing term".to_string());
        };

        self.skip_whitespace();
        // Support a simple infix '-' operator.
        if self.input[self.pos..].starts_with("-") {
            self.pos += 1;
            let right = self.parse_term()?;
            Ok(PrologTerm::Compound("-".to_string(), vec![term, right]))
        } else {
            Ok(term)
        }
    }

    /// Parses a comma–separated list of terms.
    pub fn parse_term_list(&mut self) -> Result<Vec<PrologTerm>, String> {
        let mut terms = Vec::new();
        loop {
            let term = self.parse_term()?;
            terms.push(term);
            self.skip_whitespace();
            if self.input[self.pos..].starts_with(",") {
                self.pos += 1;
                self.skip_whitespace();
            } else {
                break;
            }
        }
        Ok(terms)
    }

    /// Parses a goal: a predicate name optionally followed by a parenthesized list of arguments.
    pub fn parse_goal(&mut self) -> Result<PrologGoal, String> {
        let predicate = self.parse_identifier()?;
        self.skip_whitespace();
        let args = if self.input[self.pos..].starts_with("(") {
            self.pos += 1; // Skip '('
            let args = self.parse_term_list()?;
            self.skip_whitespace();
            if !self.input[self.pos..].starts_with(")") {
                return Err("Expected ')'".to_string());
            }
            self.pos += 1; // Skip ')'
            args
        } else {
            Vec::new()
        };
        Ok(PrologGoal { predicate, args })
    }

    /// Consumes the expected string.
    pub fn consume_expected(&mut self, expected: &str) -> Result<(), String> {
        self.skip_whitespace();
        if self.input[self.pos..].starts_with(expected) {
            self.pos += expected.len();
            Ok(())
        } else {
            Err(format!("Expected '{}'", expected))
        }
    }

    /// Parses a clause.
    ///
    /// A clause is either:
    /// - A directive (starting with ":-"), compiled as a clause with head "directive"
    /// - A fact or rule: a head goal optionally followed by ":-" and a body.
    pub fn parse_clause(&mut self) -> Result<PrologClause, String> {
        self.skip_whitespace();
        if self.input[self.pos..].starts_with(":-") {
            self.pos += 2; // Skip ":-"
            let body = self.parse_goal_list()?;
            self.consume_expected(".")?;
            let head = PrologGoal { predicate: "directive".to_string(), args: Vec::new() };
            Ok(PrologClause { head, body: Some(body) })
        } else {
            let head = self.parse_goal()?;
            self.skip_whitespace();
            if self.input[self.pos..].starts_with(":-") {
                self.pos += 2; // Skip ":-"
                let body = self.parse_goal_list()?;
                self.consume_expected(".")?;
                Ok(PrologClause { head, body: Some(body) })
            } else {
                self.consume_expected(".")?;
                Ok(PrologClause { head, body: None })
            }
        }
    }

    /// Parses a comma–separated list of goals.
    pub fn parse_goal_list(&mut self) -> Result<Vec<PrologGoal>, String> {
        let mut goals = Vec::new();
        loop {
            self.skip_whitespace();
            if self.input[self.pos..].starts_with(".") {
                break;
            }
            let goal = self.parse_goal()?;
            goals.push(goal);
            self.skip_whitespace();
            if self.input[self.pos..].starts_with(",") {
                self.pos += 1;
                self.skip_whitespace();
            } else {
                break;
            }
        }
        Ok(goals)
    }

    /// Parses an entire Prolog program.
    pub fn parse_program(&mut self) -> Result<Vec<PrologClause>, String> {
        let mut clauses = Vec::new();
        while self.pos < self.input.len() {
            self.skip_whitespace();
            if self.pos >= self.input.len() {
                break;
            }
            if self.input[self.pos..].starts_with("%") {
                // Skip a comment line.
                while let Some(ch) = self.current_char() {
                    self.pos += ch.len_utf8();
                    if ch == '\n' { break; }
                }
                continue;
            }
            let clause = self.parse_clause()?;
            clauses.push(clause);
        }
        Ok(clauses)
    }

    /// Parses a fact (a clause with no body).
    pub fn parse_fact(&mut self) -> Result<PrologClause, String> {
        let clause = self.parse_clause()?;
        if clause.body.is_some() {
            Err("Expected fact, but found a rule".to_string())
        } else {
            Ok(clause)
        }
    }

    /// Parses a query: a comma–separated list of goals terminated by a period.
    pub fn parse_query(&mut self) -> Result<Vec<PrologGoal>, String> {
        let goals = self.parse_goal_list()?;
        self.consume_expected(".")?;
        Ok(goals)
    }
}
