// ============================================
// File: src/languages/prolog/parser.rs
// ============================================

// IMPORTANT: We add an import here so that the AST types are in scope.
use crate::languages::prolog::ast::{PrologClause, PrologGoal, PrologTerm};

/// A simple recursive–descent parser for our Prolog–like language.
pub struct PrologParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> PrologParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
    }

    fn current_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// Parse an identifier: a sequence of alphanumeric characters and underscores.
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

    /// Parse a double-quoted string.
    pub fn parse_string(&mut self) -> Result<String, String> {
        self.pos += 1; // skip opening "
        let start = self.pos;
        while let Some(ch) = self.current_char() {
            if ch == '"' {
                let s = self.input[start..self.pos].to_string();
                self.pos += 1; // skip closing "
                return Ok(s);
            }
            self.pos += ch.len_utf8();
        }
        Err("Unterminated string literal".to_string())
    }

    /// Parse a single-quoted atom.
    pub fn parse_quoted_atom(&mut self) -> Result<String, String> {
        self.pos += 1; // skip opening '
        let start = self.pos;
        while let Some(ch) = self.current_char() {
            if ch == '\'' {
                let atom = self.input[start..self.pos].to_string();
                self.pos += 1; // skip closing '
                return Ok(atom);
            }
            self.pos += ch.len_utf8();
        }
        Err("Unterminated quoted atom".to_string())
    }

    /// Parse a numeric constant.
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

    /// Parse a Prolog term.
    pub fn parse_term(&mut self) -> Result<PrologTerm, String> {
        self.skip_whitespace();
        if let Some(ch) = self.current_char() {
            if ch.is_digit(10) {
                self.parse_number().map(PrologTerm::Const)
            } else if ch == '"' {
                self.parse_string().map(PrologTerm::Str)
            } else if ch == '\'' {
                self.parse_quoted_atom().map(PrologTerm::Atom)
            } else if ch.is_uppercase() {
                self.parse_identifier().map(PrologTerm::Var)
            } else {
                self.parse_identifier().map(PrologTerm::Atom)
            }
        } else {
            Err("Unexpected end of input while parsing term".to_string())
        }
    }

    /// Parse a comma–separated list of terms.
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

    /// Parse a goal: predicate followed by an optional parenthesized list of arguments.
    pub fn parse_goal(&mut self) -> Result<PrologGoal, String> {
        let predicate = self.parse_identifier()?;
        self.skip_whitespace();
        let args = if self.input[self.pos..].starts_with("(") {
            self.pos += 1; // skip '('
            let args = self.parse_term_list()?;
            self.consume_expected(")")?;
            args
        } else {
            Vec::new()
        };
        Ok(PrologGoal { predicate, args })
    }

    /// Consume an expected string.
    pub fn consume_expected(&mut self, expected: &str) -> Result<(), String> {
        self.skip_whitespace();
        if self.input[self.pos..].starts_with(expected) {
            self.pos += expected.len();
            Ok(())
        } else {
            Err(format!("Expected '{}'", expected))
        }
    }

    /// Parse a fact (a clause with no body). The fact must end with a period.
    pub fn parse_fact(&mut self) -> Result<PrologClause, String> {
        let goal = self.parse_goal()?;
        self.consume_expected(".")?;
        Ok(PrologClause { head: goal, body: None })
    }

    /// Parse a query. (For our initial version we assume a query is simply a goal that ends with a period.)
    pub fn parse_query(&mut self) -> Result<Vec<PrologGoal>, String> {
        let goal = self.parse_goal()?;
        self.consume_expected(".")?;
        Ok(vec![goal])
    }
}
