//! Internationalization (i18n) and localization runtime subsystem.
//!
//! Provides zero-cost embedded translations, automatic fallback to English,
//! runtime language switching, and helper functions for formatted localized strings.

use std::collections::HashMap;
use std::sync::RwLock;

/// Supported game languages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    English,
    Polish,
}

impl Language {
    /// Human-readable native language name for UI selection.
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Polish => "Polski",
        }
    }

    /// ISO-like two-letter language code (e.g. "en", "pl").
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Polish => "pl",
        }
    }

    /// Converts code or index to `Language`.
    pub fn from_code(code: &str) -> Self {
        match code {
            "pl" => Language::Polish,
            _ => Language::English,
        }
    }

    /// Returns list of all available compiled languages.
    pub fn all() -> &'static [Language] {
        &[Language::English, Language::Polish]
    }
}

/// Global localization state manager.
struct I18nManager {
    current_language: Language,
    en_dict: HashMap<&'static str, &'static str>,
    pl_dict: HashMap<&'static str, &'static str>,
}

impl I18nManager {
    fn new() -> Self {
        let mut en_dict = HashMap::new();
        let mut pl_dict = HashMap::new();

        Self::parse_embedded_json(include_str!("../../locales/en.json"), &mut en_dict);
        Self::parse_embedded_json(include_str!("../../locales/pl.json"), &mut pl_dict);

        Self {
            current_language: Language::English,
            en_dict,
            pl_dict,
        }
    }

    /// Minimal zero-dependency JSON key-value string parser for embedded locales.
    fn parse_embedded_json(json: &'static str, dict: &mut HashMap<&'static str, &'static str>) {
        for line in json.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('"') {
                if let Some(colon_idx) = trimmed.find("\":") {
                    let key = &trimmed[1..colon_idx];
                    let rest = trimmed[colon_idx + 2..].trim();
                    if rest.starts_with('"') {
                        let end_quote = if rest.ends_with("\",") {
                            rest.len() - 2
                        } else if rest.ends_with('"') {
                            rest.len() - 1
                        } else {
                            rest.rfind('"').unwrap_or(rest.len())
                        };
                        if end_quote > 1 {
                            let val = &rest[1..end_quote];
                            dict.insert(key, val);
                        }
                    }
                }
            }
        }
    }

    fn translate(&self, key: &str) -> &'static str {
        let dict = match self.current_language {
            Language::English => &self.en_dict,
            Language::Polish => &self.pl_dict,
        };

        if let Some(&val) = dict.get(key) {
            return val;
        }

        // Fallback to English dictionary
        if let Some(&val) = self.en_dict.get(key) {
            return val;
        }

        ""
    }
}

static I18N: std::sync::OnceLock<RwLock<I18nManager>> = std::sync::OnceLock::new();

fn get_i18n() -> &'static RwLock<I18nManager> {
    I18N.get_or_init(|| RwLock::new(I18nManager::new()))
}

/// Translates a localization key using the active language.
///
/// Falls back to English if the key is missing in the target locale.
/// Returns empty string `""` if key is not found in any locale.
pub fn t(key: &str) -> &'static str {
    if let Ok(mgr) = get_i18n().read() {
        let res = mgr.translate(key);
        if !res.is_empty() {
            return res;
        }
    }
    ""
}

/// Translates a localization key, replacing `{}` placeholders with given `args`.
pub fn t_fmt(key: &str, args: &[&str]) -> String {
    let template = t(key);
    let mut result = String::with_capacity(template.len() + 16);
    let mut arg_idx = 0;
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' && chars.peek() == Some(&'}') {
            chars.next();
            if let Some(&arg) = args.get(arg_idx) {
                result.push_str(arg);
                arg_idx += 1;
            } else {
                result.push_str("{}");
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Changes the active game language.
pub fn set_language(lang: Language) {
    if let Ok(mut mgr) = get_i18n().write() {
        mgr.current_language = lang;
    }
}

/// Returns the currently active language.
pub fn current_language() -> Language {
    if let Ok(mgr) = get_i18n().read() {
        mgr.current_language
    } else {
        Language::English
    }
}

/// Translates a localization key for a specific explicit language.
pub fn translate_lang(lang: Language, key: &str) -> &'static str {
    if let Ok(mgr) = get_i18n().read() {
        match lang {
            Language::English => mgr.en_dict.get(key).copied().unwrap_or(""),
            Language::Polish => mgr.pl_dict.get(key).copied().or_else(|| mgr.en_dict.get(key).copied()).unwrap_or(""),
        }
    } else {
        ""
    }
}

/// Returns a slice of all supported languages.
pub fn available_languages() -> &'static [Language] {
    Language::all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_translation_and_formatting() {
        assert_eq!(translate_lang(Language::English, "menu.play"), "SINGLEPLAYER");
        assert_eq!(translate_lang(Language::English, "settings.on"), "ON");
        assert_eq!(translate_lang(Language::Polish, "menu.play"), "GRA JEDNOOSOBOWA");
        assert_eq!(translate_lang(Language::Polish, "settings.on"), "WLACZONE");

        set_language(Language::English);
        assert_eq!(t("menu.play"), "SINGLEPLAYER");
        assert_eq!(t_fmt("save_select.min_ago", &["5"]), "5 min ago");

        set_language(Language::Polish);
        assert_eq!(t("menu.play"), "GRA JEDNOOSOBOWA");
        assert_eq!(t_fmt("save_select.min_ago", &["5"]), "5 min temu");

        // Restore default English
        set_language(Language::English);
    }
}

