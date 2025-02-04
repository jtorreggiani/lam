// benches/machine_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lam::machine::{Instruction, Machine}; // Adjust the module path as needed.
use lam::term::Term;

/// Builds a sample program for the machine.
/// This simple program puts a constant, performs a unification check, and proceeds.
fn build_sample_program() -> Vec<Instruction> {
    vec![
        // Put the constant 42 in register 0.
        Instruction::PutConst { register: 0, value: 42 },
        // Put a variable in register 1.
        Instruction::PutVar { register: 1, var_id: 1, name: "X".to_string() },
        // Try to unify register 0 with constant 42.
        Instruction::GetConst { register: 0, value: 42 },
        // Proceed (simulate predicate return).
        Instruction::Proceed,
    ]
}

/// This benchmark initializes the machine with the sample program and runs it.
fn benchmark_machine_run(c: &mut Criterion) {
    // Define the number of registers needed (here 2 registers).
    let num_registers = 2;
    // Build the machine code (program)
    let code = build_sample_program();

    c.bench_function("machine_run", |b| {
        b.iter(|| {
            // Create a new machine instance for every iteration to ensure isolation.
            let mut machine = Machine::new(num_registers, code.clone());
            // Run the machine; black_box prevents the compiler from optimizing away the call.
            let result = machine.run();
            // Verify that the run completed successfully.
            black_box(result.expect("Machine run should succeed"));
        })
    });
}

fn benchmark_unification(c: &mut Criterion) {
  // Create two terms for unification.
  let term1 = Term::Compound("f".to_string(), vec![Term::Const(1), Term::Var(1)]);
  let term2 = Term::Compound("f".to_string(), vec![Term::Const(1), Term::Const(2)]);

  // Setup a machine (or just the union-find if that’s separable) to call unify.
  let mut machine = Machine::new(2, vec![]); // dummy code; we're directly calling unify

  c.bench_function("unification", |b| {
      b.iter(|| {
          // We call unify; note that if unification changes machine state,
          // you might need to reset the machine for each iteration.
          let result = machine.unify(&term1, &term2);
          let _ = black_box(result);
      })
  });
}

criterion_group!(benches, benchmark_machine_run, benchmark_unification);
criterion_main!(benches);

