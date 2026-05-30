use crate::markdown::RenderedLine;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct SearchMatch {
    pub line_index: usize,
    pub start: usize,
    pub end: usize,
}

pub fn find_matches(lines: &[RenderedLine], query: &str) -> Vec<SearchMatch> {
    if query.is_empty() {
        return Vec::new();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    for (line_idx, rline) in lines.iter().enumerate() {
        let plain_lower = rline.plain_text.to_lowercase();
        let mut search_from = 0;
        while let Some(pos) = plain_lower[search_from..].find(&query_lower) {
            let abs_pos = search_from + pos;
            matches.push(SearchMatch {
                line_index: line_idx,
                start: abs_pos,
                end: abs_pos + query.len(),
            });
            search_from = abs_pos + 1;
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::parse_markdown;

    #[test]
    fn empty_query_returns_no_matches() {
        let lines = parse_markdown("some content here\n", 80);
        assert!(find_matches(&lines, "").is_empty());
    }

    #[test]
    fn search_is_case_insensitive() {
        let lines = parse_markdown("The Quick Brown Fox\n", 80);
        let matches = find_matches(&lines, "quick");
        assert_eq!(matches.len(), 1);
        // "Quick" starts at column 4 in "The Quick Brown Fox".
        assert_eq!(matches[0].start, 4);
        assert_eq!(matches[0].end, 9);
    }

    #[test]
    fn search_finds_multiple_occurrences_on_one_line() {
        let lines = parse_markdown("aa aa aa\n", 80);
        let matches = find_matches(&lines, "aa");
        assert_eq!(matches.len(), 3);
        assert!(matches.iter().all(|m| m.line_index == 0));
    }

    #[test]
    fn search_missing_term_returns_empty() {
        let lines = parse_markdown("hello world\n", 80);
        assert!(find_matches(&lines, "zzz").is_empty());
    }
}
