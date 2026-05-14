use crate::engine::system::fs::copy_with_sudo;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) fn setup_autocomplete(etc_dir: &Path, home: &str) -> bool {
    let exe_path = env::current_exe().expect("Failed to get current exe");
    let global_comp_script = etc_dir.join("flix_completion.bash");
    let mut updated = false;

    if let Ok(output) = Command::new(&exe_path).arg("generate-completion").arg("bash").output() {
        let mut script = String::from_utf8_lossy(&output.stdout).to_string();
        
        let dynamic_wrapper = r#"
# --- FLIX DYNAMIC WRAPPER ---
_flix_dynamic() {
    local cur cmd
    cur="${COMP_WORDS[COMP_CWORD]}"
    cmd="${COMP_WORDS[1]}"
    
    if [[ "$cmd" == "update" || "$cmd" == "remove" ]]; then
        if [[ "$cur" != -* ]]; then
            # Call the binary directly to get the current list
            local pkgs=$(flix _list-installed 2>/dev/null)
            COMPREPLY=( $(compgen -W "${pkgs}" -- "${cur}") )
            return 0
        fi
    fi
    
    # Fallback to standard clap completion
    _flix "$@"
}
complete -F _flix_dynamic -o bashdefault -o default flix
"#;

        script.push_str(dynamic_wrapper);

        let temp_path = env::temp_dir().join("flix_completion.bash");
        if fs::write(&temp_path, script).is_ok() {
            copy_with_sudo(&temp_path, &global_comp_script);
            let _ = fs::remove_file(temp_path);

            let source_line = format!("source {}", global_comp_script.display());
            
            for sh in [".bashrc", ".zshrc"] {
                let p = PathBuf::from(home).join(sh);
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
    updated
}
