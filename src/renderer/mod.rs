//! Markdown 渲染引擎模块
//!
//! 使用 pulldown-cmark 解析器将 Markdown 文本转换为结构化块列表，
//! 再由 blocks.rs 渲染为 egui 界面元素。

mod blocks;
mod inline;

#[allow(unused_imports)]
pub use blocks::render_block;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// 渲染块 - pulldown-cmark 解析后的结构化块
#[derive(Debug, Clone)]
pub enum Block {
    /// 标题 { 级别, 文本 }
    Heading { level: u8, text: String },
    /// 段落 { 文本 }
    Paragraph { text: String },
    /// 代码块 { 语言, 代码内容 }
    CodeBlock { lang: String, code: String },
    /// 引用块 { 文本 }
    Quote { text: String },
    /// 列表 { 是否有序, 列表项 }
    List { ordered: bool, items: Vec<String> },
    /// 水平分割线
    Rule,
}

/// 使用 pulldown-cmark 解析 Markdown 文本为块列表
/// 启用删除线、表格、任务列表扩展
pub fn parse_blocks(content: &str) -> Vec<Block> {
    let opts = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(content, opts);

    let mut blocks = Vec::new();
    let mut current_text = String::new();
    let mut in_heading: Option<u8> = None;
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_content = String::new();
    let mut in_quote = false;
    let mut quote_text = String::new();
    let mut _in_list = false;
    let mut list_ordered = false;
    let mut list_items: Vec<String> = Vec::new();
    let mut current_item = String::new();
    let mut in_item = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = Some(level as u8);
                current_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = in_heading.take() {
                    blocks.push(Block::Heading {
                        level,
                        text: current_text.clone(),
                    });
                }
            }
            Event::Start(Tag::Paragraph) => {
                if !in_quote && !in_item {
                    current_text.clear();
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if in_quote {
                    quote_text.push_str(&current_text);
                    current_text.clear();
                } else if in_item {
                    current_item.push_str(&current_text);
                    current_text.clear();
                } else {
                    blocks.push(Block::Paragraph {
                        text: current_text.clone(),
                    });
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                    _ => String::new(),
                };
                code_content.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                blocks.push(Block::CodeBlock {
                    lang: code_lang.clone(),
                    code: code_content.clone(),
                });
            }
            Event::Start(Tag::BlockQuote(_)) => {
                in_quote = true;
                quote_text.clear();
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                in_quote = false;
                blocks.push(Block::Quote {
                    text: quote_text.clone(),
                });
            }
            Event::Start(Tag::List(first_item)) => {
                _in_list = true;
                list_ordered = first_item.is_some();
                list_items.clear();
            }
            Event::End(TagEnd::List(_)) => {
                _in_list = false;
                blocks.push(Block::List {
                    ordered: list_ordered,
                    items: list_items.clone(),
                });
            }
            Event::Start(Tag::Item) => {
                in_item = true;
                current_item.clear();
            }
            Event::End(TagEnd::Item) => {
                in_item = false;
                list_items.push(current_item.clone());
            }
            Event::Text(text) => {
                if in_code_block {
                    code_content.push_str(&text);
                } else if in_item {
                    current_item.push_str(&text);
                } else if in_quote {
                    quote_text.push_str(&text);
                } else {
                    current_text.push_str(&text);
                }
            }
            Event::Code(code) => {
                current_text.push('`');
                current_text.push_str(&code);
                current_text.push('`');
            }
            Event::SoftBreak | Event::HardBreak => {
                current_text.push('\n');
            }
            Event::Rule => {
                blocks.push(Block::Rule);
            }
            _ => {}
        }
    }
    blocks
}
