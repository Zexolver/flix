use std::io::{self, Write};

/// Logic for -f / --force
pub fn should_overwrite(force: bool, exists: bool) -> bool {
    if !exists { return true; }
    force
}

/// Logic for -y / --yes (Interactive confirmation)
pub fn confirm_action(prompt: &str, auto_yes: bool) -> bool {
    if auto_yes { return true; }

    print!("{} [y/N]: ", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase() == "y"
}
