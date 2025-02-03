use lam::machine::Machine;
use lam::term::Term;
use std::time::Instant;

#[test]
fn benchmark_unify_identical_terms() {
    // Create a compound term with many arguments.
    // A vector with 10,000 identical constants.
    let args = vec![Term::Const(42); 10_000];
    let term = Term::Compound("f".to_string(), args);

    let mut machine = Machine::new(1, vec![]);

    // We'll perform many unification calls.
    let iterations = 10_000;
    let start = Instant::now();
    for _ in 0..iterations {
        // Since both terms are identical, the early exit should trigger.
        assert!(machine.unify(&term, &term).is_ok());
    }
    let duration = start.elapsed();
    println!(
        "Time taken for {} unifications: {:?}",
        iterations, duration
    );
}
