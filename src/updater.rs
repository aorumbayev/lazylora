use std::path::Path;
use std::process::Command;

use self_update::cargo_crate_version;
use self_update::update::ReleaseUpdate;

const GITHUB_REPO_OWNER: &str = "aorumbayev";
const GITHUB_REPO_NAME: &str = "lazylora";

const BINARY_NAME: &str = env!("CARGO_PKG_NAME");

// ============================================================================
// Install Source Detection
// ============================================================================

/// Represents how lazylora was installed on the system.
///
/// This is used to determine the appropriate update mechanism and provide
/// users with correct update instructions for their installation method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum InstallSource {
    /// Installed via `cargo install lazylora`
    Cargo,
    /// Installed via Homebrew on macOS
    Homebrew,
    /// Installed via AUR on Arch Linux (pacman-managed)
    AUR,
    /// Installed via Nix package manager
    Nixpkgs,
    /// Direct binary download or self-managed installation
    Binary,
}

impl InstallSource {
    /// Returns the update instructions for this installation source.
    ///
    /// Returns `Some(instructions)` with the appropriate command to update,
    /// or `None` if the installation source supports self-update (Binary).
    ///
    /// # Examples
    ///
    /// ```
    /// use lazylora::updater::InstallSource;
    ///
    /// let source = InstallSource::Cargo;
    /// assert!(source.update_instructions().is_some());
    ///
    /// let binary = InstallSource::Binary;
    /// assert!(binary.update_instructions().is_none());
    /// ```
    #[must_use]
    pub fn update_instructions(&self) -> Option<&'static str> {
        match self {
            InstallSource::Cargo => Some("cargo install lazylora --force"),
            InstallSource::Homebrew => Some("brew upgrade lazylora"),
            InstallSource::AUR => Some("Update using your AUR helper, e.g.: yay -Syu lazylora"),
            InstallSource::Nixpkgs => Some(
                "Update your Nix configuration and rebuild (e.g., nixos-rebuild switch or home-manager switch)",
            ),
            InstallSource::Binary => None,
        }
    }

    /// Returns whether this installation source supports self-update.
    ///
    /// Only `Binary` installations support self-update, as package manager
    /// installations should be updated through their respective package managers
    /// to maintain system consistency.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazylora::updater::InstallSource;
    ///
    /// assert!(!InstallSource::Cargo.supports_self_update());
    /// assert!(InstallSource::Binary.supports_self_update());
    /// ```
    #[must_use]
    pub fn supports_self_update(&self) -> bool {
        matches!(self, InstallSource::Binary)
    }

    /// Returns a human-readable description of the installation source.
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            InstallSource::Cargo => "Cargo (cargo install)",
            InstallSource::Homebrew => "Homebrew",
            InstallSource::AUR => "AUR (Arch User Repository)",
            InstallSource::Nixpkgs => "Nix/Nixpkgs",
            InstallSource::Binary => "Binary (direct download)",
        }
    }
}

impl std::fmt::Display for InstallSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Checks if a file is managed by pacman (Arch Linux package manager).
///
/// Runs `pacman -Qo <path>` to determine if the file is owned by a package.
/// Returns `true` if pacman reports ownership, `false` otherwise.
///
/// # Arguments
///
/// * `path` - The path to check for pacman ownership
///
/// # Examples
///
/// ```ignore
/// if is_pacman_managed("/usr/bin/lazylora") {
///     println!("File is managed by pacman");
/// }
/// ```
fn is_pacman_managed<P: AsRef<Path>>(path: P) -> bool {
    // Only run pacman check on Linux
    if !cfg!(target_os = "linux") {
        return false;
    }

    Command::new("pacman")
        .args(["-Qo", path.as_ref().to_string_lossy().as_ref()])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Detects install source from a given path.
///
/// This is the core detection logic, separated for testability.
/// Use [`detect_install_source`] for the public API that uses the current executable path.
///
/// # Arguments
///
/// * `exe_path` - The path to analyze
/// * `home_dir` - Optional home directory for cargo bin detection
/// * `check_pacman` - Whether to run pacman check (should be false in tests)
fn detect_install_source_from_path<P: AsRef<Path>>(
    exe_path: P,
    home_dir: Option<&Path>,
    check_pacman: bool,
) -> InstallSource {
    let exe_path = exe_path.as_ref();
    let path_str = exe_path.to_string_lossy();

    // Check for Cargo installation (~/.cargo/bin/)
    // We check both using Path::starts_with and string matching for cross-platform compatibility
    if let Some(home) = home_dir {
        let cargo_bin = home.join(".cargo").join("bin");
        // Use starts_with for native paths
        if exe_path.starts_with(&cargo_bin) {
            return InstallSource::Cargo;
        }
        // Also check string representation for cross-platform testing
        let cargo_bin_str = cargo_bin.to_string_lossy();
        let normalized_path = path_str.replace('\\', "/");
        let normalized_cargo = cargo_bin_str.replace('\\', "/");
        if normalized_path.starts_with(&normalized_cargo) {
            return InstallSource::Cargo;
        }
    }

    // Check for Homebrew installation (macOS)
    // Homebrew paths typically contain "homebrew" or "Cellar"
    // Note: We check the path pattern regardless of current OS for testability,
    // but in production on non-macOS this path pattern is unlikely to exist
    let path_lower = path_str.to_lowercase();
    if path_lower.contains("homebrew") || path_lower.contains("/cellar/") {
        return InstallSource::Homebrew;
    }

    // Check for Nix installation (/nix/store/)
    if path_str.contains("/nix/store/") {
        return InstallSource::Nixpkgs;
    }

    // Check for AUR installation (Arch Linux)
    // Typically installed to /usr/bin/ and managed by pacman
    if cfg!(target_os = "linux")
        && check_pacman
        && path_str.starts_with("/usr/bin/")
        && is_pacman_managed(exe_path)
    {
        return InstallSource::AUR;
    }

    // Default to Binary (self-managed/direct download)
    InstallSource::Binary
}

/// Detects how lazylora was installed on the system.
///
/// This function examines the current executable's path to determine
/// the installation source. It uses the following heuristics:
///
/// - Path in `~/.cargo/bin/` → [`InstallSource::Cargo`]
/// - Path contains "homebrew" or "Cellar" (macOS) → [`InstallSource::Homebrew`]
/// - Path is `/usr/bin/lazylora` and managed by pacman → [`InstallSource::AUR`]
/// - Path contains "/nix/store/" → [`InstallSource::Nixpkgs`]
/// - Otherwise → [`InstallSource::Binary`]
///
/// # Returns
///
/// The detected [`InstallSource`] variant.
///
/// # Examples
///
/// ```ignore
/// let source = detect_install_source();
/// println!("Installed via: {}", source);
///
/// if let Some(instructions) = source.update_instructions() {
///     println!("Update with: {}", instructions);
/// }
/// ```
#[must_use]
pub fn detect_install_source() -> InstallSource {
    let exe_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(_) => return InstallSource::Binary,
    };

    detect_install_source_from_path(&exe_path, dirs::home_dir().as_deref(), true)
}

fn configure_updater() -> Result<Box<dyn ReleaseUpdate>, Box<dyn std::error::Error + Send + Sync>> {
    let updater = self_update::backends::github::Update::configure()
        .repo_owner(GITHUB_REPO_OWNER)
        .repo_name(GITHUB_REPO_NAME)
        .bin_name(BINARY_NAME)
        .identifier(BINARY_NAME)
        .bin_path_in_archive(
            "{{ bin }}-{{ target }}/{{ bin }}{% if target contains 'windows' %}.exe{% endif %}",
        )
        .current_version(cargo_crate_version!())
        .build()?;
    Ok(updater)
}

pub fn check_for_updates() -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    println!("Checking for updates...");

    let updater = configure_updater()?;
    let latest_release = updater.get_latest_release()?;

    let current_version = cargo_crate_version!();
    let latest_version = &latest_release.version;

    println!("Current version: {}", current_version);
    println!("Latest version: {}", latest_version);

    if semver::Version::parse(latest_version)? > semver::Version::parse(current_version)? {
        Ok(Some(latest_version.to_string()))
    } else {
        println!("You are using the latest version");
        Ok(None)
    }
}

pub fn update_app() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting update...");

    let updater = configure_updater()?;

    match updater.update() {
        Ok(_status) => {
            println!("Update successful!");
            Ok(())
        }
        Err(e) => {
            if let self_update::errors::Error::Io(io_err) = &e
                && io_err.kind() == std::io::ErrorKind::PermissionDenied
            {
                return Err(
                    "Update failed: Permission denied. Try running with sudo or as administrator."
                        .into(),
                );
            }
            Err(format!("Update download/install failed: {}", e).into())
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // -------------------------------------------------------------------------
    // InstallSource properties - Table-driven tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_install_source_properties() {
        // Test cases: (source, has_instructions, supports_self_update, instruction_contains)
        let cases = [
            (InstallSource::Cargo, true, false, "cargo install"),
            (InstallSource::Homebrew, true, false, "brew upgrade"),
            (InstallSource::AUR, true, false, "AUR"),
            (InstallSource::Nixpkgs, true, false, "nix"),
            (InstallSource::Binary, false, true, ""),
        ];

        for (source, has_instructions, supports_self_update, instruction_contains) in cases {
            // Test update instructions
            let instructions = source.update_instructions();
            if has_instructions {
                assert!(
                    instructions.is_some(),
                    "Expected instructions for {:?}",
                    source
                );
                let instr = instructions.unwrap().to_lowercase();
                assert!(
                    instr.contains(&instruction_contains.to_lowercase()),
                    "Instructions for {:?} should contain '{}', got '{}'",
                    source,
                    instruction_contains,
                    instr
                );
            } else {
                assert!(
                    instructions.is_none(),
                    "Expected no instructions for {:?}",
                    source
                );
            }

            // Test self-update support
            assert_eq!(
                source.supports_self_update(),
                supports_self_update,
                "Self-update support mismatch for {:?}",
                source
            );

            // Test description exists and Display trait
            assert!(
                !source.description().is_empty(),
                "Empty description for {:?}",
                source
            );
            assert_eq!(
                format!("{source}"),
                source.description(),
                "Display mismatch for {:?}",
                source
            );
        }
    }

    #[test]
    fn test_install_source_traits() {
        // Test Debug, Clone, PartialEq, Eq traits
        let source = InstallSource::Homebrew;
        let cloned = source;

        // Clone (Copy)
        assert_eq!(source, cloned);

        // PartialEq and Eq
        assert_eq!(InstallSource::Cargo, InstallSource::Cargo);
        assert_ne!(InstallSource::Cargo, InstallSource::Binary);

        // Debug
        let debug_str = format!("{:?}", InstallSource::AUR);
        assert!(debug_str.contains("AUR"));
    }

    // -------------------------------------------------------------------------
    // Path detection - Cargo installation (table-driven)
    // -------------------------------------------------------------------------

    #[test]
    fn test_cargo_path_detection() {
        // Test cases: (exe_path, home_path, expected)
        let cases = [
            // Unix-style paths
            (
                "/home/user/.cargo/bin/lazylora",
                "/home/user",
                InstallSource::Cargo,
            ),
            (
                "/Users/developer/.cargo/bin/lazylora",
                "/Users/developer",
                InstallSource::Cargo,
            ),
            // Windows-style paths
            (
                "C:\\Users\\Developer\\.cargo\\bin\\lazylora.exe",
                "C:\\Users\\Developer",
                InstallSource::Cargo,
            ),
            // Not in cargo bin
            (
                "/home/user/projects/lazylora/target/debug/lazylora",
                "/home/user",
                InstallSource::Binary,
            ),
            (
                "/home/user/projects/lazylora/target/release/lazylora",
                "/home/user",
                InstallSource::Binary,
            ),
            // Different user (should not match)
            (
                "/home/bob/.cargo/bin/lazylora",
                "/home/alice",
                InstallSource::Binary,
            ),
        ];

        for (exe_path, home_path, expected) in cases {
            let result = detect_install_source_from_path(
                PathBuf::from(exe_path),
                Some(&PathBuf::from(home_path)),
                false,
            );
            assert_eq!(result, expected, "Path: {exe_path}, Home: {home_path}");
        }
    }

    // -------------------------------------------------------------------------
    // Path detection - Homebrew installation (table-driven)
    // -------------------------------------------------------------------------

    #[test]
    fn test_homebrew_path_detection() {
        let cases = [
            "/usr/local/Cellar/lazylora/1.0.0/bin/lazylora",
            "/opt/homebrew/bin/lazylora",
            "/opt/homebrew/Cellar/lazylora/1.0.0/bin/lazylora",
            // Case insensitive variants
            "/usr/local/CELLAR/lazylora/bin/lazylora",
            "/opt/HOMEBREW/bin/lazylora",
            "/usr/local/cellar/lazylora/bin/lazylora",
        ];

        for path in cases {
            let result = detect_install_source_from_path(PathBuf::from(path), None, false);
            assert_eq!(result, InstallSource::Homebrew, "Path: {path}");
        }
    }

    // -------------------------------------------------------------------------
    // Path detection - Nix installation (table-driven)
    // -------------------------------------------------------------------------

    #[test]
    fn test_nix_path_detection() {
        let cases = [
            "/nix/store/abc123-lazylora-1.0.0/bin/lazylora",
            "/nix/store/eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee-lazylora-1.0.0/bin/lazylora",
            "/nix/store/xyz789-profile/bin/lazylora",
            "/nix/store/hash-lazylora/bin/lazylora",
        ];

        for path in cases {
            let result = detect_install_source_from_path(PathBuf::from(path), None, false);
            assert_eq!(result, InstallSource::Nixpkgs, "Path: {path}");
        }
    }

    // -------------------------------------------------------------------------
    // Path detection - Binary installation (table-driven)
    // -------------------------------------------------------------------------

    #[test]
    fn test_binary_path_detection() {
        let cases = [
            // Unix-style paths
            "/usr/local/bin/lazylora",
            "/home/user/bin/lazylora",
            "/opt/lazylora/lazylora",
            "/usr/bin/lazylora", // Without pacman check
            "/Applications/lazylora",
            // Windows-style paths
            "C:\\Program Files\\LazyLora\\lazylora.exe",
            "C:\\Users\\Dev\\AppData\\Local\\lazylora\\lazylora.exe",
            "C:\\Program Files\\lazylora\\lazylora.exe",
            // Relative paths
            "./lazylora",
            // Empty path
            "",
        ];

        for path in cases {
            let result = detect_install_source_from_path(PathBuf::from(path), None, false);
            assert_eq!(result, InstallSource::Binary, "Path: {path}");
        }
    }

    // -------------------------------------------------------------------------
    // Path detection - Priority and edge cases (table-driven)
    // -------------------------------------------------------------------------

    #[test]
    fn test_path_detection_priority() {
        // Test cases: (exe_path, home_path, expected, description)
        let cases = [
            // Cargo takes priority over generic paths
            (
                "/home/user/.cargo/bin/lazylora",
                Some("/home/user"),
                InstallSource::Cargo,
                "Cargo priority",
            ),
            // Homebrew takes priority
            (
                "/opt/homebrew/bin/lazylora",
                Some("/home/user"),
                InstallSource::Homebrew,
                "Homebrew priority",
            ),
            // Nix takes priority
            (
                "/nix/store/hash-lazylora/bin/lazylora",
                None,
                InstallSource::Nixpkgs,
                "Nix priority",
            ),
            // No home dir provided (cargo check skipped)
            (
                "/home/user/.cargo/bin/lazylora",
                None,
                InstallSource::Binary,
                "No home dir",
            ),
        ];

        for (exe_path, home_path, expected, description) in cases {
            let result = detect_install_source_from_path(
                PathBuf::from(exe_path),
                home_path.map(PathBuf::from).as_deref(),
                false,
            );
            assert_eq!(result, expected, "{description}: {exe_path}");
        }
    }

    #[test]
    fn test_special_path_cases() {
        // Test cases: (exe_path, home_path, expected, description)
        let cases = [
            // Paths with spaces
            (
                "/home/user name/.cargo/bin/lazylora",
                Some("/home/user name"),
                InstallSource::Cargo,
                "Path with spaces",
            ),
            // Paths with unicode
            (
                "/home/用户/.cargo/bin/lazylora",
                Some("/home/用户"),
                InstallSource::Cargo,
                "Path with unicode",
            ),
            // Empty path
            ("", None, InstallSource::Binary, "Empty path"),
        ];

        for (exe_path, home_path, expected, description) in cases {
            let result = detect_install_source_from_path(
                PathBuf::from(exe_path),
                home_path.map(PathBuf::from).as_deref(),
                false,
            );
            assert_eq!(result, expected, "{description}: {exe_path}");
        }
    }

    // -------------------------------------------------------------------------
    // Cross-platform availability tests (table-driven)
    // -------------------------------------------------------------------------

    #[test]
    fn test_cross_platform_availability() {
        // Test cases: (exe_path, home_path, expected, platform)
        let cases = [
            // Cargo - available on all platforms
            (
                "/home/user/.cargo/bin/lazylora",
                Some("/home/user"),
                InstallSource::Cargo,
                "Linux",
            ),
            (
                "/Users/user/.cargo/bin/lazylora",
                Some("/Users/user"),
                InstallSource::Cargo,
                "macOS",
            ),
            (
                "C:\\Users\\User\\.cargo\\bin\\lazylora.exe",
                Some("C:\\Users\\User"),
                InstallSource::Cargo,
                "Windows",
            ),
            // Binary - available on all platforms
            (
                "/usr/local/bin/lazylora",
                None,
                InstallSource::Binary,
                "Linux",
            ),
            (
                "/Applications/lazylora",
                None,
                InstallSource::Binary,
                "macOS",
            ),
            (
                "C:\\Program Files\\lazylora\\lazylora.exe",
                None,
                InstallSource::Binary,
                "Windows",
            ),
        ];

        for (exe_path, home_path, expected, platform) in cases {
            let result = detect_install_source_from_path(
                PathBuf::from(exe_path),
                home_path.map(PathBuf::from).as_deref(),
                false,
            );
            assert_eq!(result, expected, "{platform}: {exe_path}");
        }

        // Verify all sources have valid descriptions (documentation check)
        let all_sources = [
            InstallSource::Cargo,
            InstallSource::Homebrew,
            InstallSource::AUR,
            InstallSource::Nixpkgs,
            InstallSource::Binary,
        ];

        for source in all_sources {
            assert!(
                !source.description().is_empty(),
                "{source:?} should have a description"
            );
        }
    }
}
