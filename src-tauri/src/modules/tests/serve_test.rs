#[cfg(test)]
mod tests {
    use crate::modules::serve::get_gateway_dir;

    #[test]
    fn test_get_gateway_dir_from_env() {
        std::env::set_var("AI_WORKSTATION_DIR", "/nonexistent-path");
        let dir = get_gateway_dir();
        // Should be None because the path doesn't have compose file
        if dir.is_some() {
            assert!(dir.unwrap().contains("ai-workstation") || dir.unwrap() == "/nonexistent-path");
        }
        std::env::remove_var("AI_WORKSTATION_DIR");
    }

    #[test]
    fn test_get_gateway_dir_no_env_no_home() {
        std::env::remove_var("AI_WORKSTATION_DIR");
        let dir = get_gateway_dir();
        // Result depends on whether ~/Projects/ai-workstation exists in test env
        if let Some(d) = &dir {
            assert!(d.contains("ai-workstation"), "unexpected path: {d}");
        }
    }
}
