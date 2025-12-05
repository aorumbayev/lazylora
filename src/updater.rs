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
    pub fn supports_self_update(&self) -> bool {
        matches!(self, InstallSource::Binary)
    }

    /// Returns a human-readable description of the installation source.
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
    // InstallSource enum tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_install_source_update_instructions_cargo() {
        let source = InstallSource::Cargo;
        let instructions = source.update_instructions();
        assert!(instructions.is_some());
        assert!(instructions.unwrap().contains("cargo install"));
        assert!(instructions.unwrap().contains("lazylora"));
    }

    #[test]
    fn test_install_source_update_instructions_homebrew() {
        let source = InstallSource::Homebrew;
        let instructions = source.update_instructions();
        assert!(instructions.is_some());
        assert!(instructions.unwrap().contains("brew upgrade"));
    }

    #[test]
    fn test_install_source_update_instructions_aur() {
        let source = InstallSource::AUR;
        let instructions = source.update_instructions();
        assert!(instructions.is_some());
        assert!(instructions.unwrap().contains("yay") || instructions.unwrap().contains("AUR"));
    }

    #[test]
    fn test_install_source_update_instructions_nixpkgs() {
        let source = InstallSource::Nixpkgs;
        let instructions = source.update_instructions();
        assert!(instructions.is_some());
        assert!(instructions.unwrap().contains("nix") || instructions.unwrap().contains("Nix"));
    }

    #[test]
    fn test_install_source_update_instructions_binary() {
        let source = InstallSource::Binary;
        assert!(source.update_instructions().is_none());
    }

    #[test]
    fn test_install_source_supports_self_update() {
        // Only Binary should support self-update
        assert!(!InstallSource::Cargo.supports_self_update());
        assert!(!InstallSource::Homebrew.supports_self_update());
        assert!(!InstallSource::AUR.supports_self_update());
        assert!(!InstallSource::Nixpkgs.supports_self_update());
        assert!(InstallSource::Binary.supports_self_update());
    }

    #[test]
    fn test_install_source_description() {
        assert!(!InstallSource::Cargo.description().is_empty());
        assert!(!InstallSource::Homebrew.description().is_empty());
        assert!(!InstallSource::AUR.description().is_empty());
        assert!(!InstallSource::Nixpkgs.description().is_empty());
        assert!(!InstallSource::Binary.description().is_empty());
    }

    #[test]
    fn test_install_source_display() {
        // Display should return the description
        assert_eq!(
            format!("{}", InstallSource::Cargo),
            InstallSource::Cargo.description()
        );
        assert_eq!(
            format!("{}", InstallSource::Binary),
            InstallSource::Binary.description()
        );
    }

    #[test]
    fn test_install_source_equality() {
        assert_eq!(InstallSource::Cargo, InstallSource::Cargo);
        assert_ne!(InstallSource::Cargo, InstallSource::Binary);
    }

    #[test]
    fn test_install_source_clone() {
        let source = InstallSource::Homebrew;
        let cloned = source;
        assert_eq!(source, cloned);
    }

    #[test]
    fn test_install_source_debug() {
        // Ensure Debug is implemented
        let debug_str = format!("{:?}", InstallSource::AUR);
        assert!(debug_str.contains("AUR"));
    }

    // -------------------------------------------------------------------------
    // Path detection tests - Cargo installation
    // -------------------------------------------------------------------------

    #[test]
    fn test_detect_cargo_unix_style() {
        let home = PathBuf::from("/home/user");
        let exe_path = PathBuf::from("/home/user/.cargo/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, Some(&home), false);
        assert_eq!(result, InstallSource::Cargo);
    }

    #[test]
    fn test_detect_cargo_macos_style() {
        let home = PathBuf::from("/Users/developer");
        let exe_path = PathBuf::from("/Users/developer/.cargo/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, Some(&home), false);
        assert_eq!(result, InstallSource::Cargo);
    }

    #[test]
    fn test_detect_cargo_windows_style() {
        let home = PathBuf::from("C:\\Users\\Developer");
        let exe_path = PathBuf::from("C:\\Users\\Developer\\.cargo\\bin\\lazylora.exe");

        let result = detect_install_source_from_path(&exe_path, Some(&home), false);
        assert_eq!(result, InstallSource::Cargo);
    }

    #[test]
    fn test_detect_cargo_not_in_cargo_bin() {
        let home = PathBuf::from("/home/user");
        let exe_path = PathBuf::from("/home/user/projects/lazylora/target/debug/lazylora");

        let result = detect_install_source_from_path(&exe_path, Some(&home), false);
        assert_eq!(result, InstallSource::Binary);
    }

    #[test]
    fn test_detect_cargo_different_user_home() {
        // Binary in one user's cargo, but home is different user
        let home = PathBuf::from("/home/alice");
        let exe_path = PathBuf::from("/home/bob/.cargo/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, Some(&home), false);
        // Should NOT match because it's not in alice's cargo bin
        assert_eq!(result, InstallSource::Binary);
    }

    #[test]
    fn test_detect_no_home_dir() {
        let exe_path = PathBuf::from("/home/user/.cargo/bin/lazylora");

        // No home directory provided
        let result = detect_install_source_from_path(&exe_path, None, false);
        assert_eq!(result, InstallSource::Binary);
    }

    // -------------------------------------------------------------------------
    // Path detection tests - Homebrew installation (macOS)
    // -------------------------------------------------------------------------

    #[test]
    fn test_detect_homebrew_cellar_path() {
        let exe_path = PathBuf::from("/usr/local/Cellar/lazylora/1.0.0/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, None, false);
        assert_eq!(result, InstallSource::Homebrew);
    }

    #[test]
    fn test_detect_homebrew_opt_path() {
        let exe_path = PathBuf::from("/opt/homebrew/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, None, false);
        assert_eq!(result, InstallSource::Homebrew);
    }

    #[test]
    fn test_detect_homebrew_arm64_cellar() {
        // Apple Silicon Macs use /opt/homebrew
        let exe_path = PathBuf::from("/opt/homebrew/Cellar/lazylora/1.0.0/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, None, false);
        assert_eq!(result, InstallSource::Homebrew);
    }

    #[test]
    fn test_detect_homebrew_case_insensitive() {
        // Test case variations
        let paths = [
            "/usr/local/CELLAR/lazylora/bin/lazylora",
            "/opt/HOMEBREW/bin/lazylora",
            "/usr/local/cellar/lazylora/bin/lazylora",
        ];

        for path in paths {
            let exe_path = PathBuf::from(path);
            let result = detect_install_source_from_path(&exe_path, None, false);
            assert_eq!(result, InstallSource::Homebrew, "Failed for path: {}", path);
        }
    }

    // -------------------------------------------------------------------------
    // Path detection tests - Nix installation
    // -------------------------------------------------------------------------

    #[test]
    fn test_detect_nix_store() {
        let exe_path = PathBuf::from("/nix/store/abc123-lazylora-1.0.0/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, None, false);
        assert_eq!(result, InstallSource::Nixpkgs);
    }

    #[test]
    fn test_detect_nix_store_with_hash() {
        let exe_path = PathBuf::from(
            "/nix/store/eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee-lazylora-1.0.0/bin/lazylora",
        );

        let result = detect_install_source_from_path(&exe_path, None, false);
        assert_eq!(result, InstallSource::Nixpkgs);
    }

    #[test]
    fn test_detect_nix_profile() {
        // Nix profile paths also contain /nix/store/
        let exe_path = PathBuf::from("/nix/store/xyz789-profile/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, None, false);
        assert_eq!(result, InstallSource::Nixpkgs);
    }

    // -------------------------------------------------------------------------
    // Path detection tests - Binary (direct download)
    // -------------------------------------------------------------------------

    #[test]
    fn test_detect_binary_usr_local_bin() {
        let exe_path = PathBuf::from("/usr/local/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, None, false);
        assert_eq!(result, InstallSource::Binary);
    }

    #[test]
    fn test_detect_binary_home_bin() {
        let home = PathBuf::from("/home/user");
        let exe_path = PathBuf::from("/home/user/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, Some(&home), false);
        assert_eq!(result, InstallSource::Binary);
    }

    #[test]
    fn test_detect_binary_opt() {
        let exe_path = PathBuf::from("/opt/lazylora/lazylora");

        let result = detect_install_source_from_path(&exe_path, None, false);
        assert_eq!(result, InstallSource::Binary);
    }

    #[test]
    fn test_detect_binary_windows_program_files() {
        let exe_path = PathBuf::from("C:\\Program Files\\LazyLora\\lazylora.exe");

        let result = detect_install_source_from_path(&exe_path, None, false);
        assert_eq!(result, InstallSource::Binary);
    }

    #[test]
    fn test_detect_binary_windows_appdata() {
        let exe_path = PathBuf::from("C:\\Users\\Dev\\AppData\\Local\\lazylora\\lazylora.exe");

        let result = detect_install_source_from_path(&exe_path, None, false);
        assert_eq!(result, InstallSource::Binary);
    }

    #[test]
    fn test_detect_binary_current_directory() {
        let exe_path = PathBuf::from("./lazylora");

        let result = detect_install_source_from_path(&exe_path, None, false);
        assert_eq!(result, InstallSource::Binary);
    }

    #[test]
    fn test_detect_binary_target_debug() {
        // Development build
        let home = PathBuf::from("/home/user");
        let exe_path = PathBuf::from("/home/user/projects/lazylora/target/debug/lazylora");

        let result = detect_install_source_from_path(&exe_path, Some(&home), false);
        assert_eq!(result, InstallSource::Binary);
    }

    #[test]
    fn test_detect_binary_target_release() {
        // Release build
        let home = PathBuf::from("/home/user");
        let exe_path = PathBuf::from("/home/user/projects/lazylora/target/release/lazylora");

        let result = detect_install_source_from_path(&exe_path, Some(&home), false);
        assert_eq!(result, InstallSource::Binary);
    }

    // -------------------------------------------------------------------------
    // Path detection tests - AUR (Arch Linux)
    // Note: AUR detection requires pacman check, which we skip in tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_detect_usr_bin_without_pacman_check() {
        // Without pacman check, /usr/bin should be detected as Binary
        let exe_path = PathBuf::from("/usr/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, None, false);
        assert_eq!(result, InstallSource::Binary);
    }

    // -------------------------------------------------------------------------
    // Edge cases and priority tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_priority_cargo_over_binary() {
        // Cargo paths should be detected before falling through to Binary
        let home = PathBuf::from("/home/user");
        let exe_path = PathBuf::from("/home/user/.cargo/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, Some(&home), false);
        assert_eq!(result, InstallSource::Cargo);
    }

    #[test]
    fn test_priority_homebrew_over_binary() {
        // Homebrew paths should be detected before falling through to Binary
        let home = PathBuf::from("/Users/user");
        let exe_path = PathBuf::from("/opt/homebrew/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, Some(&home), false);
        assert_eq!(result, InstallSource::Homebrew);
    }

    #[test]
    fn test_priority_nix_over_binary() {
        // Nix paths should be detected before falling through to Binary
        let exe_path = PathBuf::from("/nix/store/hash-lazylora/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, None, false);
        assert_eq!(result, InstallSource::Nixpkgs);
    }

    #[test]
    fn test_empty_path() {
        let exe_path = PathBuf::from("");

        let result = detect_install_source_from_path(&exe_path, None, false);
        assert_eq!(result, InstallSource::Binary);
    }

    #[test]
    fn test_path_with_spaces() {
        let home = PathBuf::from("/home/user name");
        let exe_path = PathBuf::from("/home/user name/.cargo/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, Some(&home), false);
        assert_eq!(result, InstallSource::Cargo);
    }

    #[test]
    fn test_path_with_unicode() {
        let home = PathBuf::from("/home/用户");
        let exe_path = PathBuf::from("/home/用户/.cargo/bin/lazylora");

        let result = detect_install_source_from_path(&exe_path, Some(&home), false);
        assert_eq!(result, InstallSource::Cargo);
    }

    // -------------------------------------------------------------------------
    // Platform-specific distribution availability tests
    // These document which install sources are available on each platform
    // -------------------------------------------------------------------------

    #[test]
    fn test_available_sources_documented() {
        // Document distribution availability:
        // - Cargo: Linux, macOS, Windows
        // - AUR: Linux (Arch only)
        // - Homebrew: macOS only (we don't support linuxbrew)
        // - Binary: Linux, macOS, Windows

        // All sources should have valid descriptions
        let all_sources = [
            InstallSource::Cargo,
            InstallSource::Homebrew,
            InstallSource::AUR,
            InstallSource::Nixpkgs,
            InstallSource::Binary,
        ];

        for source in all_sources {
            assert!(!source.description().is_empty());
        }
    }

    #[test]
    fn test_cargo_available_all_platforms() {
        // Cargo installation works on all platforms
        // Test Linux-style path
        let home = PathBuf::from("/home/user");
        let exe_path = PathBuf::from("/home/user/.cargo/bin/lazylora");
        assert_eq!(
            detect_install_source_from_path(&exe_path, Some(&home), false),
            InstallSource::Cargo
        );

        // Test macOS-style path
        let home = PathBuf::from("/Users/user");
        let exe_path = PathBuf::from("/Users/user/.cargo/bin/lazylora");
        assert_eq!(
            detect_install_source_from_path(&exe_path, Some(&home), false),
            InstallSource::Cargo
        );

        // Test Windows-style path
        let home = PathBuf::from("C:\\Users\\User");
        let exe_path = PathBuf::from("C:\\Users\\User\\.cargo\\bin\\lazylora.exe");
        assert_eq!(
            detect_install_source_from_path(&exe_path, Some(&home), false),
            InstallSource::Cargo
        );
    }

    #[test]
    fn test_binary_available_all_platforms() {
        // Direct binary download works on all platforms
        // Test Linux-style path
        assert_eq!(
            detect_install_source_from_path(PathBuf::from("/usr/local/bin/lazylora"), None, false),
            InstallSource::Binary
        );

        // Test macOS-style path
        assert_eq!(
            detect_install_source_from_path(PathBuf::from("/Applications/lazylora"), None, false),
            InstallSource::Binary
        );

        // Test Windows-style path
        assert_eq!(
            detect_install_source_from_path(
                PathBuf::from("C:\\Program Files\\lazylora\\lazylora.exe"),
                None,
                false
            ),
            InstallSource::Binary
        );
    }
}
