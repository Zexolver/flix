use flix_core::{engine, flags};

pub fn handle_install(url: &str, force: bool, tags: Vec<String>) {
    engine::install(url, force, &tags, None);
}

pub fn handle_remove(name: &str, auto_yes: bool) {
    if flags::confirm_action(&format!("Are you sure you want to remove '{}'?", name), auto_yes) {
        engine::remove(name);
    } else {
        println!("❌ Aborted.");
    }
}

pub fn handle_shell_init() {
    engine::shell_init();
}

pub fn handle_setup() {
    engine::self_install();
}
