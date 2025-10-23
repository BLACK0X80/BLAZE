/// Helper functions for generating error suggestions

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    
    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[len1][len2]
}

/// Find the closest match from a list of candidates
pub fn find_closest_match<'a>(target: &str, candidates: &[&'a str]) -> Option<&'a str> {
    if candidates.is_empty() {
        return None;
    }

    let mut best_match = candidates[0];
    let mut best_distance = levenshtein_distance(target, best_match);

    for &candidate in &candidates[1..] {
        let distance = levenshtein_distance(target, candidate);
        if distance < best_distance {
            best_distance = distance;
            best_match = candidate;
        }
    }

    // Only suggest if the distance is reasonable (less than half the length)
    if best_distance <= target.len() / 2 && best_distance <= 3 {
        Some(best_match)
    } else {
        None
    }
}

/// Generate a "did you mean" suggestion
pub fn did_you_mean(target: &str, candidates: &[&str]) -> Option<String> {
    find_closest_match(target, candidates)
        .map(|suggestion| format!("did you mean `{}`?", suggestion))
}

/// Common type error suggestions
pub fn suggest_type_conversion(from_type: &str, to_type: &str) -> Option<String> {
    match (from_type, to_type) {
        ("i32", "f64") | ("i64", "f64") => {
            Some("try converting with `as f64`".to_string())
        }
        ("f64", "i32") | ("f64", "i64") => {
            Some("try converting with `as i32` (note: this will truncate)".to_string())
        }
        ("&str", "String") => {
            Some("try using `.to_string()` or `String::from()`".to_string())
        }
        ("String", "&str") => {
            Some("try using `&` to create a string slice".to_string())
        }
        _ => None,
    }
}

/// Suggest fixes for common borrow errors
pub fn suggest_borrow_fix(error_type: &str) -> Option<String> {
    match error_type {
        "multiple_mutable_borrows" => {
            Some("consider using separate scopes or restructuring your code to avoid simultaneous mutable borrows".to_string())
        }
        "mutable_and_immutable_borrow" => {
            Some("ensure all immutable borrows are no longer in use before creating a mutable borrow".to_string())
        }
        "use_after_move" => {
            Some("consider cloning the value before moving it, or use a reference instead".to_string())
        }
        _ => None,
    }
}

/// Suggest fixes for common syntax errors
pub fn suggest_syntax_fix(error_type: &str) -> Option<String> {
    match error_type {
        "missing_semicolon" => {
            Some("add a semicolon `;` at the end of the statement".to_string())
        }
        "missing_brace" => {
            Some("ensure all opening braces `{` have matching closing braces `}`".to_string())
        }
        "missing_paren" => {
            Some("ensure all opening parentheses `(` have matching closing parentheses `)`".to_string())
        }
        "unexpected_token" => {
            Some("check for typos or missing punctuation".to_string())
        }
        _ => None,
    }
}

/// Generate example code for common patterns
pub fn example_for_pattern(pattern: &str) -> Option<String> {
    match pattern {
        "function_definition" => {
            Some("fn function_name(param: Type) -> ReturnType {\n    // function body\n}".to_string())
        }
        "variable_declaration" => {
            Some("let variable_name: Type = value;".to_string())
        }
        "if_statement" => {
            Some("if condition {\n    // then branch\n} else {\n    // else branch\n}".to_string())
        }
        "while_loop" => {
            Some("while condition {\n    // loop body\n}".to_string())
        }
        "for_loop" => {
            Some("for item in collection {\n    // loop body\n}".to_string())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("", "test"), 4);
    }

    #[test]
    fn test_find_closest_match() {
        let candidates = vec!["count", "counter", "amount"];
        assert_eq!(find_closest_match("cont", &candidates), Some("count"));
        assert_eq!(find_closest_match("countr", &candidates), Some("counter"));
        assert_eq!(find_closest_match("xyz", &candidates), None);
    }

    #[test]
    fn test_did_you_mean() {
        let candidates = vec!["println", "print", "format"];
        let suggestion = did_you_mean("printl", &candidates);
        assert_eq!(suggestion, Some("did you mean `println`?".to_string()));
    }

    #[test]
    fn test_suggest_type_conversion() {
        assert!(suggest_type_conversion("i32", "f64").is_some());
        assert!(suggest_type_conversion("&str", "String").is_some());
        assert!(suggest_type_conversion("i32", "i64").is_none());
    }

    #[test]
    fn test_suggest_borrow_fix() {
        assert!(suggest_borrow_fix("multiple_mutable_borrows").is_some());
        assert!(suggest_borrow_fix("unknown_error").is_none());
    }

    #[test]
    fn test_example_for_pattern() {
        assert!(example_for_pattern("function_definition").is_some());
        assert!(example_for_pattern("unknown_pattern").is_none());
    }
}
