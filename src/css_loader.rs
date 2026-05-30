use std::collections::HashMap;
use std::path::Path;

use egui::Color32;
use crate::theme::Theme;

pub fn load_theme_from_css(path: &Path) -> Option<Theme> {
    let content = std::fs::read_to_string(path).ok()?;
    let rules = parse_css(&content);
    let theme = apply_css_to_theme(rules);
    Some(theme)
}

pub fn debug_theme(theme: &Theme) -> String {
    format!(
        "Theme loaded:\n\
         base.background: {:?}\n\
         base.text: {:?}\n\
         font.line_height: {}\n\
         heading.separator[0]: {:?}\n\
         heading.separator[1]: {:?}\n\
         code.inline_rounding: {}\n\
         code.block_bg: {:?}\n\
         code.block_text: {:?}\n\
         code.block_padding: {}\n\
         quote.bar_color: {:?}\n\
         rule.color: {:?}\n\
         link.color: {:?}\n\
         table.header_text: {:?}\n\
         table.cell_padding: {}",
        theme.base.background,
        theme.base.text,
        theme.font.line_height,
        theme.heading.separator_colors[0],
        theme.heading.separator_colors[1],
        theme.code.inline_rounding,
        theme.code.block_bg,
        theme.code.block_text,
        theme.code.block_padding,
        theme.quote.bar_color,
        theme.rule.color,
        theme.link.color,
        theme.table.header_text,
        theme.table.cell_padding,
    )
}

struct CssRule {
    properties: HashMap<String, String>,
}

fn parse_css(content: &str) -> HashMap<String, CssRule> {
    let mut rules: HashMap<String, CssRule> = HashMap::new();
    let mut chars = content.chars().peekable();

    while chars.peek().is_some() {
        skip_whitespace_and_comments(&mut chars);
        let selector = read_until(&mut chars, '{');
        if selector.is_empty() {
            break;
        }
        let body = read_until(&mut chars, '}');
        let selector = selector.trim().to_string();
        let properties = parse_properties(&body);
        rules.entry(selector.clone())
            .and_modify(|r| r.properties.extend(properties.clone()))
            .or_insert(CssRule { properties });
    }
    rules
}

fn skip_whitespace_and_comments(chars: &mut std::iter::Peekable<std::str::Chars>) {
    loop {
        match chars.peek() {
            Some(&c) if c.is_whitespace() => { chars.next(); }
            Some(&'/') => {
                let mut clone = chars.clone();
                clone.next();
                if clone.peek() == Some(&'*') {
                    chars.next(); chars.next();
                    loop {
                        match chars.next() {
                            Some('*') if chars.peek() == Some(&'/') => {
                                chars.next();
                                break;
                            }
                            None => break,
                            _ => {}
                        }
                    }
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
}

fn read_until(chars: &mut std::iter::Peekable<std::str::Chars>, end: char) -> String {
    let mut result = String::new();
    while let Some(&c) = chars.peek() {
        if c == end {
            chars.next();
            break;
        }
        result.push(c);
        chars.next();
    }
    result
}

fn parse_properties(body: &str) -> HashMap<String, String> {
    let mut props = HashMap::new();
    // 先去掉注释
    let body = remove_comments(body);
    for decl in body.split(';') {
        let decl = decl.trim();
        if decl.is_empty() { continue; }
        if let Some((key, val)) = decl.split_once(':') {
            let key = key.trim().to_lowercase();
            let val = val.trim()
                .replace("!important", "").trim().to_string();
            if !key.is_empty() {
                props.insert(key, val);
            }
        }
    }
    props
}

fn remove_comments(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            loop {
                match chars.next() {
                    Some('*') if chars.peek() == Some(&'/') => {
                        chars.next();
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn parse_color(val: &str) -> Option<Color32> {
    let val = val.trim();
    if val.starts_with('#') {
        let hex = &val[1..];
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Color32::from_rgb(r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color32::from_rgb(r, g, b))
            }
            _ => None,
        }
    } else if val.starts_with("rgb(") {
        let inner = val.strip_prefix("rgb(")?.strip_suffix(')')?;
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() != 3 { return None; }
        let r = parts[0].trim().parse::<u8>().ok()?;
        let g = parts[1].trim().parse::<u8>().ok()?;
        let b = parts[2].trim().parse::<u8>().ok()?;
        Some(Color32::from_rgb(r, g, b))
    } else if val.starts_with("rgba(") {
        let inner = val.strip_prefix("rgba(")?.strip_suffix(')')?;
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() != 4 { return None; }
        let r = parts[0].trim().parse::<u8>().ok()?;
        let g = parts[1].trim().parse::<u8>().ok()?;
        let b = parts[2].trim().parse::<u8>().ok()?;
        let a = (parts[3].trim().parse::<f32>().ok()? * 255.0) as u8;
        Some(Color32::from_rgba_premultiplied(r, g, b, a))
    } else {
        None
    }
}

fn parse_border_color(val: &str) -> Option<Color32> {
    // "0.2rem solid #2995d9" → extract color
    for part in val.split_whitespace() {
        if let Some(c) = parse_color(part) {
            return Some(c);
        }
    }
    None
}

fn parse_padding_first(val: &str) -> Option<f32> {
    val.split_whitespace()
        .next()?
        .trim_end_matches("px")
        .parse::<f32>()
        .ok()
}

fn get_prop<'a>(
    rules: &'a HashMap<String, CssRule>,
    selector: &str,
    prop: &str,
) -> Option<&'a String> {
    rules.get(selector)?.properties.get(prop)
}

fn find_rule<'a>(
    rules: &'a HashMap<String, CssRule>,
    needle: &str,
) -> Option<&'a CssRule> {
    // 先尝试精确匹配
    if let Some(rule) = rules.get(needle) {
        return Some(rule);
    }
    // 回退：找包含 needle 的最短选择器（最精确）
    rules.iter()
        .filter(|(k, _)| k.contains(needle))
        .min_by_key(|(k, _)| k.len())
        .map(|(_, v)| v)
}

fn apply_css_to_theme(rules: HashMap<String, CssRule>) -> Theme {
    let mut theme = Theme::light();

    // .vditor-reset → base colors
    if let Some(rule) = find_rule(&rules, ".vditor-reset") {
        if let Some(c) = rule.properties.get("color").and_then(|v| parse_color(v)) {
            theme.base.text = c;
            theme.heading.colors = [c; 6];
            theme.code.block_text = c;
        }
        if let Some(c) = rule.properties.get("background-color")
            .and_then(|v| parse_color(v))
        {
            theme.base.background = c;
        }
        if let Some(v) = rule.properties.get("line-height") {
            if let Ok(lh) = v.parse::<f32>() {
                theme.font.line_height = lh;
            }
        }
    }

    // h1 border-bottom
    if let Some(rule) = find_rule(&rules, "h1") {
        if let Some(c) = rule.properties.get("border-bottom")
            .and_then(|v| parse_border_color(v))
        {
            theme.heading.separator_colors[0] = Some(c);
        }
    }

    // h2 border-bottom
    if let Some(rule) = find_rule(&rules, "h2") {
        if let Some(c) = rule.properties.get("border-bottom")
            .and_then(|v| parse_border_color(v))
        {
            theme.heading.separator_colors[1] = Some(c);
        }
    }

    // code (inline)
    if let Some(rule) = find_rule(&rules, "reset code") {
        if let Some(v) = rule.properties.get("border-radius") {
            if let Some(r) = parse_padding_first(v) {
                theme.code.inline_rounding = r;
            }
        }
    }

    // pre (code block)
    if let Some(rule) = find_rule(&rules, "reset pre") {
        if let Some(c) = rule.properties.get("background-color")
            .and_then(|v| parse_color(v))
        {
            theme.code.block_bg = c;
        }
        if let Some(c) = rule.properties.get("color")
            .and_then(|v| parse_color(v))
        {
            theme.code.block_text = c;
        }
        if let Some(v) = rule.properties.get("padding") {
            if let Some(p) = parse_padding_first(v) {
                theme.code.block_padding = p;
            }
        }
    }

    // hr
    if let Some(rule) = find_rule(&rules, "reset hr") {
        if let Some(c) = rule.properties.get("border-bottom")
            .and_then(|v| parse_border_color(v))
        {
            theme.rule.color = c;
        }
    }

    // a (link)
    if let Some(rule) = find_rule(&rules, "reset a") {
        if let Some(c) = rule.properties.get("color")
            .and_then(|v| parse_color(v))
        {
            theme.link.color = c;
        }
    }

    // table th
    if let Some(rule) = find_rule(&rules, "table th") {
        if let Some(c) = rule.properties.get("color")
            .and_then(|v| parse_color(v))
        {
            theme.table.header_text = c;
        }
    }

    // table td padding
    if let Some(rule) = find_rule(&rules, "table td") {
        if let Some(v) = rule.properties.get("padding") {
            if let Some(p) = parse_padding_first(v) {
                theme.table.cell_padding = p;
            }
        }
    }

    theme
}