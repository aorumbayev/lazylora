use self_update::cargo_crate_version;
use self_update::update::ReleaseUpdate;

const GITHUB_REPO_OWNER: &str = "aorumbayev";
const GITHUB_REPO_NAME: &str = "lazylora";

const BINARY_NAME: &str = env!("CARGO_PKG_NAME");

fn configure_updater() -> Result<Box<dyn ReleaseUpdate>, Box<dyn std::error::Error>> {
    let updater = self_update::backends::github::Update::configure()
        .repo_owner(GITHUB_REPO_OWNER)
        .repo_name(GITHUB_REPO_NAME)
        .bin_name(BINARY_NAME)
        .current_version(cargo_crate_version!())
        .build()?;
    Ok(updater)
}

pub fn check_for_updates() -> Result<Option<String>, Box<dyn std::error::Error>> {
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

pub fn update_app() -> Result<(), Box<dyn std::error::Error>> {
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
