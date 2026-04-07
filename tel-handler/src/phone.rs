use regex::Regex;

/// Extract and normalize a phone number from arbitrary input.
///
/// Handles common German phone number variants:
/// - `+49(0)1234/5678-90`  → `0123456789` (trunk zero collapsed to local form)
/// - `+49 40 12345678`     → `+494012345678`
/// - `0049 40 12345678`    → `+494012345678`
/// - `(0151) 555 01 00`    → `01515550100`
///
/// Returns `None` if no number with at least 3 digits can be extracted.
pub fn normalize_phone(input: &str) -> Option<String> {
    let re = Regex::new(r"[\+]?[\d\s\(\)\-/\.]{3,}").ok()?;
    let raw = re.find(input)?.as_str().trim();

    let has_trunk = (raw.contains("+49") || raw.contains("0049")) && raw.contains("(0)");

    let mut digits: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect();

    if digits.starts_with("00") {
        digits = format!("+{}", &digits[2..]);
    }

    if has_trunk && digits.starts_with("+490") {
        digits = format!("0{}", &digits[4..]);
    }

    let count = digits.chars().filter(|c| c.is_ascii_digit()).count();
    if count < 3 {
        return None;
    }

    Some(digits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn international_with_spaces() {
        assert_eq!(
            normalize_phone("+49 40 12345678"),
            Some("+494012345678".into())
        );
    }

    #[test]
    fn international_compact() {
        assert_eq!(
            normalize_phone("+4912345121210"),
            Some("+4912345121210".into())
        );
    }

    #[test]
    fn double_zero_prefix_becomes_plus() {
        assert_eq!(
            normalize_phone("0049 40 12345678"),
            Some("+494012345678".into())
        );
    }

    #[test]
    fn trunk_zero_in_parentheses() {
        // +49(0)1234... → local form 0123...
        assert_eq!(
            normalize_phone("+49(0)1234567890"),
            Some("01234567890".into())
        );
    }

    #[test]
    fn trunk_zero_with_spaces() {
        assert_eq!(
            normalize_phone("+49 (0) 8031 2626 0"),
            Some("0803126260".into())
        );
    }

    #[test]
    fn local_with_parentheses_and_dashes() {
        assert_eq!(
            normalize_phone("(0151) 555-01-00"),
            Some("01515550100".into())
        );
    }

    #[test]
    fn embedded_in_text() {
        assert_eq!(
            normalize_phone("Call me at +49 40 12345678 please"),
            Some("+494012345678".into())
        );
    }

    #[test]
    fn empty_input_returns_none() {
        assert_eq!(normalize_phone(""), None);
    }

    #[test]
    fn pure_text_returns_none() {
        assert_eq!(normalize_phone("Hallo Welt"), None);
    }

    #[test]
    fn too_few_digits_returns_none() {
        assert_eq!(normalize_phone("12"), None);
    }
}
