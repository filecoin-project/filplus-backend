use alloy::primitives::Address;
use alloy::signers::Signature;
use anyhow::Result;

pub struct SignatureReader;

impl SignatureReader {
    pub async fn get_address_from_signature(
        &self,
        signature: &Signature,
        message: &[u8],
    ) -> Result<Address, alloy::signers::Error> {
        let address_from_signature = signature.recover_address_from_msg(&message[..]).unwrap();
        println!("Signature recovered address: {}", address_from_signature);

        Ok(address_from_signature)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[actix_rt::test]
    async fn verifier_returns_valid_address_for_valid_message() {
        let signature_reader = SignatureReader {};

        let message = b"Hello world2";
        let signature = Signature::from_str("0x69ef80549b7a8f22fbf9cacc3cfa6bbb81a579ab0252630325078a3c01f1c6816b8306303938d9c5fe3cd833bdd5380e5ff343c856cdbf5f125ef562b3faa65b1b").unwrap();

        let address_from_signature = signature_reader
            .get_address_from_signature(&signature, message)
            .await
            .unwrap();

        let expected_address =
            Address::from_str("0xC17611C85Ea2216Ee343763035eeB9c104B4066F").unwrap();

        assert_eq!(expected_address, address_from_signature);
    }

    #[actix_rt::test]
    async fn verifier_returns_invalid_address_for_invalid_message() {
        let signature_reader = SignatureReader {};

        let message = b"Hello world";
        let signature = Signature::from_str("0x69ef80549b7a8f22fbf9cacc3cfa6bbb81a579ab0252630325078a3c01f1c6816b8306303938d9c5fe3cd833bdd5380e5ff343c856cdbf5f125ef562b3faa65b1b").unwrap();

        let address_from_signature = signature_reader
            .get_address_from_signature(&signature, message)
            .await
            .unwrap();

        let expected_address =
            Address::from_str("0xC17611C85Ea2216Ee343763035eeB9c104B4066F").unwrap();

        assert_ne!(expected_address, address_from_signature);
    }
}
