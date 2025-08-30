use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

pub struct TextExtractor;

impl TextExtractor {
    pub fn extract_text(file_path: &str) -> Result<String> {
        let path = Path::new(file_path);
        
        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", file_path));
        }

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "txt" | "md" => Self::extract_plain_text(file_path),
            "pdf" => Self::extract_pdf_text(file_path),
            "html" | "htm" => Self::extract_html_text(file_path),
            _ => Err(anyhow!("Unsupported file type: {}", extension)),
        }
    }

    fn extract_plain_text(file_path: &str) -> Result<String> {
        let content = fs::read_to_string(file_path)?;
        Ok(content)
    }

    fn extract_pdf_text(file_path: &str) -> Result<String> {
        let doc = lopdf::Document::load(file_path)?;
        let mut text = String::new();

        for page_id in doc.get_pages() {
            if let Ok(page_text) = doc.extract_text(&[page_id.0]) {
                text.push_str(&page_text);
                text.push('\n');
            }
        }

        if text.trim().is_empty() {
            return Err(anyhow!("No text extracted from PDF"));
        }

        Ok(text)
    }

    fn extract_html_text(file_path: &str) -> Result<String> {
        let html_content = fs::read_to_string(file_path)?;
        let text = html2text::from_read(html_content.as_bytes(), 80)?;
        Ok(text)
    }

    pub fn extract_markdown_text(file_path: &str) -> Result<String> {
        let markdown_content = fs::read_to_string(file_path)?;
        let parser = pulldown_cmark::Parser::new(&markdown_content);
        
        let mut text = String::new();
        for event in parser {
            match event {
                pulldown_cmark::Event::Text(t) => text.push_str(&t),
                pulldown_cmark::Event::Code(t) => text.push_str(&t),
                pulldown_cmark::Event::SoftBreak | pulldown_cmark::Event::HardBreak => {
                    text.push(' ');
                }
                _ => {}
            }
        }
        
        Ok(text)
    }
}