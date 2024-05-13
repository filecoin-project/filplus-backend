use alloy::primitives::Address;
use alloy::signers::Signature;
use anyhow::Result;

pub async fn get_address_from_signature(
    signature: &Signature,
    message: &[u8],
) -> Result<Address, alloy::signers::Error> {
    let address_from_signature = signature.recover_address_from_msg(&message[..]).unwrap();

    Ok(address_from_signature)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    const SIGNATURE_HASH: &str = "0xeeec9a87a01977a48fa5bac97d2f1c67d83905ac378573d6749ae078b76b3ef078f2187ef9cbf4eaf2069066fdc32b07823508db3871ab07f32d30137c0140a81c";

    #[actix_rt::test]
    async fn verifier_returns_valid_address_for_valid_message() {
        let message = b"KYC";
        let signature = Signature::from_str(SIGNATURE_HASH).unwrap();

        let address_from_signature = get_address_from_signature(&signature, message)
            .await
            .unwrap();

        let expected_address =
            Address::from_str("0x79e214f3aa3101997ffe810a57eca4586e3bdeb2").unwrap();

        assert_eq!(expected_address, address_from_signature);
    }

    #[actix_rt::test]
    async fn verifier_returns_invalid_address_for_invalid_message() {
        let message = b"Invalid message";
        let signature = Signature::from_str(SIGNATURE_HASH).unwrap();

        let address_from_signature = get_address_from_signature(&signature, message)
            .await
            .unwrap();

        let expected_address =
            Address::from_str("0x79e214f3aa3101997ffe810a57eca4586e3bdeb2").unwrap();

        assert_ne!(expected_address, address_from_signature);
    }
}
