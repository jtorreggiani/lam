// src/machine/core.rs
//! Core implementation of the LAM abstract machine.

use std::collections::HashMap;
use log::debug;

use crate::machine::choice_point::ChoicePoint;
use crate::machine::error_handling::MachineError;
use crate::machine::frame::Frame;
use crate::machine::instruction::Instruction;
use crate::machine::term::Term;
use crate::machine::unification::UnionFind;

/// The built–in predicate function type.
pub type BuiltinPredicate = fn(&mut Machine) -> Result<(), MachineError>;

/// The LAM abstract machine.
#[derive(Debug)]
pub struct Machine {
    /// Registers.
    pub registers: Vec<Option<Term>>,
    /// Program (list of instructions).
    pub code: Vec<Instruction>,
    /// Program counter.
    pub pc: usize,
    /// Current substitution.
    pub substitution: HashMap<usize, Term>,
    /// Control stack for call/return.
    pub control_stack: Vec<Frame>,
    /// Predicate table mapping names to clause addresses.
    pub predicate_table: HashMap<String, Vec<usize>>,
    /// Choice stack for backtracking (each choice point is boxed).
    pub choice_stack: Vec<Box<ChoicePoint>>,
    /// Environment stack.
    pub environment_stack: Vec<Vec<Option<Term>>>,
    /// Index table for clause indexing.
    pub index_table: HashMap<String, HashMap<Vec<Term>, Vec<usize>>>,
    /// Mapping from variable IDs to names.
    pub variable_names: HashMap<usize, String>,
    /// Union–find structure for unification.
    pub uf: UnionFind,
    /// If true, the machine will trace execution.
    pub verbose: bool,
    /// Built–in predicates.
    pub builtins: HashMap<String, BuiltinPredicate>,
}

impl Machine {
    /// Creates a new machine with the specified number of registers and program code.
    pub fn new(num_registers: usize, code: Vec<Instruction>) -> Self {
        let mut machine = Self {
            registers: vec![None; num_registers],
            code,
            pc: 0,
            substitution: HashMap::new(),
            control_stack: Vec::new(),
            predicate_table: HashMap::new(),
            choice_stack: Vec::new(),
            environment_stack: Vec::new(),
            index_table: HashMap::new(),
            variable_names: HashMap::new(),
            uf: UnionFind::new(),
            verbose: false,
            builtins: HashMap::new(),
        };
        // Register example built–in predicates.
        machine.builtins.insert("print".to_string(), Machine::builtin_print);
        machine.builtins.insert("print_subst".to_string(), Machine::builtin_print_subst);
        machine.builtins.insert("write".to_string(), Machine::builtin_write);
        machine.builtins.insert("nl".to_string(), Machine::builtin_nl);
        machine.builtins.insert("halt".to_string(), Machine::builtin_halt);
        // Register the equality built-in for unification.
        machine.builtins.insert("=".to_string(), Machine::builtin_eq);
        machine
    }

    /// Logs the execution of an instruction if verbose mode is enabled.
    pub fn trace(&self, instr: &Instruction) {
        if self.verbose {
            debug!("PC {}: Executing {:?}", self.pc - 1, instr);
            debug!("Registers: {:?}", self.registers);
            debug!("Substitution: {:?}", self.substitution);
        }
    }

    /// Updates the index table upon clause retraction.
    pub fn update_index_table_on_retract(&mut self, predicate: &str, clause_address: usize) {
        if let Some(index_map) = self.index_table.get_mut(predicate) {
            for (_key, clauses) in index_map.iter_mut() {
                if let Some(pos) = clauses.iter().position(|&a| a == clause_address) {
                    clauses.remove(pos);
                }
            }
        }
    }

    // Executes a move: copies the term from register src into register dst.
    pub fn execute_move(&mut self, src: usize, dst: usize) -> Result<(), MachineError> {
        if src >= self.registers.len() {
            return Err(MachineError::RegisterOutOfBounds(src));
        }
        if dst >= self.registers.len() {
            return Err(MachineError::RegisterOutOfBounds(dst));
        }
        let term = self.registers[src].clone();
        self.registers[dst] = term;
        Ok(())
    }

    /// Registers an indexed clause.
    pub fn register_indexed_clause(&mut self, predicate: String, key: Vec<Term>, address: usize) {
        let entry = self.index_table.entry(predicate).or_insert_with(HashMap::new);
        entry.entry(key).or_insert_with(Vec::new).push(address);
    }

    /// Registers a clause for a given predicate.
    pub fn register_predicate(&mut self, name: String, address: usize) {
        self.predicate_table.entry(name).or_insert_with(Vec::new).push(address);
    }

    /// Unifies two terms.
    pub fn unify(&mut self, t1: &Term, t2: &Term) -> Result<(), MachineError> {
        debug!("Attempting to unify {:?} with {:?}", t1, t2);
        let resolved1 = self.uf.resolve(t1);
        let resolved2 = self.uf.resolve(t2);
    
        match (&resolved1, &resolved2) {
            (&Term::Const(a), &Term::Const(b)) => {
                if a == b {
                    debug!("Constants matched: {} == {}", a, b);
                    Ok(())
                } else {
                    Err(MachineError::UnificationFailed(format!("Constants do not match: {} vs {}", a, b)))
                }
            },
            (&Term::Str(ref s1), &Term::Str(ref s2)) => {
                if s1 == s2 {
                    debug!("String constants matched: {} == {}", s1, s2);
                    Ok(())
                } else {
                    Err(MachineError::UnificationFailed(format!("String constants do not match: {} vs {}", s1, s2)))
                }
            },
            (&Term::Var(v), other) => {
                debug!("Binding variable {} to {:?}", v, other);
                self.uf.bind(v, other)
            },
            (other, &Term::Var(v)) => {
                debug!("Binding variable {} to {:?}", v, other);
                self.uf.bind(v, other)
            },
            (&Term::Compound(ref f1, ref args1), &Term::Compound(ref f2, ref args2)) => {
                if f1 != f2 || args1.len() != args2.len() {
                    return Err(MachineError::UnificationFailed(format!("Compound term mismatch: {} vs {}", f1, f2)));
                }
                for (a, b) in args1.iter().zip(args2.iter()) {
                    self.unify(a, b)?;
                }
                Ok(())
            },
            (t1, t2) => Err(MachineError::UnificationFailed(format!("Failed to unify {:?} with {:?}", t1, t2))),
        }
    }

    /// Built–in predicate: halt.
    /// When called, it stops execution by setting the program counter
    /// to the end of the code.
    pub fn builtin_halt(&mut self) -> Result<(), MachineError> {
        self.pc = self.code.len();
        Ok(())
    }

    /// Built–in predicate: prints the machine registers.
    pub fn builtin_print(&mut self) -> Result<(), MachineError> {
        println!("--- Machine Registers ---");
        for (i, reg) in self.registers.iter().enumerate() {
            if let Some(term) = reg {
                match term {
                    Term::Var(id) => {
                        let name = self.variable_names.get(id).cloned().unwrap_or_default();
                        println!("Reg {:>3}: Var({}) \"{}\"", i, id, name);
                    }
                    _ => println!("Reg {:>3}: {:?}", i, term),
                }
            }
        }
        println!("-------------------------");
        Ok(())
    }

    /// Built–in predicate: prints the current substitution.
    pub fn builtin_print_subst(&mut self) -> Result<(), MachineError> {
        println!("--- Current Substitution ---");
        if self.substitution.is_empty() {
            println!("(no bindings)");
        } else {
            for (var_id, term) in &self.substitution {
                let var_name = self.variable_names.get(var_id).cloned().unwrap_or_default();
                println!("Variable {} (id {}) = {:?}", 
                    if var_name.is_empty() { format!("_{}", var_id) } else { var_name },
                    var_id,
                    term);
            }
        }
        println!("----------------------------");
        Ok(())
    }

    /// Built–in predicate: writes a term from register 0.
    /// **Changed:** If the term is a compound with functor "-" and two arguments,
    /// print it in the format "arg0-arg1".
    pub fn builtin_write(&mut self) -> Result<(), MachineError> {
        if let Some(Some(term)) = self.registers.get(0) {
            // Resolve the term to see if it has been bound to another term.
            let resolved = self.uf.resolve(term);
            match resolved {
                Term::Compound(ref functor, ref args) if functor == "-" && args.len() == 2 => {
                    print!("{}-{}", args[0], args[1]);
                },
                _ => {
                    print!("{}", resolved);
                }
            }
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        Ok(())
    }

    /// Built–in predicate: outputs a newline.
    pub fn builtin_nl(&mut self) -> Result<(), MachineError> {
        println!();
        Ok(())
    }

    /// Built–in predicate for equality (unification).
    /// Assumes that the left argument is in register 0 and the right argument is in register 1.
    pub fn builtin_eq(&mut self) -> Result<(), MachineError> {
        let term1 = self.registers.get(0)
            .and_then(|opt| opt.clone())
            .ok_or(MachineError::UninitializedRegister(0))?;
        let term2 = self.registers.get(1)
            .and_then(|opt| opt.clone())
            .ok_or(MachineError::UninitializedRegister(1))?;
        self.unify(&term1, &term2)
    }

    /// Executes one instruction.
    pub fn step(&mut self) -> Result<(), MachineError> {
        let instr = self.code.get(self.pc)
            .ok_or(MachineError::NoMoreInstructions)?
            .clone();
        self.pc += 1;
        self.trace(&instr);
        instr.execute(self)
    }

    /// Runs the machine until a Halt instruction or the end of the code.
    /// 
    /// **Changed:** If a unification failure occurs and there is no choice point available,
    /// return the unification failure error directly rather than trying to backtrack.
    pub fn run(&mut self) -> Result<(), MachineError> {
        while self.pc < self.code.len() {
            if let Some(Instruction::Halt) = self.code.get(self.pc) {
                debug!("Halt: Stopping execution");
                break;
            }
            match self.step() {
                Ok(()) => {},
                Err(MachineError::UnificationFailed(msg)) => {
                    if self.choice_stack.is_empty() {
                        return Err(MachineError::UnificationFailed(msg));
                    } else {
                        self.execute_fail()?;
                    }
                },
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

/// Simple ping function.
pub fn ping() -> &'static str {
    "pong"
}
