pub struct OutlineItem {
    pub level: u8,
    pub title: String,
    pub line: usize,
}

pub fn extract_outline(content: &str) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            if level <= 6 {
                let title = trimmed[level..].trim_start().to_string();
                if !title.is_empty() {
                    items.push(OutlineItem {
                        level: level as u8,
                        title,
                        line: line_idx,
                    });
                }
            }
        }
    }
    items
}
