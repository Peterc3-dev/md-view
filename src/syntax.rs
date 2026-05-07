use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

/// Simple keyword-based syntax highlighting for common languages
pub fn highlight<'a>(line: &str, lang: Option<&str>) -> Vec<Span<'static>> {
    let lang = match lang {
        Some(l) => l.to_lowercase(),
        None => return vec![Span::styled(line.to_string(), Style::default().fg(Color::Cyan))],
    };

    match lang.as_str() {
        "rust" | "rs" => highlight_rust(line),
        "python" | "py" => highlight_python(line),
        "javascript" | "js" | "typescript" | "ts" => highlight_js(line),
        "bash" | "sh" | "shell" | "zsh" => highlight_shell(line),
        "go" | "golang" => highlight_go(line),
        "c" | "cpp" | "c++" | "h" | "hpp" => highlight_c(line),
        "toml" => highlight_toml(line),
        "yaml" | "yml" => highlight_yaml(line),
        "json" => highlight_json(line),
        "sql" => highlight_sql(line),
        _ => vec![Span::styled(line.to_string(), Style::default().fg(Color::Cyan))],
    }
}

fn keyword_style() -> Style {
    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
}

fn string_style() -> Style {
    Style::default().fg(Color::Green)
}

fn comment_style() -> Style {
    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
}

fn number_style() -> Style {
    Style::default().fg(Color::Yellow)
}

fn type_style() -> Style {
    Style::default().fg(Color::LightCyan)
}

fn default_style() -> Style {
    Style::default().fg(Color::Cyan)
}

fn function_style() -> Style {
    Style::default().fg(Color::LightBlue)
}

/// Tokenize a line into spans with syntax-aware coloring
fn highlight_with_rules(line: &str, keywords: &[&str], types: &[&str], comment_prefix: &str) -> Vec<Span<'static>> {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];

    // Check for comment line
    if !comment_prefix.is_empty() && trimmed.starts_with(comment_prefix) {
        return vec![
            Span::styled(indent.to_string(), default_style()),
            Span::styled(trimmed.to_string(), comment_style()),
        ];
    }

    let mut spans = Vec::new();
    if !indent.is_empty() {
        spans.push(Span::styled(indent.to_string(), default_style()));
    }

    // Simple word-by-word tokenization
    let mut chars = trimmed.char_indices().peekable();
    let mut current_word = String::new();
    let mut in_string = false;
    let mut string_char = '"';
    let mut string_start = String::new();

    while let Some(&(_, ch)) = chars.peek() {
        if in_string {
            current_word.push(ch);
            chars.next();
            if ch == string_char {
                spans.push(Span::styled(
                    format!("{}{}", string_start, current_word),
                    string_style(),
                ));
                string_start.clear();
                current_word.clear();
                in_string = false;
            }
            continue;
        }

        if ch == '"' || ch == '\'' {
            // Flush current word
            if !current_word.is_empty() {
                spans.push(classify_word(&current_word, keywords, types));
                current_word.clear();
            }
            in_string = true;
            string_char = ch;
            string_start = ch.to_string();
            chars.next();
            continue;
        }

        if ch.is_alphanumeric() || ch == '_' {
            current_word.push(ch);
            chars.next();
        } else {
            if !current_word.is_empty() {
                // Check if this is a function call (followed by '(')
                if ch == '(' && !keywords.contains(&current_word.as_str()) && !types.contains(&current_word.as_str()) {
                    spans.push(Span::styled(current_word.clone(), function_style()));
                } else {
                    spans.push(classify_word(&current_word, keywords, types));
                }
                current_word.clear();
            }
            // Check for inline comment
            let remaining: String = trimmed[chars.peek().map(|&(i, _)| i).unwrap_or(trimmed.len())..].to_string();
            if !comment_prefix.is_empty() && remaining.starts_with(comment_prefix) {
                spans.push(Span::styled(
                    format!("{}{}", ch, &remaining[..]),
                    comment_style(),
                ));
                return spans;
            }
            spans.push(Span::styled(ch.to_string(), default_style()));
            chars.next();
        }
    }

    // Handle remaining
    if in_string {
        spans.push(Span::styled(
            format!("{}{}", string_start, current_word),
            string_style(),
        ));
    } else if !current_word.is_empty() {
        spans.push(classify_word(&current_word, keywords, types));
    }

    if spans.is_empty() {
        spans.push(Span::styled(line.to_string(), default_style()));
    }

    spans
}

fn classify_word(word: &str, keywords: &[&str], types: &[&str]) -> Span<'static> {
    if keywords.contains(&word) {
        Span::styled(word.to_string(), keyword_style())
    } else if types.contains(&word) {
        Span::styled(word.to_string(), type_style())
    } else if word.chars().all(|c| c.is_ascii_digit() || c == '.' || c == 'x' || c == 'b' || c == 'o')
        && word.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
    {
        Span::styled(word.to_string(), number_style())
    } else if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        Span::styled(word.to_string(), type_style())
    } else {
        Span::styled(word.to_string(), default_style())
    }
}

fn highlight_rust(line: &str) -> Vec<Span<'static>> {
    let keywords = &[
        "fn", "let", "mut", "const", "static", "struct", "enum", "impl", "trait",
        "pub", "use", "mod", "crate", "self", "super", "as", "if", "else",
        "match", "for", "while", "loop", "break", "continue", "return",
        "where", "type", "async", "await", "move", "ref", "unsafe", "extern",
        "dyn", "in", "true", "false",
    ];
    let types = &[
        "String", "Vec", "Option", "Result", "Box", "Rc", "Arc",
        "HashMap", "HashSet", "BTreeMap", "i8", "i16", "i32", "i64", "i128",
        "u8", "u16", "u32", "u64", "u128", "f32", "f64", "bool", "char",
        "usize", "isize", "str", "Self",
    ];
    highlight_with_rules(line, keywords, types, "//")
}

fn highlight_python(line: &str) -> Vec<Span<'static>> {
    let keywords = &[
        "def", "class", "import", "from", "as", "if", "elif", "else",
        "for", "while", "break", "continue", "return", "yield", "with",
        "try", "except", "finally", "raise", "pass", "lambda", "and",
        "or", "not", "in", "is", "True", "False", "None", "self",
        "async", "await", "global", "nonlocal",
    ];
    let types = &[
        "int", "float", "str", "bool", "list", "dict", "tuple", "set",
        "bytes", "type", "object",
    ];
    highlight_with_rules(line, keywords, types, "#")
}

fn highlight_js(line: &str) -> Vec<Span<'static>> {
    let keywords = &[
        "function", "const", "let", "var", "if", "else", "for", "while",
        "do", "switch", "case", "break", "continue", "return", "throw",
        "try", "catch", "finally", "class", "extends", "new", "this",
        "import", "export", "default", "from", "async", "await", "yield",
        "typeof", "instanceof", "in", "of", "true", "false", "null",
        "undefined", "interface", "type", "enum",
    ];
    let types = &[
        "String", "Number", "Boolean", "Array", "Object", "Promise",
        "Map", "Set", "Date", "RegExp", "Error",
    ];
    highlight_with_rules(line, keywords, types, "//")
}

fn highlight_shell(line: &str) -> Vec<Span<'static>> {
    let keywords = &[
        "if", "then", "else", "elif", "fi", "for", "while", "do", "done",
        "case", "esac", "in", "function", "return", "exit", "export",
        "local", "readonly", "declare", "typeset", "source", "alias",
        "unalias", "set", "unset", "shift", "trap", "eval", "exec",
        "sudo", "cd", "ls", "grep", "find", "echo", "cat", "mkdir",
        "rm", "cp", "mv", "chmod", "chown", "curl", "wget",
    ];
    let types: &[&str] = &[];
    highlight_with_rules(line, keywords, types, "#")
}

fn highlight_go(line: &str) -> Vec<Span<'static>> {
    let keywords = &[
        "func", "package", "import", "var", "const", "type", "struct",
        "interface", "map", "chan", "go", "select", "if", "else", "for",
        "range", "switch", "case", "default", "break", "continue",
        "return", "defer", "fallthrough", "goto", "true", "false", "nil",
    ];
    let types = &[
        "string", "int", "int8", "int16", "int32", "int64",
        "uint", "uint8", "uint16", "uint32", "uint64",
        "float32", "float64", "bool", "byte", "rune", "error",
    ];
    highlight_with_rules(line, keywords, types, "//")
}

fn highlight_c(line: &str) -> Vec<Span<'static>> {
    let keywords = &[
        "if", "else", "for", "while", "do", "switch", "case", "break",
        "continue", "return", "goto", "typedef", "struct", "union",
        "enum", "extern", "static", "const", "volatile", "register",
        "sizeof", "auto", "inline", "restrict", "class", "public",
        "private", "protected", "virtual", "override", "template",
        "namespace", "using", "new", "delete", "try", "catch", "throw",
        "true", "false", "nullptr", "NULL", "include", "define",
    ];
    let types = &[
        "int", "char", "float", "double", "void", "long", "short",
        "unsigned", "signed", "size_t", "bool", "uint8_t", "uint16_t",
        "uint32_t", "uint64_t", "int8_t", "int16_t", "int32_t", "int64_t",
        "string", "vector", "map", "set",
    ];
    highlight_with_rules(line, keywords, types, "//")
}

fn highlight_toml(line: &str) -> Vec<Span<'static>> {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];

    if trimmed.starts_with('#') {
        return vec![
            Span::styled(indent.to_string(), default_style()),
            Span::styled(trimmed.to_string(), comment_style()),
        ];
    }
    if trimmed.starts_with('[') {
        return vec![
            Span::styled(indent.to_string(), default_style()),
            Span::styled(trimmed.to_string(), keyword_style()),
        ];
    }
    if let Some(eq_pos) = trimmed.find('=') {
        let key = &trimmed[..eq_pos];
        let rest = &trimmed[eq_pos..];
        return vec![
            Span::styled(indent.to_string(), default_style()),
            Span::styled(key.to_string(), type_style()),
            Span::styled(rest.to_string(), string_style()),
        ];
    }
    vec![Span::styled(line.to_string(), default_style())]
}

fn highlight_yaml(line: &str) -> Vec<Span<'static>> {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];

    if trimmed.starts_with('#') {
        return vec![
            Span::styled(indent.to_string(), default_style()),
            Span::styled(trimmed.to_string(), comment_style()),
        ];
    }
    if let Some(colon_pos) = trimmed.find(':') {
        let key = &trimmed[..colon_pos];
        let rest = &trimmed[colon_pos..];
        return vec![
            Span::styled(indent.to_string(), default_style()),
            Span::styled(key.to_string(), type_style()),
            Span::styled(rest.to_string(), default_style()),
        ];
    }
    if trimmed.starts_with('-') {
        return vec![
            Span::styled(indent.to_string(), default_style()),
            Span::styled("- ".to_string(), keyword_style()),
            Span::styled(trimmed.trim_start_matches('-').trim_start().to_string(), default_style()),
        ];
    }
    vec![Span::styled(line.to_string(), default_style())]
}

fn highlight_json(line: &str) -> Vec<Span<'static>> {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];

    // Simple JSON: keys in quotes before colon
    if let Some(colon_pos) = trimmed.find(':') {
        let before = &trimmed[..colon_pos];
        let after = &trimmed[colon_pos..];
        if before.trim().starts_with('"') {
            return vec![
                Span::styled(indent.to_string(), default_style()),
                Span::styled(before.to_string(), type_style()),
                Span::styled(after.to_string(), string_style()),
            ];
        }
    }
    vec![Span::styled(line.to_string(), default_style())]
}

fn highlight_sql(line: &str) -> Vec<Span<'static>> {
    let keywords = &[
        "SELECT", "FROM", "WHERE", "INSERT", "INTO", "VALUES", "UPDATE",
        "SET", "DELETE", "CREATE", "TABLE", "DROP", "ALTER", "ADD",
        "INDEX", "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "ON",
        "AND", "OR", "NOT", "IN", "IS", "NULL", "LIKE", "BETWEEN",
        "ORDER", "BY", "GROUP", "HAVING", "LIMIT", "OFFSET", "AS",
        "DISTINCT", "COUNT", "SUM", "AVG", "MAX", "MIN", "EXISTS",
        "UNION", "ALL", "PRIMARY", "KEY", "FOREIGN", "REFERENCES",
        "CASCADE", "BEGIN", "COMMIT", "ROLLBACK", "TRANSACTION",
        "select", "from", "where", "insert", "into", "values", "update",
        "set", "delete", "create", "table", "drop", "alter", "add",
        "join", "left", "right", "inner", "outer", "on", "and", "or",
        "not", "in", "is", "null", "like", "between", "order", "by",
        "group", "having", "limit", "offset", "as", "distinct",
    ];
    let types = &[
        "INTEGER", "TEXT", "REAL", "BLOB", "VARCHAR", "CHAR", "INT",
        "BIGINT", "BOOLEAN", "DATE", "TIMESTAMP", "FLOAT", "DOUBLE",
        "DECIMAL", "SERIAL",
    ];
    highlight_with_rules(line, keywords, types, "--")
}
