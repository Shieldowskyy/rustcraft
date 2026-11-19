//! Bez-zależnościowy parser JSON z obsługą jednoliniowych komentarzy `//` (`JsonNode`, `parse_json_value_root`).

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum JsonNode {
    Str(String),
    Number(f64),
    Bool(bool),
    Null,
    Array(Vec<JsonNode>),
    Object(HashMap<String, JsonNode>),
}

impl JsonNode {
    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            JsonNode::Str(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn as_f32(&self) -> Option<f32> {
        match self {
            JsonNode::Number(n) => Some(*n as f32),
            _ => None,
        }
    }

    pub(crate) fn as_u8(&self) -> Option<u8> {
        match self {
            JsonNode::Number(n) => Some(*n as u8),
            _ => None,
        }
    }

    pub(crate) fn as_bool(&self) -> Option<bool> {
        match self {
            JsonNode::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub(crate) fn as_array(&self) -> Option<&[JsonNode]> {
        match self {
            JsonNode::Array(a) => Some(a),
            _ => None,
        }
    }

    pub(crate) fn as_object(&self) -> Option<&HashMap<String, JsonNode>> {
        match self {
            JsonNode::Object(o) => Some(o),
            _ => None,
        }
    }

    pub(crate) fn as_color3(&self) -> Option<[f32; 3]> {
        match self {
            JsonNode::Array(arr) if arr.len() >= 3 => {
                let r = arr[0].as_f32().unwrap_or(0.0);
                let g = arr[1].as_f32().unwrap_or(0.0);
                let b = arr[2].as_f32().unwrap_or(0.0);
                Some([r, g, b])
            }
            JsonNode::Str(hex) if hex.starts_with('#') && hex.len() == 7 => {
                let r = u8::from_str_radix(&hex[1..3], 16).ok()? as f32 / 255.0;
                let g = u8::from_str_radix(&hex[3..5], 16).ok()? as f32 / 255.0;
                let b = u8::from_str_radix(&hex[5..7], 16).ok()? as f32 / 255.0;
                Some([r, g, b])
            }
            _ => None,
        }
    }
}

pub(crate) fn parse_json_value_root(input: &str) -> Result<JsonNode, String> {
    let mut cleaned = String::new();
    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        cleaned.push_str(line);
        cleaned.push('\n');
    }

    let chars: Vec<char> = cleaned.chars().collect();
    let mut idx = 0;
    skip_whitespace(&chars, &mut idx);
    let (node, _) = parse_node(&chars, idx)?;
    Ok(node)
}

fn skip_whitespace(chars: &[char], idx: &mut usize) {
    while *idx < chars.len() && chars[*idx].is_whitespace() {
        *idx += 1;
    }
}

fn parse_node(chars: &[char], mut idx: usize) -> Result<(JsonNode, usize), String> {
    skip_whitespace(chars, &mut idx);
    if idx >= chars.len() {
        return Err("Unexpected end of JSON input".to_string());
    }

    match chars[idx] {
        '{' => parse_object(chars, idx),
        '[' => parse_array(chars, idx),
        '"' => {
            let (s, next) = parse_string(chars, idx)?;
            Ok((JsonNode::Str(s), next))
        }
        't' | 'f' => parse_bool(chars, idx),
        'n' => parse_null(chars, idx),
        '-' | '0'..='9' => parse_number(chars, idx),
        other => Err(format!("Unexpected character '{}' at offset {}", other, idx)),
    }
}

fn parse_object(chars: &[char], mut idx: usize) -> Result<(JsonNode, usize), String> {
    idx += 1;
    let mut map = HashMap::new();

    loop {
        skip_whitespace(chars, &mut idx);
        if idx >= chars.len() {
            return Err("Unclosed object".to_string());
        }
        if chars[idx] == '}' {
            idx += 1;
            break;
        }

        if chars[idx] != '"' {
            return Err(format!("Expected string key in object at offset {}", idx));
        }
        let (key, next_after_key) = parse_string(chars, idx)?;
        idx = next_after_key;

        skip_whitespace(chars, &mut idx);
        if idx >= chars.len() || chars[idx] != ':' {
            return Err(format!(
                "Expected ':' after key '{}' at offset {}",
                key, idx
            ));
        }
        idx += 1;

        let (val, next_after_val) = parse_node(chars, idx)?;
        idx = next_after_val;
        map.insert(key, val);

        skip_whitespace(chars, &mut idx);
        if idx < chars.len() && chars[idx] == ',' {
            idx += 1;
        } else if idx < chars.len() && chars[idx] == '}' {
            idx += 1;
            break;
        }
    }

    Ok((JsonNode::Object(map), idx))
}

fn parse_array(chars: &[char], mut idx: usize) -> Result<(JsonNode, usize), String> {
    idx += 1;
    let mut arr = Vec::new();

    loop {
        skip_whitespace(chars, &mut idx);
        if idx >= chars.len() {
            return Err("Unclosed array".to_string());
        }
        if chars[idx] == ']' {
            idx += 1;
            break;
        }

        let (val, next_after_val) = parse_node(chars, idx)?;
        idx = next_after_val;
        arr.push(val);

        skip_whitespace(chars, &mut idx);
        if idx < chars.len() && chars[idx] == ',' {
            idx += 1;
        } else if idx < chars.len() && chars[idx] == ']' {
            idx += 1;
            break;
        }
    }

    Ok((JsonNode::Array(arr), idx))
}

fn parse_string(chars: &[char], mut idx: usize) -> Result<(String, usize), String> {
    idx += 1;
    let mut s = String::new();
    while idx < chars.len() {
        let c = chars[idx];
        if c == '"' {
            idx += 1;
            return Ok((s, idx));
        } else if c == '\\' {
            idx += 1;
            if idx >= chars.len() {
                return Err("Dangling escape in string".to_string());
            }
            match chars[idx] {
                '"' => s.push('"'),
                '\\' => s.push('\\'),
                '/' => s.push('/'),
                'b' => s.push('\x08'),
                'f' => s.push('\x0c'),
                'n' => s.push('\n'),
                'r' => s.push('\r'),
                't' => s.push('\t'),
                _ => s.push(chars[idx]),
            }
        } else {
            s.push(c);
        }
        idx += 1;
    }
    Err("Unclosed string literal".to_string())
}

fn parse_bool(chars: &[char], idx: usize) -> Result<(JsonNode, usize), String> {
    let rem: String = chars[idx..].iter().take(5).collect();
    if rem.starts_with("true") {
        Ok((JsonNode::Bool(true), idx + 4))
    } else if rem.starts_with("false") {
        Ok((JsonNode::Bool(false), idx + 5))
    } else {
        Err(format!("Invalid boolean token at offset {}", idx))
    }
}

fn parse_null(chars: &[char], idx: usize) -> Result<(JsonNode, usize), String> {
    let rem: String = chars[idx..].iter().take(4).collect();
    if rem.starts_with("null") {
        Ok((JsonNode::Null, idx + 4))
    } else {
        Err(format!("Invalid token at offset {}", idx))
    }
}

fn parse_number(chars: &[char], mut idx: usize) -> Result<(JsonNode, usize), String> {
    let start = idx;
    if chars[idx] == '-' {
        idx += 1;
    }
    while idx < chars.len()
        && (chars[idx].is_ascii_digit()
            || chars[idx] == '.'
            || chars[idx] == 'e'
            || chars[idx] == 'E'
            || chars[idx] == '+'
            || chars[idx] == '-')
    {
        idx += 1;
    }
    let s: String = chars[start..idx].iter().collect();
    let num = s
        .parse::<f64>()
        .map_err(|e| format!("Bad number '{}': {}", s, e))?;
    Ok((JsonNode::Number(num), idx))
}
