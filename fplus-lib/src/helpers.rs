use size::Size;

use crate::error::LDNError;

pub fn parse_size_to_bytes(size: &str) -> Option<u64> {
    let size = Size::from_str(size).ok()?;
    let bytes = size.bytes();
    bytes.try_into().ok()
}

pub fn format_size_human_readable(size: &str) -> Result<String, LDNError> {
    let bytes = parse_size_to_bytes(size)
        .ok_or(LDNError::Load("Failed to parse size to bytes".to_string()))?;
    let parsed_value = Size::from_bytes(bytes);
    Ok(parsed_value.to_string())
}

pub fn is_allocator_allowance_bigger_than_allocation_amount(
    allowance: &str,
    new_allocation_amount: &str,
) -> Result<bool, LDNError> {
    let allowance_bytes: u64 = allowance.parse::<u64>().map_err(|e| {
        LDNError::New(format!(
            "Parse allowance: {} to u64 failed. {}",
            &allowance, e
        ))
    })?;
    let allocation_bytes = parse_size_to_bytes(new_allocation_amount).ok_or(LDNError::Load(
        "Failed to parse allocation amount to bytes".to_string(),
    ))?;

    Ok(allowance_bytes >= allocation_bytes)
}

pub fn process_amount(mut amount: String) -> String {
    // Trim 'S' or 's' from the end of the string
    amount = amount.trim_end_matches(['s', 'S']).to_string();

    // Replace 'b' with 'B'
    amount.replace('b', "B")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_size_to_bytes_should_work_with_whitespace() {
        let res = parse_size_to_bytes("2 PiB");
        assert_eq!(res, Some(2251799813685248));
        let res = parse_size_to_bytes("2\tPiB");
        assert_eq!(res, Some(2251799813685248));
        let res = parse_size_to_bytes("2\nPiB");
        assert_eq!(res, Some(2251799813685248));
    }

    #[test]
    fn parse_size_to_bytes_should_work_without_whitespace() {
        let res = parse_size_to_bytes("2PiB");
        assert_eq!(res, Some(2251799813685248));
    }

    #[test]
    fn should_work_with_fractions() {
        let res = parse_size_to_bytes("4.32PiB");
        assert_eq!(res, Some(4863887597560136));
    }

    #[test]
    fn should_work_with_fractions_with_space() {
        let res = parse_size_to_bytes("4.32 PiB");
        assert_eq!(res, Some(4863887597560136));
    }
}
