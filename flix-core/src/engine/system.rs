use crate::config::load_config;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
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

pub fn self_install() {
    let config = load_config();
    let bin_dir = config.default_install_path.unwrap_or_else(|| PathBuf::from("/usr/local/flix/bin"));
    let current_exe = env::current_exe().expect("Failed to get current exe path");
    let target_path = bin_dir.join("flix");

    ensure_dir_exists(&bin_dir);
    copy_with_sudo(&current_exe, &target_path);
    println!("✅ Flix installed to {}. Run 'flix shell-init'.", target_path.display());
}

pub fn shell_init() {
    let config = load_config();
    let base_dir = config.default_install_path.unwrap_or_else(|| PathBuf::from("/usr/local/flix/bin"));
    let etc_dir = base_dir.parent().unwrap().join("etc"); // Resolves to /usr/local/flix/etc
    
    ensure_dir_exists(&etc_dir);

    let path_str = base_dir.to_string_lossy().to_string();
    let path_line = format!("\n# Flix Package Manager\nexport PATH=\"$PATH:{}\"", path_str);
    
    let home = env::var("HOME").unwrap_or_else(|_| "/home".into());
    let shells = [".bashrc", ".zshrc", ".profile"];
    let mut updated = false;

    // --- 1. Path Setup ---
    for sh in shells {
        let p = PathBuf::from(&home).join(sh);
        if p.exists() {
            let contents = fs::read_to_string(&p).unwrap_or_default();
            
            if !contents.contains(&path_str) {
                if let Ok(mut file) = OpenOptions::new().append(true).open(&p) {
                    if let Err(e) = writeln!(file, "{}", path_line) {
                        eprintln!("❌ Failed to write to {}: {}", sh, e);
                    } else {
                        println!("✅ Added Flix to {}", sh);
                        updated = true;
                    }
                }
            } else {
                println!("ℹ️ Flix path already exists in {}", sh);
            }
        }
    }

    // --- 2. Autocomplete Setup ---
    let exe_path = env::current_exe().expect("Failed to get current exe");
    let global_comp_script = etc_dir.join("flix_completion.bash");
    let toml_path = etc_dir.join("flix.toml");

    if let Ok(output) = Command::new(&exe_path).arg("generate-completion").arg("bash").output() {
        let mut script = String::from_utf8_lossy(&output.stdout).to_string();
        
        // Dynamically inject the paths into the bash script so it scales if the user changes install dir
        let dynamic_wrapper = format!(r#"
# --- FLIX DYNAMIC WRAPPER ---
_flix_dynamic() {{
    local cur cmd
    cur="${{COMP_WORDS[COMP_CWORD]}}"
    cmd="${{COMP_WORDS[1]}}"
    
    if [[ "$cmd" == "update" || "$cmd" == "remove" ]]; then
        if [[ "$cur" != -* ]]; then
            # Ultra-fast bash regex to grab installed packages straight from the TOML
            local pkgs=$(sed -n 's/^\[packages\.\(.*\)\]$/\1/p' "{}" 2>/dev/null)
            COMPREPLY=( $(compgen -W "${{pkgs}}" -- "${{cur}}") )
            return 0
        fi
    fi
    
    # Fallback to standard clap completion
    _flix "$@"
}}
complete -F _flix_dynamic -o bashdefault -o default flix
"#, toml_path.display());

        script.push_str(&dynamic_wrapper);

        // Write locally, then sudo copy to global etc directory
        let temp_path = env::temp_dir().join("flix_completion.bash");
        if fs::write(&temp_path, script).is_ok() {
            copy_with_sudo(&temp_path, &global_comp_script);
            let _ = fs::remove_file(temp_path); // cleanup

            let source_line = format!("source {}", global_comp_script.display());
            
            // Append source line to bashrc/zshrc
            for sh in [".bashrc", ".zshrc"] {
                let p = PathBuf::from(&home).join(sh);
                if p.exists() {
                    let contents = fs::read_to_string(&p).unwrap_or_default();
                    if !contents.contains(&source_line) {
                        if let Ok(mut file) = OpenOptions::new().append(true).open(&p) {
                            writeln!(file, "\n# Flix Autocompletion\n{}", source_line).unwrap();
                            println!("✅ Hooked autocompletion into {}", sh);
                            updated = true;
                        }
                    }
                }
            }
        }
    }

    if updated {
        println!("\n✨ PATH and Autocomplete updated! To use 'flix' immediately, run:");
        println!("   source ~/.bashrc  (or your shell's config file)");
    }
}
