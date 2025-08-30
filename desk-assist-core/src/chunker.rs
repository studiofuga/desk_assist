pub struct TextChunker;

impl TextChunker {
    const DEFAULT_CHUNK_SIZE: usize = 1000;
    const DEFAULT_OVERLAP: usize = 200;

    pub fn chunk_text(text: &str) -> Vec<String> {
        Self::chunk_text_with_options(text, Self::DEFAULT_CHUNK_SIZE, Self::DEFAULT_OVERLAP)
    }

    pub fn chunk_text_with_options(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
        if text.len() <= chunk_size {
            return vec![text.to_string()];
        }

        let mut chunks = Vec::new();
        let mut start = 0;
        
        while start < text.len() {
            let end = std::cmp::min(start + chunk_size, text.len());
            let mut chunk_end = end;
            
            if end < text.len() {
                if let Some(last_sentence) = Self::find_sentence_boundary(&text[start..end]) {
                    chunk_end = start + last_sentence;
                } else if let Some(last_word) = Self::find_word_boundary(&text[start..end]) {
                    chunk_end = start + last_word;
                }
            }
            
            let chunk = text[start..chunk_end].trim().to_string();
            if !chunk.is_empty() {
                chunks.push(chunk);
            }
            
            if chunk_end >= text.len() {
                break;
            }
            
            start = if chunk_end > overlap {
                chunk_end - overlap
            } else {
                chunk_end
            };
        }
        
        chunks
    }
    
    fn find_sentence_boundary(text: &str) -> Option<usize> {
        let sentence_endings = ['.', '!', '?'];
        
        for &ending in &sentence_endings {
            if let Some(pos) = text.rfind(ending) {
                if pos + 1 < text.len() {
                    return Some(pos + 1);
                }
            }
        }
        
        None
    }
    
    fn find_word_boundary(text: &str) -> Option<usize> {
        text.rfind(char::is_whitespace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_short_text() {
        let text = "This is a short text.";
        let chunks = TextChunker::chunk_text(text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_chunk_long_text() {
        let text = "A".repeat(2000);
        let chunks = TextChunker::chunk_text_with_options(&text, 500, 50);
        assert!(chunks.len() > 1);
        
        for chunk in &chunks {
            assert!(chunk.len() <= 500);
        }
    }
}