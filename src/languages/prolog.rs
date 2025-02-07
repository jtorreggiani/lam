//! A refactored Prolog interpreter built on top of the LAM abstract machine.
//!
//! This interpreter supports a basic Prolog syntax including facts, rules, and queries,
//! an initialization directive (e.g. `:- initialization(main).`), and a REPL that mimics
//! SWI‑Prolog’s prompt and output.
//!
//! In file mode, facts are compiled into clauses; in the interactive REPL, any input that
//! does not begin with an assert, retract, or dynamic declaration is treated as a query.
//!
//! For example, in the REPL you can type:
//!
//!    assert(likes(john, pizza)).
//!    true.
//!
//!    likes(X, pizza).
//!    X = john .
//!
//! Notice that you do not have to type a leading `?-` (the prompt already shows `?- `).

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};

use lam::machine::core::Machine;
use lam::machine::instruction::Instruction;
use lam::machine::term::Term;

/// A Prolog term: a constant, a string, a variable (if the identifier starts with an uppercase letter),
/// or an atom.
#[derive(Debug, Clone)]
pub enum PrologTerm {
    Const(i32),
    Str(String),
    Var(String),
    Atom(String),
}

/// An atomic goal: a predicate and a (possibly empty) list of arguments.
#[derive(Debug, Clone)]
pub struct PrologGoal {
    pub predicate: String,
    pub args: Vec<PrologTerm>,
}

/// A clause (fact or rule). In a rule, the head and body are separated by ":-".
#[derive(Debug, Clone)]
pub struct PrologClause {
    pub head: PrologGoal,
    pub body: Option<Vec<PrologGoal>>, // None means fact; Some(goals) means rule.
}

/// Top-level commands.
#[derive(Debug, Clone)]
pub enum PrologCommand {
    Clause(PrologClause),
    Query(Vec<PrologGoal>),
    AssertClause(PrologClause),
    RetractClause(PrologClause),
    DynamicDeclaration(String),
}

/// A simple recursive–descent parser for a Prolog–like language.
struct PrologParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> PrologParser<'a> {
    fn new(input: &'a str) -> Self {
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

    /// Parse one command.
    fn parse_command(&mut self) -> Result<Option<PrologCommand>, String> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return Ok(None);
        }
        // Skip comment lines.
        if self.input[self.pos..].starts_with("%") || self.input[self.pos..].starts_with("//") {
            if let Some(newline_pos) = self.input[self.pos..].find('\n') {
                self.pos += newline_pos + 1;
            } else {
                self.pos = self.input.len();
            }
            return self.parse_command();
        }
        // Check for initialization, assert, retract, dynamic declaration, or query.
        if self.input[self.pos..].starts_with(":- initialization(") {
            self.pos += ":- initialization(".len();
            self.skip_whitespace();
            let goals = self.parse_goal_sequence()?;
            self.consume_expected(").")?;
            return Ok(Some(PrologCommand::Query(goals)));
        } else if self.input[self.pos..].starts_with("assert(") {
            self.pos += "assert(".len();
            let clause = self.parse_clause_in_assert()?;
            self.consume_expected(").")?;
            return Ok(Some(PrologCommand::AssertClause(clause)));
        } else if self.input[self.pos..].starts_with("retract(") {
            self.pos += "retract(".len();
            let clause = self.parse_clause()?;
            self.consume_expected(").")?;
            return Ok(Some(PrologCommand::RetractClause(clause)));
        } else if self.input[self.pos..].starts_with(":- dynamic(") {
            self.pos += ":- dynamic(".len();
            self.skip_whitespace();
            let start = self.pos;
            while let Some(ch) = self.current_char() {
                if ch == '/' { break; }
                self.pos += ch.len_utf8();
            }
            let pred = self.input[start..self.pos].trim().to_string();
            self.consume_expected(").")?;
            return Ok(Some(PrologCommand::DynamicDeclaration(pred)));
        } else if self.input[self.pos..].starts_with("?-") {
            // Strip leading "?-"
            self.pos += 2;
            self.skip_whitespace();
            let goals = self.parse_goal_sequence()?;
            self.consume_expected(".")?;
            return Ok(Some(PrologCommand::Query(goals)));
        } else {
            // In file mode, a clause is expected.
            let clause = self.parse_clause()?;
            return Ok(Some(PrologCommand::Clause(clause)));
        }
    }

    fn parse_goal_sequence(&mut self) -> Result<Vec<PrologGoal>, String> {
        let mut goals = Vec::new();
        loop {
            let goal = self.parse_goal_atom()?;
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

    /// Parse an atomic goal. If no '(' follows the identifier, assume no arguments.
    fn parse_goal_atom(&mut self) -> Result<PrologGoal, String> {
        self.skip_whitespace();
        let predicate = self.parse_identifier()?;
        self.skip_whitespace();
        let args = if self.input[self.pos..].starts_with("(") {
            self.consume_expected("(")?;
            let args = self.parse_arguments()?;
            self.consume_expected(")")?;
            args
        } else {
            Vec::new()
        };
        Ok(PrologGoal { predicate, args })
    }

    fn parse_clause(&mut self) -> Result<PrologClause, String> {
        self.skip_whitespace();
        let head = self.parse_goal_atom()?;
        self.skip_whitespace();
        let body = if self.input[self.pos..].starts_with(":-") {
            self.pos += 2;
            self.skip_whitespace();
            let goals = self.parse_goal_sequence()?;
            Some(goals)
        } else {
            None
        };
        self.consume_expected(".")?;
        Ok(PrologClause { head, body })
    }

    /// Parse a clause for an assert (no trailing period required).
    fn parse_clause_in_assert(&mut self) -> Result<PrologClause, String> {
        self.skip_whitespace();
        let head = self.parse_goal_atom()?;
        self.skip_whitespace();
        let body = if self.input[self.pos..].starts_with(":-") {
            self.pos += 2;
            self.skip_whitespace();
            let goals = self.parse_goal_sequence()?;
            Some(goals)
        } else {
            None
        };
        Ok(PrologClause { head, body })
    }

    fn parse_arguments(&mut self) -> Result<Vec<PrologTerm>, String> {
        let mut args = Vec::new();
        loop {
            self.skip_whitespace();
            let term = self.parse_term()?;
            args.push(term);
            self.skip_whitespace();
            if self.input[self.pos..].starts_with(",") {
                self.pos += 1;
            } else {
                break;
            }
        }
        Ok(args)
    }

    fn parse_term(&mut self) -> Result<PrologTerm, String> {
        self.skip_whitespace();
        if let Some(ch) = self.current_char() {
            if ch.is_digit(10) {
                self.parse_number().map(PrologTerm::Const)
            } else if ch == '"' {
                self.parse_string().map(PrologTerm::Str)
            } else if ch.is_uppercase() {
                self.parse_identifier().map(PrologTerm::Var)
            } else {
                self.parse_identifier().map(PrologTerm::Atom)
            }
        } else {
            Err("Unexpected end of input while parsing term".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<i32, String> {
        let start = self.pos;
        while let Some(ch) = self.current_char() {
            if ch.is_digit(10) {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
        self.input[start..self.pos].parse::<i32>().map_err(|e| e.to_string())
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.pos += 1; // skip opening quote
        let start = self.pos;
        while let Some(ch) = self.current_char() {
            if ch == '"' {
                let s = self.input[start..self.pos].to_string();
                self.pos += 1; // skip closing quote
                return Ok(s);
            }
            self.pos += ch.len_utf8();
        }
        Err("Unterminated string literal".to_string())
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
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

    fn consume_expected(&mut self, expected: &str) -> Result<(), String> {
        self.skip_whitespace();
        if self.input[self.pos..].starts_with(expected) {
            self.pos += expected.len();
            Ok(())
        } else {
            Err(format!("Expected '{}'", expected))
        }
    }
}

/// Compiler functions convert AST nodes into LAM instructions.

fn compile_clause(clause: &PrologClause) -> (Vec<Instruction>, String) {
    let mut instructions = Vec::new();
    let arity = clause.head.args.len();
    let mut var_map = HashMap::new();
    let mut next_var_id = 1000;
    // Compile the head with Get–instructions.
    for (i, arg) in clause.head.args.iter().enumerate() {
        match arg {
            PrologTerm::Const(n) => {
                instructions.push(Instruction::GetConst { register: i, value: *n });
            }
            PrologTerm::Str(s) => {
                instructions.push(Instruction::GetStr { register: i, value: s.clone() });
            }
            PrologTerm::Var(name) => {
                let var_id = *var_map.entry(name.clone()).or_insert_with(|| {
                    let id = next_var_id;
                    next_var_id += 1;
                    id
                });
                instructions.push(Instruction::GetVar { register: i, var_id, name: name.clone() });
            }
            PrologTerm::Atom(a) => {
                instructions.push(Instruction::GetStr { register: i, value: a.clone() });
            }
        }
    }
    instructions.push(Instruction::BuildCompound {
        target: arity,
        functor: clause.head.predicate.clone(),
        arg_registers: (0..arity).collect(),
    });
    if let Some(body_goals) = &clause.body {
        for goal in body_goals {
            let goal_instr = compile_goal(goal);
            instructions.extend(goal_instr);
        }
    }
    instructions.push(Instruction::Proceed);
    (instructions, clause.head.predicate.clone())
}

fn compile_goal(goal: &PrologGoal) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    let arity = goal.args.len();
    let mut var_map = HashMap::new();
    let mut next_var_id = 100;
    for (i, arg) in goal.args.iter().enumerate() {
        match arg {
            PrologTerm::Const(n) => {
                instructions.push(Instruction::PutConst { register: i, value: *n });
            }
            PrologTerm::Str(s) => {
                instructions.push(Instruction::PutStr { register: i, value: s.clone() });
            }
            PrologTerm::Var(name) => {
                let var_id = *var_map.entry(name.clone()).or_insert_with(|| {
                    let id = next_var_id;
                    next_var_id += 1;
                    id
                });
                instructions.push(Instruction::PutVar { register: i, var_id, name: name.clone() });
            }
            PrologTerm::Atom(a) => {
                instructions.push(Instruction::PutStr { register: i, value: a.clone() });
            }
        }
    }
    instructions.push(Instruction::BuildCompound {
        target: arity,
        functor: goal.predicate.clone(),
        arg_registers: (0..arity).collect(),
    });
    instructions.push(Instruction::Call { predicate: goal.predicate.clone() });
    instructions.push(Instruction::Proceed);
    instructions
}

/// Compile a query into LAM instructions and return a mapping from query variable IDs to names.
fn compile_query(goals: &[PrologGoal]) -> (Vec<Instruction>, HashMap<usize, String>) {
    let mut instructions = Vec::new();
    let mut var_map = HashMap::new();
    let mut query_var_names = HashMap::new();
    let mut next_var_id = 100;
    for goal in goals {
        let arity = goal.args.len();
        for (i, arg) in goal.args.iter().enumerate() {
            match arg {
                PrologTerm::Const(n) => {
                    instructions.push(Instruction::PutConst { register: i, value: *n });
                }
                PrologTerm::Str(s) => {
                    instructions.push(Instruction::PutStr { register: i, value: s.clone() });
                }
                PrologTerm::Var(name) => {
                    let var_id = *var_map.entry(name.clone()).or_insert_with(|| {
                        let id = next_var_id;
                        next_var_id += 1;
                        id
                    });
                    query_var_names.insert(var_id, name.clone());
                    instructions.push(Instruction::PutVar { register: i, var_id, name: name.clone() });
                }
                PrologTerm::Atom(a) => {
                    instructions.push(Instruction::PutStr { register: i, value: a.clone() });
                }
            }
        }
        instructions.push(Instruction::BuildCompound {
            target: arity,
            functor: goal.predicate.clone(),
            arg_registers: (0..arity).collect(),
        });
        instructions.push(Instruction::Call { predicate: goal.predicate.clone() });
        instructions.push(Instruction::Proceed);
    }
    instructions.push(Instruction::Halt);
    (instructions, query_var_names)
}

/// Run a Prolog program from a file.
pub fn run_prolog_program(source: &str) {
    let mut parser = PrologParser::new(source);
    let mut commands = Vec::new();
    while let Ok(Some(cmd)) = parser.parse_command() {
        commands.push(cmd);
    }
    let mut db_code = Vec::new();
    let mut predicate_table: HashMap<String, Vec<usize>> = HashMap::new();
    let mut query_code: Option<(Vec<Instruction>, HashMap<usize, String>)> = None;
    for cmd in commands {
        match cmd {
            PrologCommand::Query(goals) => {
                query_code = Some(compile_query(&goals));
            }
            PrologCommand::Clause(clause) => {
                let (code, pred) = compile_clause(&clause);
                let addr = db_code.len();
                db_code.extend(code);
                predicate_table.entry(pred).or_insert_with(Vec::new).push(addr);
            }
            PrologCommand::AssertClause(clause) => {
                let (code, pred) = compile_clause(&clause);
                let addr = db_code.len();
                db_code.extend(code);
                predicate_table.entry(pred).or_insert_with(Vec::new).push(addr);
                println!("true.");
            }
            PrologCommand::RetractClause(_clause) => {
                println!("Retract not yet implemented in the interpreter.");
            }
            PrologCommand::DynamicDeclaration(pred) => {
                println!("Dynamic declaration for predicate '{}' noted.", pred);
            }
        }
    }
    if let Some((query_instr, query_var_names)) = query_code {
        let query_start = db_code.len();
        let mut full_code = db_code;
        full_code.extend(query_instr);
        let mut machine = Machine::new(100, full_code);
        machine.pc = query_start;
        for (pred, addrs) in predicate_table {
            for addr in addrs {
                machine.register_predicate(pred.clone(), addr);
            }
        }
        machine.variable_names = query_var_names;
        machine.verbose = false;
        match machine.run() {
            Ok(_) => {
                if machine.variable_names.is_empty() {
                    println!("true.");
                } else {
                    for (var_id, name) in machine.variable_names.iter() {
                        let binding = machine.uf.resolve(&Term::Var(*var_id));
                        match binding {
                            Term::Str(ref s) => println!("{} = {} .", name, s),
                            _ => println!("{} = {} .", name, binding),
                        }
                    }
                }
            }
            Err(_) => {
                println!("false.");
            }
        }
    }
}

/// Run an interactive REPL.
pub fn run_repl() {
    let mut db_code = Vec::new();
    let mut predicate_table: HashMap<String, Vec<usize>> = HashMap::new();

    println!("Welcome to the refactored LAM Prolog REPL.");
    println!("Enter queries (e.g., parent(\"alice\", X).) or commands such as:");
    println!("  assert(<clause>).    to add a clause");
    println!("  retract(<clause>).   to remove a clause");
    println!("  :- dynamic(<pred>/<arity>).   to declare a predicate dynamic.\n");

    loop {
        print!("?- ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }
        let trimmed = input.trim();
        if trimmed.eq_ignore_ascii_case("halt.") || trimmed.eq_ignore_ascii_case("quit.") {
            break;
        }
        // In the REPL we do not require a leading "?-". Any input not starting with a known command
        // is treated as a query.
        let query_input = if trimmed.starts_with("assert(")
            || trimmed.starts_with("retract(")
            || trimmed.starts_with(":- dynamic(")
        {
            trimmed.to_string()
        } else {
            // If the user types a leading "?-", remove it.
            if trimmed.starts_with("?-") {
                trimmed.trim_start_matches("?-").trim().to_string()
            } else {
                trimmed.to_string()
            }
        };

        let mut parser = PrologParser::new(&query_input);
        match parser.parse_command() {
            Ok(Some(cmd)) => {
                // In the REPL, if the input was a fact (Clause) rather than a Query,
                // convert it to a Query so that a query is executed.
                let cmd = match cmd {
                    PrologCommand::Clause(cl) => PrologCommand::Query(vec![cl.head]),
                    other => other,
                };
                match cmd {
                    PrologCommand::Query(goals) => {
                        let (query_instr, query_var_names) = compile_query(&goals);
                        let query_start = db_code.len();
                        let mut full_code = db_code.clone();
                        full_code.extend(query_instr);
                        let mut machine = Machine::new(100, full_code);
                        machine.pc = query_start;
                        for (pred, addrs) in &predicate_table {
                            for &addr in addrs {
                                machine.register_predicate(pred.clone(), addr);
                            }
                        }
                        machine.variable_names = query_var_names;
                        machine.verbose = false;
                        match machine.run() {
                            Ok(_) => {
                                if machine.variable_names.is_empty() {
                                    println!("true.");
                                } else {
                                    for (var_id, name) in machine.variable_names.iter() {
                                        let binding = machine.uf.resolve(&Term::Var(*var_id));
                                        match binding {
                                            Term::Str(ref s) => println!("{} = {} .", name, s),
                                            _ => println!("{} = {} .", name, binding),
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                println!("false.");
                            }
                        }
                    }
                    PrologCommand::AssertClause(clause) => {
                        let (code, pred) = compile_clause(&clause);
                        let addr = db_code.len();
                        db_code.extend(code);
                        predicate_table.entry(pred).or_insert_with(Vec::new).push(addr);
                        println!("true.");
                    }
                    PrologCommand::RetractClause(_clause) => {
                        println!("Retract not yet implemented.");
                    }
                    PrologCommand::DynamicDeclaration(pred) => {
                        println!("Dynamic declaration noted for predicate '{}'.", pred);
                    }
                    _ => { }
                }
            }
            Ok(None) => { }
            Err(e) => {
                println!("Parse error: {}", e);
            }
        }
    }
    println!("Goodbye.");
}

/// Entry point. If a filename is provided as a command-line argument, run in file mode;
/// otherwise, start the REPL.
pub fn start() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let filename = &args[1];
        let content = fs::read_to_string(filename)
            .unwrap_or_else(|_| panic!("Failed to read file: {}", filename));
        run_prolog_program(&content);
    } else {
        run_repl();
    }
}

fn main() {
    env_logger::init();
    start();
}
