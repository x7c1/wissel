//! The mapping between picker rows and their digit shortcuts.
//!
//! The first nine rows get the digits `1` to `9`, in order; later rows have no
//! shortcut. This module holds no GTK types so that the mapping can be unit
//! tested; the picker only renders it.

/// The number of rows that get a digit shortcut.
const SHORTCUT_COUNT: usize = 9;

/// Returns the digit that chooses the row at `index`, if it has one.
pub fn digit_for(index: usize) -> Option<char> {
    if index < SHORTCUT_COUNT {
        char::from_digit(index as u32 + 1, 10)
    } else {
        None
    }
}

/// Returns the index of the row that `key` chooses in a list of `row_count`
/// rows, or `None` when the key is not a shortcut or its row does not exist.
pub fn index_for(key: char, row_count: usize) -> Option<usize> {
    let digit = key.to_digit(10)? as usize;
    let index = digit.checked_sub(1)?;
    (index < SHORTCUT_COUNT && index < row_count).then_some(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_nine_rows_get_digits_in_order() {
        let digits: Vec<Option<char>> = (0..9).map(digit_for).collect();
        let expected: Vec<Option<char>> = "123456789".chars().map(Some).collect();
        assert_eq!(digits, expected);
    }

    #[test]
    fn rows_after_the_ninth_get_no_digit() {
        assert_eq!(digit_for(9), None);
        assert_eq!(digit_for(10), None);
        assert_eq!(digit_for(usize::MAX), None);
    }

    #[test]
    fn digit_resolves_to_its_row() {
        assert_eq!(index_for('1', 12), Some(0));
        assert_eq!(index_for('5', 12), Some(4));
        assert_eq!(index_for('9', 12), Some(8));
    }

    #[test]
    fn digit_beyond_the_rows_resolves_to_nothing() {
        assert_eq!(index_for('3', 2), None);
        assert_eq!(index_for('1', 0), None);
    }

    #[test]
    fn zero_and_non_digits_resolve_to_nothing() {
        assert_eq!(index_for('0', 12), None);
        assert_eq!(index_for('a', 12), None);
        assert_eq!(index_for('\u{0661}', 12), None); // ARABIC-INDIC DIGIT ONE
    }

    #[test]
    fn digit_and_index_agree() {
        for index in 0..12 {
            if let Some(digit) = digit_for(index) {
                assert_eq!(index_for(digit, 12), Some(index));
            }
        }
    }
}
