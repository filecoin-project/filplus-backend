use size::Size;

pub fn parse_size_to_bytes(size: &str) -> Option<u64> {
    let size = Size::from_str(size).ok()?;
    let bytes = size.bytes();
    bytes.try_into().ok()
}

pub fn compare_allowance_and_allocation(
    allowance: &str,
    new_allocation_amount: Option<String>,
) -> Option<bool> {
    let allowance_bytes: u64 = match allowance.parse::<u64>() {
        Ok(value) => {
            println!("Allowance value: {}", value);
            value
        }
        Err(_) => {
            println!("Error parsing allowance value");
            return None;
        }
    };

    let allocation_bytes = match new_allocation_amount {
        Some(amount) => {
            println!("Allowance value: {}", amount);
            parse_size_to_bytes(&amount)?
        }
        None => {
            println!("Error parsing allocation value");
            return None;
        }
    };

    Some(allowance_bytes >= allocation_bytes)
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
