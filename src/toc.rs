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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::parse_markdown;

    #[test]
    fn extracts_headings_in_order_with_levels() {
        let src = "# First\n\n## Second\n\ntext\n\n### Third\n";
        let lines = parse_markdown(src, 80);
        let toc = extract_toc(&lines);
        assert_eq!(toc.len(), 3);
        assert_eq!(toc[0].level, 1);
        assert_eq!(toc[0].text, "First");
        assert_eq!(toc[1].level, 2);
        assert_eq!(toc[1].text, "Second");
        assert_eq!(toc[2].level, 3);
        assert_eq!(toc[2].text, "Third");
    }

    #[test]
    fn strips_rendered_heading_prefix_glyphs() {
        let lines = parse_markdown("# Hello\n", 80);
        let toc = extract_toc(&lines);
        assert_eq!(toc.len(), 1);
        // The "█ " prefix added during rendering must not leak into TOC text.
        assert_eq!(toc[0].text, "Hello");
    }

    #[test]
    fn no_headings_yields_empty_toc() {
        let lines = parse_markdown("just a paragraph\n", 80);
        assert!(extract_toc(&lines).is_empty());
    }

    #[test]
    fn line_index_points_at_a_header_line() {
        let src = "intro\n\n## Target\n";
        let lines = parse_markdown(src, 80);
        let toc = extract_toc(&lines);
        assert_eq!(toc.len(), 1);
        let idx = toc[0].line_index;
        assert!(matches!(lines[idx].line_type, LineType::Header(2)));
    }
}
