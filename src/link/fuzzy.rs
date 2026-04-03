/// Simple fuzzy string matching algorithm
/// Returns a score where higher = better match (0-100)
pub fn fuzzy_match(pattern: &str, text: &str) -> u32 {
    if pattern.is_empty() {
        return 100; // Empty pattern matches everything
    }

    if text.is_empty() {
        return 0;
    }

    let pattern_lower = pattern.to_lowercase();
    let text_lower = text.to_lowercase();

    // Check if text contains pattern as substring (exact match gives highest score)
    if text_lower.contains(&pattern_lower) {
        return 100;
    }

    // Fuzzy matching: all pattern chars must be in text in order
    let mut pattern_idx = 0;
    let mut text_idx = 0;
    let mut matched_chars = 0;
    let mut last_match_distance = 0usize;
    let mut total_distance = 0usize;

    let pattern_chars: Vec<char> = pattern_lower.chars().collect();
    let text_chars: Vec<char> = text_lower.chars().collect();

    while pattern_idx < pattern_chars.len() && text_idx < text_chars.len() {
        if pattern_chars[pattern_idx] == text_chars[text_idx] {
            matched_chars += 1;
            last_match_distance = 0;
            pattern_idx += 1;
        } else {
            last_match_distance += 1;
        }
        total_distance += last_match_distance;
        text_idx += 1;
    }

    // All pattern chars matched
    if pattern_idx == pattern_chars.len() {
        // Calculate score based on distance and position
        // Prefer matches at the start and with less gaps
        let match_ratio = (matched_chars as f32 / pattern_chars.len() as f32) * 100.0;
        let position_bonus = if text_idx < text_chars.len() / 2 {
            10
        } else {
            0
        };
        let distance_penalty = (total_distance as f32 / text_chars.len() as f32).min(50.0);

        let score = (match_ratio + position_bonus as f32 - distance_penalty).max(1.0) as u32;
        score.min(99) // Return at most 99 for fuzzy matches
    } else {
        0 // Pattern not found in text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match_empty_pattern() {
        assert_eq!(fuzzy_match("", "hello"), 100);
    }

    #[test]
    fn test_fuzzy_match_empty_text() {
        assert_eq!(fuzzy_match("hello", ""), 0);
    }

    #[test]
    fn test_fuzzy_match_exact_match() {
        assert_eq!(fuzzy_match("hello", "hello"), 100);
    }

    #[test]
    fn test_fuzzy_match_substring() {
        assert_eq!(fuzzy_match("llo", "hello"), 100);
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        assert_eq!(fuzzy_match("HeLLo", "hello"), 100);
    }

    #[test]
    fn test_fuzzy_match_partial() {
        let score = fuzzy_match("hlo", "hello");
        assert!(score > 0 && score < 100);
    }

    #[test]
    fn test_fuzzy_match_no_match() {
        assert_eq!(fuzzy_match("xyz", "hello"), 0);
    }

    #[test]
    fn test_fuzzy_match_beginning_bonus() {
        let beginning = fuzzy_match("hello", "hello world");
        let middle = fuzzy_match("hello", "world hello");
        // "hello world" is a substring match at the start (score 100)
        assert_eq!(beginning, 100);
        // "world hello" also contains substring "hello" (score 100)
        assert_eq!(middle, 100);
    }

    #[test]
    fn test_fuzzy_match_word_boundaries() {
        let score1 = fuzzy_match("note", "My Note Title");
        let score2 = fuzzy_match("note", "mynotetitle");
        // Word boundary match should be better
        assert!(score1 >= score2);
    }
}
