use regex::Regex;

/// Extrahiert und normalisiert eine deutsche Telefonnummer aus einem String.
///
/// Behandelt alle gängigen Formate:
/// - `+49(0)1234/34421-10`  → `012343442110` (0 nach +49 entfernt, lokal)
/// - `+49 30 12345678`      → `+493012345678` (international, bleibt)
/// - `+4912345121210`       → `+4912345121210` (international, bleibt)
/// - `0049 89 22334455`     → `+498922334455` (00→+)
/// - `(0151) 555-01-00`     → `01515550100` (lokal)
/// - `0151/5550100`         → `01515550100`
/// - `Tel: +49 (0) 8031 / 2626-0` → `0803126260`
pub fn extract_phone_number(input: &str) -> Option<String> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    // Telefonnummer im Text finden
    let re = Regex::new(r"[\+]?[\d\s\(\)\-/\.]{6,}").unwrap();
    let raw = re.find(input)?.as_str();

    Some(normalize_number(raw))
}

fn normalize_number(raw: &str) -> String {
    let raw = raw.trim();

    // 1) Alles außer Ziffern und + entfernen, aber (0) merken
    //    Erkennung: +49(0)... oder +49 (0) ... → die (0) ist redundant
    let has_plus49_trunk = {
        // Pattern: +49, dann optional Whitespace, dann (0)
        let stripped: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
        stripped.starts_with("+49(0)")
            || stripped.starts_with("+49-(0)")
            || stripped.starts_with("+49/(0)")
            || stripped.starts_with("0049(0)")
            || stripped.starts_with("0049-(0)")
            || stripped.starts_with("0049/(0)")
            // Auch: +49 (0) mit Leerzeichen
            || raw.contains("+49") && raw.contains("(0)")
            || raw.contains("0049") && raw.contains("(0)")
    };

    // 2) Nur Ziffern und führendes + behalten
    let mut digits: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect();

    // 3) 00 am Anfang → +
    if digits.starts_with("00") {
        digits = format!("+{}", &digits[2..]);
    }

    // 4) +49(0)... → die 0 nach +49 entfernen und zu lokaler Nummer konvertieren
    if has_plus49_trunk {
        // +490... → entferne +49 und die Extra-0, setze eine 0 davor
        if digits.starts_with("+490") {
            digits = format!("0{}", &digits[4..]);
        } else if digits.starts_with("+49") {
            digits = format!("0{}", &digits[3..]);
        }
    }

    // 5) Mindestlänge prüfen
    let digit_count = digits.chars().filter(|c| c.is_ascii_digit()).count();
    if digit_count < 3 {
        return digits;
    }

    digits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_international() {
        assert_eq!(
            extract_phone_number("+49 30 12345678"),
            Some("+493012345678".into())
        );
    }

    #[test]
    fn test_international_compact() {
        assert_eq!(
            extract_phone_number("+4912345121210"),
            Some("+4912345121210".into())
        );
    }

    #[test]
    fn test_plus49_with_trunk_zero() {
        // +49(0) → die (0) ist redundant, Ergebnis ist lokale Nummer
        assert_eq!(
            extract_phone_number("+49(0)1234/34421-10"),
            Some("012343442110".into())
        );
    }

    #[test]
    fn test_plus49_space_trunk() {
        assert_eq!(
            extract_phone_number("+49 (0) 8031 / 2626-0"),
            Some("0803126260".into())
        );
    }

    #[test]
    fn test_double_zero_prefix() {
        assert_eq!(
            extract_phone_number("0049 89 22334455"),
            Some("+498922334455".into())
        );
    }

    #[test]
    fn test_local_with_parens() {
        assert_eq!(
            extract_phone_number("(0151) 555 01 00"),
            Some("01515550100".into())
        );
    }

    #[test]
    fn test_local_with_slash() {
        assert_eq!(
            extract_phone_number("0151/5550100"),
            Some("01515550100".into())
        );
    }

    #[test]
    fn test_local_with_dashes() {
        assert_eq!(
            extract_phone_number("0151-555-0100"),
            Some("01515550100".into())
        );
    }

    #[test]
    fn test_from_text() {
        assert_eq!(
            extract_phone_number("Tel: +49 30 12345678 - Danke"),
            Some("+493012345678".into())
        );
    }

    #[test]
    fn test_empty() {
        assert_eq!(extract_phone_number(""), None);
    }

    #[test]
    fn test_no_number() {
        assert_eq!(extract_phone_number("Hallo Welt"), None);
    }
}
