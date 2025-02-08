#[cfg(test)]
mod tests {
    use quickcheck::quickcheck;
    use lam::machine::term::Term;
    use lam::machine::unification::UnionFind;
    
    quickcheck! {
        fn prop_undo_binding(var_id: usize, value: i32) -> bool {
            let mut uf = UnionFind::new();
            let initial_trail_len = uf.trail.len();
            if uf.bind(var_id, &Term::Const(value)).is_err() {
                return false;
            }
            let bound = uf.resolve(&Term::Var(var_id));
            if bound != Term::Const(value) {
                return false;
            }
            uf.undo_trail(initial_trail_len);
            let unbound = uf.resolve(&Term::Var(var_id));
            unbound == Term::Var(var_id)
        }
    }

    quickcheck! {
        fn prop_unify_same_constants(n: i32) -> bool {
            let mut uf = UnionFind::new();
            uf.bind(0, &Term::Const(n)).is_ok() && uf.resolve(&Term::Const(n)) == Term::Const(n)
        }
    }
}
