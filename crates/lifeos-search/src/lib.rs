use unicode_normalization::UnicodeNormalization;

pub fn normalize_for_search(value: &str) -> String {
    value
        .nfkc()
        .flat_map(char::to_lowercase)
        .filter_map(|character| match character {
            '\u{0640}' | '\u{064B}'..='\u{065F}' | '\u{0670}' => None,
            '\u{0622}' | '\u{0623}' | '\u{0625}' | '\u{0671}' => Some('ا'),
            '\u{0649}' => Some('ي'),
            other => Some(other),
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn fts_query(value: &str, prefix: bool) -> Option<String> {
    let normalized = normalize_for_search(value);
    let tokens: Vec<String> = normalized
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(|token| {
            let escaped = token.replace('"', "");
            if prefix {
                format!("\"{escaped}\"*")
            } else {
                format!("\"{escaped}\"")
            }
        })
        .collect();
    (!tokens.is_empty()).then(|| tokens.join(" AND "))
}

#[cfg(test)]
mod tests {
    use super::normalize_for_search;

    #[test]
    fn preserves_original_only_in_caller_and_normalizes_common_equivalents() {
        assert_eq!(normalize_for_search("إدارة الـمَشْرُوع"), "ادارة المشروع");
        assert_eq!(normalize_for_search("Mixed مشروع 123"), "mixed مشروع 123");
        assert_eq!(normalize_for_search("مهمة"), "مهمة");
        assert_ne!(normalize_for_search("مدرسة"), normalize_for_search("مدرسه"));
    }
}
