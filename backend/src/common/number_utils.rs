use crate::common::constants::{DIGIT_GROUP_SEPARATOR, DIGIT_GROUP_SIZE};

pub fn format_whole_number(whole_number: u64) -> String {
    let digits = whole_number.to_string();
    let mut formatted_number =
        String::with_capacity(digits.len() + digits.len() / DIGIT_GROUP_SIZE);
    for (digit_index, digit) in digits.chars().enumerate() {
        let remaining_digit_count = digits.len() - digit_index;
        if digit_index > 0 && remaining_digit_count.is_multiple_of(DIGIT_GROUP_SIZE) {
            formatted_number.push(DIGIT_GROUP_SEPARATOR);
        }
        formatted_number.push(digit);
    }
    formatted_number
}

#[cfg(test)]
mod tests {
    use super::format_whole_number;

    #[test]
    fn groups_digits_in_threes() {
        assert_eq!(format_whole_number(0), "0");
        assert_eq!(format_whole_number(999), "999");
        assert_eq!(format_whole_number(1_000), "1,000");
        assert_eq!(format_whole_number(41_250), "41,250");
        assert_eq!(format_whole_number(1_000_000), "1,000,000");
    }
}
