// Clipboard helper supporting Linux (xclip / wl-paste / wl-copy) and fallback internal buffer

static INTERNAL_CLIPBOARD: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

pub fn get_clipboard() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(out) = std::process::Command::new("wl-paste")
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

    #[cfg(target_os = "linux")]
    {
        use std::io::Write;
        if let Ok(mut child) = std::process::Command::new("wl-copy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        } else if let Ok(mut child) = std::process::Command::new("xclip")
            .args(["-selection", "clipboard", "-in"])
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
