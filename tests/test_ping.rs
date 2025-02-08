#[cfg(test)]
mod tests {
    use lam::machine::core::ping;

    #[test]
    fn test_ping() {
        assert_eq!(ping(), "pong");
    }
}
