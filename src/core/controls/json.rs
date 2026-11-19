//! Minimal zero-dependency JSON tokenizer and parser for `controls.json`.

#[derive(Debug, PartialEq, Clone)]
pub(super) enum JsonToken {
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Colon,
    Comma,
    StringVal(String),
}

#[derive(Debug, Clone)]
pub(super) enum JsonValue {
    Str(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

pub(super) fn tokenize_json(json: &str) -> Result<Vec<JsonToken>, String> {
    let mut tokens = Vec::new();
    let mut chars = json.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '{' => {
                chars.next();
                tokens.push(JsonToken::BraceOpen);
            }
            '}' => {
                chars.next();
                tokens.push(JsonToken::BraceClose);
            }
            '[' => {
                chars.next();
                tokens.push(JsonToken::BracketOpen);
            }
            ']' => {
                chars.next();
                tokens.push(JsonToken::BracketClose);
            }
            ':' => {
                chars.next();
                tokens.push(JsonToken::Colon);
            }
            ',' => {
                chars.next();
                tokens.push(JsonToken::Comma);
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                let mut escaped = false;
                while let Some(ch) = chars.next() {
                    if escaped {
                        s.push(ch);
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch == '"' {
                        break;
                    } else {
                        s.push(ch);
                    }
                }
                tokens.push(JsonToken::StringVal(s));
            }
            _ if c.is_whitespace() => {
                chars.next();
            }
            _ => {
                chars.next();
            }
        }
    }

    Ok(tokens)
}

pub(super) fn parse_json_object(tokens: &[JsonToken]) -> Result<Vec<(String, JsonValue)>, String> {
    let mut idx = 0;
    if idx < tokens.len() && tokens[idx] == JsonToken::BraceOpen {
        idx += 1;
    }

    let mut result = Vec::new();
    while idx < tokens.len() {
        if tokens[idx] == JsonToken::BraceClose {
            break;
        }
        if let JsonToken::StringVal(ref key) = tokens[idx] {
            let key = key.clone();
            idx += 1;
            if idx >= tokens.len() || tokens[idx] != JsonToken::Colon {
                return Err("Expected colon after key".to_string());
            }
            idx += 1;
            let (val, next_idx) = parse_json_value(tokens, idx)?;
            result.push((key, val));
            idx = next_idx;
            if idx < tokens.len() && tokens[idx] == JsonToken::Comma {
                idx += 1;
            }
        } else {
            idx += 1;
        }
    }

    Ok(result)
}

fn parse_json_value(tokens: &[JsonToken], start: usize) -> Result<(JsonValue, usize), String> {
    if start >= tokens.len() {
        return Err("Unexpected end of tokens".to_string());
    }

    match &tokens[start] {
        JsonToken::StringVal(s) => Ok((JsonValue::Str(s.clone()), start + 1)),
        JsonToken::BracketOpen => {
            let mut arr = Vec::new();
            let mut idx = start + 1;
            while idx < tokens.len() {
                if tokens[idx] == JsonToken::BracketClose {
                    idx += 1;
                    break;
                }
                let (val, next_idx) = parse_json_value(tokens, idx)?;
                arr.push(val);
                idx = next_idx;
                if idx < tokens.len() && tokens[idx] == JsonToken::Comma {
                    idx += 1;
                }
            }
            Ok((JsonValue::Array(arr), idx))
        }
        JsonToken::BraceOpen => {
            let mut obj = Vec::new();
            let mut idx = start + 1;
            while idx < tokens.len() {
                if tokens[idx] == JsonToken::BraceClose {
                    idx += 1;
                    break;
                }
                if let JsonToken::StringVal(ref k) = tokens[idx] {
                    let k = k.clone();
                    idx += 1;
                    if idx >= tokens.len() || tokens[idx] != JsonToken::Colon {
                        return Err("Expected colon in object".to_string());
                    }
                    idx += 1;
                    let (val, next_idx) = parse_json_value(tokens, idx)?;
                    obj.push((k, val));
                    idx = next_idx;
                    if idx < tokens.len() && tokens[idx] == JsonToken::Comma {
                        idx += 1;
                    }
                } else {
                    idx += 1;
                }
            }
            Ok((JsonValue::Object(obj), idx))
        }
        _ => Err(format!("Unexpected token at {}", start)),
    }
}

