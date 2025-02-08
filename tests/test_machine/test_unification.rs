#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use lam::machine::unification::{UnionFind, TrailEntry};
    use lam::machine::term::Term;
    use lam::machine::error_handling::MachineError;

    #[test]
    fn test_new_union_find() {
        let uf = UnionFind::new();
        assert!(uf.bindings.is_empty(), "Bindings should be empty on creation");
        assert!(uf.trail.is_empty(), "Trail should be empty on creation");
    }

    #[test]
    fn test_bind_constant() {
        let mut uf = UnionFind::new();
        // Bind variable 1 to constant 42.
        uf.bind(1, &Term::Const(42)).unwrap();
        let res = uf.resolve(&Term::Var(1));
        assert_eq!(res, Term::Const(42));
    }

    #[test]
    fn test_bind_variable_to_variable() {
        let mut uf = UnionFind::new();
        // Bind variable 1 to variable 2.
        uf.bind(1, &Term::Var(2)).unwrap();
        // Then bind variable 2 to constant 99.
        uf.bind(2, &Term::Const(99)).unwrap();
        // Resolving Var(1) should yield Const(99) due to the chain.
        let res = uf.resolve(&Term::Var(1));
        assert_eq!(res, Term::Const(99));
    }

    #[test]
    fn test_bind_self_no_change() {
        let mut uf = UnionFind::new();
        // Binding a variable to itself should not change the union-find.
        uf.bind(1, &Term::Var(1)).unwrap();
        let res = uf.resolve(&Term::Var(1));
        assert_eq!(res, Term::Var(1));
        // The trail should remain empty.
        assert!(uf.trail.is_empty(), "Self-binding should not update the trail");
    }

    #[test]
    fn test_undo_trail() {
        let mut uf = UnionFind::new();
        // First, bind variable 1 to Const(42).
        uf.bind(1, &Term::Const(42)).unwrap();
        let initial_trail_len = uf.trail.len();
        // Bind variable 1 again to Const(100).
        uf.bind(1, &Term::Const(100)).unwrap();
        // Resolution should now yield Const(100).
        let res = uf.resolve(&Term::Var(1));
        assert_eq!(res, Term::Const(100));
        // Undo the last binding by restoring to the previous trail length.
        uf.undo_trail(initial_trail_len);
        // Resolution should revert to Const(42).
        let res2 = uf.resolve(&Term::Var(1));
        assert_eq!(res2, Term::Const(42));
    }

    #[test]
    fn test_path_compression() {
        let mut uf = UnionFind::new();
        // Create a chain: Var(1) -> Var(2), Var(2) -> Var(3), Var(3) -> Const(7)
        uf.bind(1, &Term::Var(2)).unwrap();
        uf.bind(2, &Term::Var(3)).unwrap();
        uf.bind(3, &Term::Const(7)).unwrap();
        // Resolving Var(1) should yield Const(7) and perform path compression.
        let res = uf.resolve(&Term::Var(1));
        assert_eq!(res, Term::Const(7));
        // Now check that Var(1) is directly bound to Const(7) (i.e. path compressed).
        let binding = uf.bindings.get(&1).expect("Var(1) should have a binding");
        assert_eq!(binding, &Term::Const(7));
    }
}

