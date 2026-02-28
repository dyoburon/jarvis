mod ensure;
mod resolve;

pub use ensure::ensure_dirs;
pub use resolve::{
    cache_dir, config_dir, config_file, crash_report_dir, data_dir, identity_file, log_dir,
    resolve_config_dir,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_dir_ends_with_jarvis() {
        let path = config_dir().unwrap();
        assert!(
            path.ends_with("jarvis"),
            "config_dir should end with 'jarvis', got: {path:?}"
        );
    }

    #[test]
    fn data_dir_ends_with_jarvis() {
        let path = data_dir().unwrap();
        assert!(
            path.ends_with("jarvis"),
            "data_dir should end with 'jarvis', got: {path:?}"
        );
    }

    #[test]
    fn cache_dir_ends_with_jarvis() {
        let path = cache_dir().unwrap();
        assert!(
            path.ends_with("jarvis"),
            "cache_dir should end with 'jarvis', got: {path:?}"
        );
    }

    #[test]
    fn config_file_has_correct_name() {
        let path = config_file().unwrap();
        assert_eq!(path.file_name().unwrap().to_str().unwrap(), "config.toml");
        assert!(
            path.parent().unwrap().ends_with("jarvis"),
            "config_file parent should end with 'jarvis', got: {path:?}"
        );
    }

    #[test]
    fn identity_file_has_correct_name() {
        let path = identity_file().unwrap();
        assert_eq!(path.file_name().unwrap().to_str().unwrap(), "identity.json");
        assert!(
            path.parent().unwrap().ends_with("jarvis"),
            "identity_file parent should end with 'jarvis', got: {path:?}"
        );
    }

    #[test]
    fn log_dir_is_inside_data_dir() {
        let log = log_dir().unwrap();
        let data = data_dir().unwrap();
        assert!(
            log.starts_with(&data),
            "log_dir should be inside data_dir: log={log:?}, data={data:?}"
        );
        assert_eq!(log.file_name().unwrap().to_str().unwrap(), "logs");
    }

    #[test]
    fn resolve_config_dir_returns_ok() {
        let result = resolve_config_dir();
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("jarvis"));
    }
}
