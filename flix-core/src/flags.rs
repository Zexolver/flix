use std::process::Command;

/// Logic for handling the --quiet flag
pub fn apply_quiet(command: &mut Command, quiet: bool) {
    if quiet {
        command.stdout(std::process::Stdio::null());
        command.stderr(std::process::Stdio::null());
    }
}

/// Logic for handling the --release flag
/// For Rust/Cargo, this is usually default, but we can use this to toggle profiles
pub fn get_build_profile(release: bool) -> Vec<&'static str> {
    if release {
        vec!["--release"]
    } else {
        vec![] // Dev build
    }
}

/// Logic for checking if we should auto-confirm (--yes)
pub fn should_confirm(auto_yes: bool) -> bool {
    !auto_yes
}
