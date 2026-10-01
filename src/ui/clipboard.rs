// Clipboard helper supporting Linux (wl-copy/wl-paste, xclip, xsel) and Windows (clip, powershell) with internal fallback buffer

static INTERNAL_CLIPBOARD: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

pub fn get_clipboard() -> String {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        if let Ok(out) = std::process::Command::new("wl-paste").output() {
            if out.status.success() {
                if let Ok(s) = String::from_utf8(out.stdout) {
                    let trimmed = s.trim_matches(|c| c == '\r' || c == '\n').to_string();
                    if !trimmed.is_empty() {
                        return trimmed;
                    }
                }
            }
        }
        if let Ok(out) = std::process::Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
        {
            if out.status.success() {
                if let Ok(s) = String::from_utf8(out.stdout) {
                    let trimmed = s.trim_matches(|c| c == '\r' || c == '\n').to_string();
                    if !trimmed.is_empty() {
                        return trimmed;
                    }
                }
            }
        }
        if let Ok(out) = std::process::Command::new("xsel")
            .args(["--clipboard", "--output"])
            .output()
        {
            if out.status.success() {
                if let Ok(s) = String::from_utf8(out.stdout) {
                    let trimmed = s.trim_matches(|c| c == '\r' || c == '\n').to_string();
                    if !trimmed.is_empty() {
                        return trimmed;
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(out) = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", "Get-Clipboard"])
            .output()
        {
            if out.status.success() {
                if let Ok(s) = String::from_utf8(out.stdout) {
                    let trimmed = s.trim_matches(|c| c == '\r' || c == '\n').to_string();
                    if !trimmed.is_empty() {
                        return trimmed;
                    }
                }
            }
        }
    }

    if let Ok(g) = INTERNAL_CLIPBOARD.lock() {
        g.clone()
    } else {
        String::new()
    }
}

pub fn set_clipboard(text: &str) {
    if let Ok(mut g) = INTERNAL_CLIPBOARD.lock() {
        *g = text.to_string();
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        use std::io::Write;
        let copied = if let Ok(mut child) = std::process::Command::new("wl-copy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            child.wait().map(|s| s.success()).unwrap_or(false)
        } else {
            false
        };

        if !copied {
            let copied_xclip = if let Ok(mut child) = std::process::Command::new("xclip")
                .args(["-selection", "clipboard", "-in"])
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                child.wait().map(|s| s.success()).unwrap_or(false)
            } else {
                false
            };

            if !copied_xclip {
                if let Ok(mut child) = std::process::Command::new("xsel")
                    .args(["--clipboard", "--input"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(text.as_bytes());
                    }
                    let _ = child.wait();
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::io::Write;
        let mut success = false;
        if let Ok(mut child) = std::process::Command::new("clip")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            success = child.wait().map(|s| s.success()).unwrap_or(false);
        }

        if !success {
            let escaped = text.replace('\'', "''");
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", &format!("Set-Clipboard -Value '{}'", escaped)])
                .output();
        }
    }
}
