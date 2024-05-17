use regex::Regex;

pub fn parse_size_to_bytes(size: &str) -> Option<u64> {
  let re = Regex::new(r"^(\d+)([a-zA-Z]+)$").unwrap();
  let caps = re.captures(size.trim())?;

  let number = caps.get(1)?.as_str().parse::<u64>().ok()?;
  let unit = caps.get(2)?.as_str().to_uppercase();

  // Normalize the unit by removing any trailing 'i', 's' and converting to upper case
  let normalized_unit = unit.trim_end_matches('S');

  match normalized_unit {
      "KIB" => Some(number * 1024),                             // 2^10
      "MIB" => Some(number * 1024 * 1024),                      // 2^20
      "GIB" => Some(number * 1024 * 1024 * 1024),               // 2^30
      "TIB" => Some(number * 1024 * 1024 * 1024 * 1024),        // 2^40
      "PIB" => Some(number * 1024 * 1024 * 1024 * 1024 * 1024), // 2^50
      "KB"  => Some(number * 1000),                             // 10^3
      "MB"  => Some(number * 1000 * 1000),                      // 10^6
      "GB"  => Some(number * 1000 * 1000 * 1000),               // 10^9
      "TB"  => Some(number * 1000 * 1000 * 1000 * 1000),        // 10^12
      "PB"  => Some(number * 1000 * 1000 * 1000 * 1000 * 1000), // 10^15
      _ => None, // Unsupported unit
  }
}

pub fn compare_allowance_and_allocation(allowance: &str, new_allocation_amount: Option<String>) -> Option<bool> {
  let allowance_bytes: u64 = match allowance.parse::<u64>() {
      Ok(value) => {
          println!("Allowance value: {}", value);
          value
      }
      Err(_) => {
          println!("Error parsing allowance value");
          return None
      }
  };


  let allocation_bytes = match new_allocation_amount {
      Some(amount) => {
          println!("Allowance value: {}", amount);
          parse_size_to_bytes(&amount)?
      },
      None => {
          println!("Error parsing allocation value");
          return None
      }, 
  };

  Some(allowance_bytes >= allocation_bytes)
}

pub fn process_amount(mut amount: String) -> String {
    // Trim 'S' or 's' from the end of the string
    amount = amount.trim_end_matches(|c: char| c == 'S' || c == 's').to_string();

    // Replace 'b' with 'B'
    amount.replace('b', "B")
}