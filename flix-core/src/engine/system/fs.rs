use std::path::Path;
use std::process::Command;

pub fn ensure_dir_exists(path: &Path) {
    if !path.exists() {
        let _ = Command::new("sudo").arg("mkdir").arg("-p").arg(path).status();
    }
}

pub fn copy_with_sudo(from: &Path, to: &Path) {
    let _ = Command::new("sudo").arg("cp").arg(from).arg(to).status();
    let _ = Command::new("sudo").arg("chmod").arg("+x").arg(to).status();
}
