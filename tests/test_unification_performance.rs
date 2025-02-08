#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::term::Term;
    use std::time::Instant;

    #[test]
    fn benchmark_unify_identical_terms() {
        // Create a compound term with 10,000 identical constants.
        let args = vec![Term::Const(42); 10_000];
        let term = Term::Compound("f".to_string(), args);

        let mut machine = Machine::new(1, vec![]);

        let iterations = 10_000;
        let start = Instant::now();
        for _ in 0..iterations {
            machine.unify(&term, &term).expect("Unification should succeed");
        }
        let duration = start.elapsed();
        println!("Time taken for {} unifications: {:?}", iterations, duration);
    }
}
