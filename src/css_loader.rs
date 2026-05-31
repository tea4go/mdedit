//! CSS 主题加载器 - 从 CSS 文件解析并应用 Markdown 编辑器主题
//!
//! 支持解析 vditor 风格的 CSS 主题文件，提取颜色、圆角、间距等样式属性，
//! 映射到内部 Theme 结构体。支持 #hex、rgb()、rgba() 颜色格式。

use std::collections::HashMap;
use std::path::Path;

use egui::Color32;
use crate::theme::Theme;

/// 从 CSS 文件加载主题
/// 读取文件内容 → 解析 CSS 规则 → 应用到 Theme 结构体
pub fn load_theme_from_css(path: &Path) -> Option<Theme> {
    let content = std::fs::read_to_string(path).ok()?;
    let rules = parse_css(&content);
    let theme = apply_css_to_theme(rules);
    Some(theme)
}

/// 生成主题调试信息（中文化输出）
pub fn debug_theme(theme: &Theme) -> String {
    format!(
        "主题已加载:\n\
         基础.背景色: {:?}\n\
         基础.文字色: {:?}\n\
         字体.行高: {}\n\
         标题.分隔线[0]: {:?}\n\
         标题.分隔线[1]: {:?}\n\
         代码.行内圆角: {}\n\
         代码.块背景: {:?}\n\
         代码.块文字: {:?}\n\
         代码.块内边距: {}\n\
         引用.边条色: {:?}\n\
         分割线.颜色: {:?}\n\
         链接.颜色: {:?}\n\
         表格.头部文字: {:?}\n\
         表格.单元格内边距: {}",
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

/// CSS 规则 - 存储选择器对应的属性键值对
struct CssRule {
    properties: HashMap<String, String>,
}

/// 解析 CSS 文本为规则映射（选择器 → 属性集合）
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
        // 同一选择器的属性合并
        rules.entry(selector.clone())
            .and_modify(|r| r.properties.extend(properties.clone()))
            .or_insert(CssRule { properties });
    }
    rules
}

/// 跳过空白字符和 /* */ 注释
fn skip_whitespace_and_comments(chars: &mut std::iter::Peekable<std::str::Chars>) {
    loop {
        match chars.peek() {
            Some(&c) if c.is_whitespace() => { chars.next(); }
            Some(&'/') => {
                let mut clone = chars.clone();
                clone.next();
                if clone.peek() == Some(&'*') {
                    // 发现注释开始，跳过整个注释块
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

/// 读取字符直到遇到指定结束符，返回已读取的内容
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

/// 解析 CSS 属性声明块为键值对
/// 自动移除注释和 !important 标记
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

/// 移除 CSS 文本中的 /* */ 注释
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

/// 解析 CSS 颜色值为 egui Color32
/// 支持 #hex(3/6/8位)、rgb()、rgba() 格式
fn parse_color(val: &str) -> Option<Color32> {
    let val = val.trim();
    if val.starts_with('#') {
        let hex = &val[1..];
        match hex.len() {
            // #RGB 简写格式
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Color32::from_rgb(r, g, b))
            }
            // #RRGGBB 标准格式
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color32::from_rgb(r, g, b))
            }
            _ => None,
        }
    } else if val.starts_with("rgb(") {
        // rgb(R, G, B) 格式
        let inner = val.strip_prefix("rgb(")?.strip_suffix(')')?;
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() != 3 { return None; }
        let r = parts[0].trim().parse::<u8>().ok()?;
        let g = parts[1].trim().parse::<u8>().ok()?;
        let b = parts[2].trim().parse::<u8>().ok()?;
        Some(Color32::from_rgb(r, g, b))
    } else if val.starts_with("rgba(") {
        // rgba(R, G, B, A) 格式
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

/// 从 border 属性值中提取颜色
/// 例如 "0.2rem solid #2995d9" → 提取 #2995d9
fn parse_border_color(val: &str) -> Option<Color32> {
    for part in val.split_whitespace() {
        if let Some(c) = parse_color(part) {
            return Some(c);
        }
    }
    None
}

/// 从 padding 属性值中提取第一个数值（px）
fn parse_padding_first(val: &str) -> Option<f32> {
    val.split_whitespace()
        .next()?
        .trim_end_matches("px")
        .parse::<f32>()
        .ok()
}

/// 从规则映射中获取指定选择器和属性名的值
fn get_prop<'a>(
    rules: &'a HashMap<String, CssRule>,
    selector: &str,
    prop: &str,
) -> Option<&'a String> {
    rules.get(selector)?.properties.get(prop)
}

/// 在规则映射中查找匹配的 CSS 规则
/// 先尝试精确匹配选择器，再回退到包含查找（选最短的，最精确）
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

/// 将解析的 CSS 规则应用到 Theme 结构体
/// 按照 vditor CSS 的选择器命名约定映射各样式属性
fn apply_css_to_theme(rules: HashMap<String, CssRule>) -> Theme {
    let mut theme = Theme::light();

    // .vditor-reset → 基础颜色
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

    // h1 下边框
    if let Some(rule) = find_rule(&rules, "h1") {
        if let Some(c) = rule.properties.get("border-bottom")
            .and_then(|v| parse_border_color(v))
        {
            theme.heading.separator_colors[0] = Some(c);
        }
    }

    // h2 下边框
    if let Some(rule) = find_rule(&rules, "h2") {
        if let Some(c) = rule.properties.get("border-bottom")
            .and_then(|v| parse_border_color(v))
        {
            theme.heading.separator_colors[1] = Some(c);
        }
    }

    // code（行内代码）
    if let Some(rule) = find_rule(&rules, "reset code") {
        if let Some(v) = rule.properties.get("border-radius") {
            if let Some(r) = parse_padding_first(v) {
                theme.code.inline_rounding = r;
            }
        }
    }

    // pre（代码块）
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

    // hr（水平分割线）
    if let Some(rule) = find_rule(&rules, "reset hr") {
        if let Some(c) = rule.properties.get("border-bottom")
            .and_then(|v| parse_border_color(v))
        {
            theme.rule.color = c;
        }
    }

    // a（链接）
    if let Some(rule) = find_rule(&rules, "reset a") {
        if let Some(c) = rule.properties.get("color")
            .and_then(|v| parse_color(v))
        {
            theme.link.color = c;
        }
    }

    // table th（表头）
    if let Some(rule) = find_rule(&rules, "table th") {
        if let Some(c) = rule.properties.get("color")
            .and_then(|v| parse_color(v))
        {
            theme.table.header_text = c;
        }
    }

    // table td（表格单元格）内边距
    if let Some(rule) = find_rule(&rules, "table td") {
        if let Some(v) = rule.properties.get("padding") {
            if let Some(p) = parse_padding_first(v) {
                theme.table.cell_padding = p;
            }
        }
    }

    theme
}