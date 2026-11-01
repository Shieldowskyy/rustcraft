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

    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "windows"))]
    {
        let text = text.to_string();
        if let Err(error) = std::thread::Builder::new()
            .name("clipboard-write".to_string())
            .spawn(move || set_system_clipboard(&text))
        {
            crate::log_warn!("Clipboard", "Could not start clipboard worker: {}", error);
        }
    }
}

#[cfg(test)]
pub(crate) fn internal_clipboard_text() -> String {
    INTERNAL_CLIPBOARD
        .lock()
        .expect("clipboard lock poisoned")
        .clone()
}

#[cfg(test)]
pub(crate) static CLIPBOARD_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn set_system_clipboard(text: &str) {
    let copied = write_to_command(&mut std::process::Command::new("wl-copy"), text)
        || write_to_command(
            std::process::Command::new("xclip").args(["-selection", "clipboard", "-in"]),
            text,
        )
        || write_to_command(
            std::process::Command::new("xsel").args(["--clipboard", "--input"]),
            text,
        );
    if !copied {
        crate::log_debug!(
            "Clipboard",
            "No supported system clipboard command succeeded"
        );
    }
}

#[cfg(target_os = "windows")]
fn set_system_clipboard(text: &str) {
    if !write_to_command(&mut std::process::Command::new("clip"), text) {
        let escaped = text.replace('\'', "''");
        if let Err(error) = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("Set-Clipboard -Value '{}'", escaped),
            ])
            .output()
        {
            crate::log_warn!("Clipboard", "Could not access system clipboard: {}", error);
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "windows"))]
fn write_to_command(command: &mut std::process::Command, text: &str) -> bool {
    use std::io::Write;

    let Ok(mut child) = command.stdin(std::process::Stdio::piped()).spawn() else {
        return false;
    };
    let Some(mut stdin) = child.stdin.take() else {
        return false;
    };
    if stdin.write_all(text.as_bytes()).is_err() {
        return false;
    }
    drop(stdin);
    child.wait().map(|status| status.success()).unwrap_or(false)
}
