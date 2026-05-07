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
