use std::process::Command;

#[derive(Debug, Clone)]
pub struct ShellContext {
    pub os: String,
    pub shell: String,
    pub package_manager: String,
}

impl ShellContext {
    pub fn detect() -> Self {
        ShellContext {
            os: detect_os(),
            shell: detect_shell(),
            package_manager: detect_package_manager(),
        }
    }
}

fn detect_os() -> String {
    std::env::consts::OS.to_string()
}

fn detect_shell() -> String {
    if cfg!(target_os = "windows") {
        // Check if running in PowerShell
        if std::env::var("PSModulePath").is_ok() {
            "powershell".to_string()
        } else {
            "cmd".to_string()
        }
    } else {
        std::env::var("SHELL")
            .ok()
            .and_then(|s| {
                std::path::Path::new(&s)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "sh".to_string())
    }
}

fn detect_package_manager() -> String {
    let managers = ["pacman", "apt", "brew", "dnf", "zypper"];
    let which_cmd = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };

    for pm in &managers {
        let result = Command::new(which_cmd).arg(pm).output();
        if let Ok(output) = result {
            if output.status.success() {
                return pm.to_string();
            }
        }
    }

    "unknown".to_string()
}
