//! Update checker — checks GitHub Releases for new versions.

use serde::Deserialize;

/// A GitHub release entry.
#[derive(Debug, Deserialize)]
pub struct GithubRelease {
    /// The tag name, e.g. "v0.2.0".
    pub tag_name: String,
    /// URL to the release page.
    pub html_url: String,
    /// Release notes body (markdown).
    pub body: Option<String>,
}

/// Checks for newer versions on GitHub Releases.
pub struct UpdateChecker {
    api_url: String,
    current_version: String,
}

impl UpdateChecker {
    /// Create a checker for the given GitHub `owner/repo`.
    pub fn new(repo: &str) -> Self {
        Self {
            api_url: format!("https://api.github.com/repos/{repo}/releases/latest"),
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Check if a newer version is available.
    ///
    /// Returns `Some(release)` if a newer version exists, `None` otherwise.
    /// Returns `None` on any network or parsing error (fail silently).
    pub async fn check(&self) -> Option<GithubRelease> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok()?;

        let response = client
            .get(&self.api_url)
            .header("User-Agent", "jarvis-updater")
            .send()
            .await
            .ok()?;

        if !response.status().is_success() {
            return None;
        }

        let release: GithubRelease = response.json().await.ok()?;

        let latest = release.tag_name.trim_start_matches('v');
        if latest != self.current_version && is_newer(latest, &self.current_version) {
            Some(release)
        } else {
            None
        }
    }

    /// Current version string.
    pub fn current_version(&self) -> &str {
        &self.current_version
    }
}

/// Simple semver comparison: returns true if `a` > `b`.
fn is_newer(a: &str, b: &str) -> bool {
    let parse =
        |v: &str| -> Vec<u64> { v.split('.').filter_map(|s| s.parse::<u64>().ok()).collect() };

    let va = parse(a);
    let vb = parse(b);

    for i in 0..va.len().max(vb.len()) {
        let a_part = va.get(i).copied().unwrap_or(0);
        let b_part = vb.get(i).copied().unwrap_or(0);
        if a_part > b_part {
            return true;
        }
        if a_part < b_part {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comparison() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(is_newer("1.0.1", "1.0.0"));
        assert!(!is_newer("0.1.0", "0.2.0"));
        assert!(!is_newer("1.0.0", "1.0.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
    }

    #[test]
    fn version_comparison_different_lengths() {
        assert!(is_newer("1.0.0", "0.9"));
        assert!(!is_newer("0.9", "1.0.0"));
        assert!(is_newer("1.1", "1.0.0"));
    }

    #[test]
    fn checker_creation() {
        let checker = UpdateChecker::new("dylan/jarvis");
        assert!(!checker.current_version().is_empty());
    }
}
