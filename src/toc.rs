use crate::markdown::{LineType, RenderedLine};

#[derive(Clone, Debug)]
pub struct TocEntry {
    pub level: u8,
    pub text: String,
    /// Line index in the rendered output
    pub line_index: usize,
}

pub fn extract_toc(lines: &[RenderedLine]) -> Vec<TocEntry> {
    let mut entries = Vec::new();

    for (idx, rline) in lines.iter().enumerate() {
        if let LineType::Header(level) = rline.line_type {
            let text = rline.plain_text.trim().to_string();
            // Strip the heading prefix chars (█ ▓ ▒ ░)
            let text = text
                .trim_start_matches('█')
                .trim_start_matches('▓')
                .trim_start_matches('▒')
                .trim_start_matches('░')
                .trim()
                .to_string();

            if !text.is_empty() {
                entries.push(TocEntry {
                    level,
                    text,
                    line_index: idx,
                });
            }
        }
    }

    entries
}
