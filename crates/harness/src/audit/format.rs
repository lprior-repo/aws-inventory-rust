//! Formatting helpers for audit evidence.

/// Format an integer with Python's `,` thousands grouping.
pub fn group_thousands(value: i64) -> String {
    let digits = value.unsigned_abs().to_string();
    let total = digits.chars().count();
    let mut grouped = String::new();
    for (index, character) in digits.chars().enumerate() {
        let position = total.saturating_sub(index);
        if index > 0 && position.rem_euclid(3) == 0 {
            grouped.push(',');
        }
        grouped.push(character);
    }
    if value < 0 {
        format!("-{grouped}")
    } else {
        grouped
    }
}
