use chrono::prelude::*;

use super::file::{AppState, LifeCycle};

impl AppState {
    pub fn as_str(&self) -> &str {
        match *self {
            AppState::AdditionalInfoRequired => "additional information required",
            AppState::AdditionalInfoSubmitted => "additional information submitted",
            AppState::Submitted => "validated",
            AppState::KYCRequested => "kyc requested",
            AppState::ChangesRequested => "application changes requested",
            AppState::ReadyToSign => "ready to sign",
            AppState::StartSignDatacap => "start sign datacap",
            AppState::Granted => "granted",
            AppState::TotalDatacapReached => "total datacap reached",
            AppState::ChangingSP => "changing SPs",
            AppState::Error => "error",
        }
    }
}

impl LifeCycle {
    pub fn submitted(client_on_chain_address: String, multisig_address: String) -> Self {
        let empty = "".to_string();
        LifeCycle {
            state: AppState::Submitted,
            validated_at: empty.clone(),
            validated_by: empty.clone(),
            is_active: true,
            updated_at: Utc::now().to_string(),
            active_request: Some(empty),
            client_on_chain_address,
            multisig_address,
            edited: Some(false),
        }
    }

    pub fn kyc_request(&self) -> Self {
        LifeCycle {
            state: AppState::KYCRequested,
            updated_at: Utc::now().to_string(),
            ..self.clone()
        }
    }

    /// Change Application state to Proposal from Governance Review
    /// Actor input is the actor who is changing the state
    pub fn finish_governance_review(&self, actor: String, current_allocation_id: String) -> Result<Self, String> {
        let new_lifecycle = LifeCycle {
            state: AppState::ReadyToSign,
            validated_by: actor,
            validated_at: Utc::now().to_string(),
            updated_at: Utc::now().to_string(),
            active_request: Some(current_allocation_id),
            ..self.clone()
        };

        new_lifecycle.validate()?;
        Ok(new_lifecycle)
    }

    pub fn sign_grant_datacap_proposal(&self, validated_by: &str) -> Result<Self, String> {
        let new_lifecycle = LifeCycle {
            state: AppState::StartSignDatacap,
            updated_at: Utc::now().to_string(),
            validated_by: validated_by.into(),
            validated_at: Utc::now().to_string(),
            ..self.clone()
        };

        new_lifecycle.validate()?;
        Ok(new_lifecycle)
    }

    pub fn update_lifecycle_after_sign(
        &self,
        state: &AppState,
        validated_by: &String,
        request_id: &String,
    ) -> Self {
        LifeCycle {
            state: state.clone(),
            updated_at: Utc::now().to_string(),
            validated_by: validated_by.into(),
            validated_at: Utc::now().to_string(),
            active_request: Some(request_id.into()),
            ..self.clone()
        }
    }

    pub fn finish_grant_datacap_approval(&self, validated_by: &str) -> Self {
        LifeCycle {
            state: AppState::Granted,
            updated_at: Utc::now().to_string(),
            validated_by: validated_by.into(),
            validated_at: Utc::now().to_string(),
            ..self.clone()
        }
    }

    pub fn get_state(&self) -> AppState {
        self.state.clone()
    }

    pub fn get_active_status(&self) -> bool {
        self.is_active
    }

    pub fn start_refill_request(&self, actor: String, request_id: String) -> Self {
        LifeCycle {
            state: AppState::ReadyToSign,
            validated_by: actor,
            validated_at: Utc::now().to_string(),
            updated_at: Utc::now().to_string(),
            active_request: Some(request_id),
            ..self.clone()
        }
    }

    pub fn get_active_allocation_id(self) -> Option<String> {
        self.active_request
    }

    pub fn reached_total_datacap(self) -> Self {
        let empty = "".to_string();

        LifeCycle {
            is_active: false,
            updated_at: Utc::now().to_string(),
            active_request: Some(empty),
            ..self
        }
    }

    pub fn move_back_to_governance_review(self) -> Self {
        let empty = "".to_string();

        LifeCycle {
            state: AppState::Submitted,
            validated_at: empty.clone(),
            validated_by: empty.clone(),
            updated_at: Utc::now().to_string(),
            active_request: Some(empty),
            ..self
        }
    }
    pub fn move_back_to_submit_state(self) -> Self {
        LifeCycle {
            state: AppState::Submitted,
            updated_at: Utc::now().to_string(),
            ..self.clone()
        }
    }

    pub fn move_back_to_ready_to_sign(self) -> Self {
        LifeCycle {
            state: AppState::ReadyToSign,
            updated_at: Utc::now().to_string(),
            ..self.clone()
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.client_on_chain_address.is_empty() {
            return Err("Client on-chain address is required".to_string());
        }
        if self.multisig_address.is_empty() {
            return Err("Multisig address is required".to_string());
        }

        match self.state {
            AppState::Granted | AppState::StartSignDatacap => {
                if self.validated_by.is_empty() {
                    return Err("Validated by is required for Granted/StartSignDatacap state".to_string());
                }
                if self.validated_at.is_empty() {
                    return Err("Validated at is required for Granted/StartSignDatacap state".to_string());
                }
            }
            AppState::ReadyToSign => {
                if self.validated_by.is_empty() {
                    return Err("Validated by is required for ReadyToSign state".to_string());
                }
                if self.validated_at.is_empty() {
                    return Err("Validated at is required for ReadyToSign state".to_string());
                }
                if self.active_request.is_none() || self.active_request.as_ref().unwrap().is_empty() {
                    return Err("Active request ID is required for ReadyToSign state".to_string());
                }
            }
            _ => {}
        }

        Ok(())
    }
}
