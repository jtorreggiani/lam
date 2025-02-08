#[cfg(test)]
mod tests {
    use quickcheck::quickcheck;
    use lam::machine::term::Term;
    use lam::machine::unification::UnionFind;
    // Property: After binding a variable to a constant and then undoing the binding,
    // the variable should be unbound.
    quickcheck! {
        fn prop_undo_binding(var_id: usize, value: i32) -> bool {
            let mut uf = UnionFind::new();
            let initial_trail_len = uf.trail.len();
            // Bind variable 'var_id' to constant 'value'
            let bind_result = uf.bind(var_id, &Term::Const(value));
            if bind_result.is_err() {
                return false;
            }
            let bound = uf.resolve(&Term::Var(var_id));
            if bound != Term::Const(value) {
                return false;
            }
            // Undo the binding.
            uf.undo_trail(initial_trail_len);
            let unbound = uf.resolve(&Term::Var(var_id));
            // After undo, the variable should appear unbound.
            unbound == Term::Var(var_id)
        }
    }

    // Property: Unifying a constant with itself should succeed and not change the union–find state.
    quickcheck! {
        fn prop_unify_same_constants(n: i32) -> bool {
            let mut uf = UnionFind::new();
            // Bind constant with itself (no change should occur)
            let result = uf.bind(0, &Term::Const(n));
            result.is_ok() && uf.resolve(&Term::Const(n)) == Term::Const(n)
        }
    }
}