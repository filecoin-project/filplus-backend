use chrono::Utc;

use super::file::{
    SpsChangeRequest, SpsChangeRequests, StorageProviderChangeVerifier,
    StorageProviderChangeVerifiers,
};

impl StorageProviderChangeVerifiers {
    pub fn add_signer(&mut self, signer: &StorageProviderChangeVerifier) -> Self {
        self.0.push(signer.clone());
        self.clone()
    }
}

impl SpsChangeRequest {
    pub fn new(
        request_id: &String,
        allowed_sps: Option<Vec<u64>>,
        max_deviation: Option<String>,
        signer: &StorageProviderChangeVerifier,
        is_active: bool,
    ) -> Self {
        Self {
            id: request_id.into(),
            created_at: Utc::now().to_string(),
            updated_at: Utc::now().to_string(),
            is_active,
            allowed_sps,
            max_deviation,
            signers: StorageProviderChangeVerifiers(vec![signer.clone()]),
        }
    }

    pub fn get_signers(&self) -> Option<Vec<StorageProviderChangeVerifier>> {
        if !self.signers.0.is_empty() {
            Some(self.signers.0.clone())
        } else {
            None
        }
    }
}

impl SpsChangeRequests {
    pub fn add_signer_to_active_request(
        &mut self,
        request_id: &str,
        signer: &StorageProviderChangeVerifier,
    ) -> Self {
        if let Some(request) = self
            .0
            .iter_mut()
            .find(|request| request.id == request_id && request.is_active)
        {
            request.signers.add_signer(signer);
        }
        self.clone()
    }

    pub fn complete_change_request(&mut self, request_id: &str) -> Self {
        if let Some(request) = self
            .0
            .iter_mut()
            .find(|request| request.id == request_id && request.is_active)
        {
            request.is_active = false;
        }
        self.clone()
    }

    pub fn add_change_request(&mut self, sps_change_request: &SpsChangeRequest) -> Self {
        self.0.push(sps_change_request.clone());
        self.clone()
    }

    pub fn get_active_change_request(&self, request_id: &String) -> Option<SpsChangeRequest> {
        self.0
            .iter()
            .find(|request| request.id == *request_id && request.is_active)
            .cloned()
    }

    pub fn get_active_request_signers(&self) -> Option<StorageProviderChangeVerifiers> {
        self.0
            .iter()
            .find(|request| request.is_active)
            .map(|active_request| active_request.signers.clone())
    }
}
