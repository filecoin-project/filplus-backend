use chrono::prelude::*;

use super::file::{AppState, LifeCycle};

impl AppState {
    pub fn as_str(&self) -> &str {
        match *self {
            AppState::Submitted => "Submitted",
            AppState::ReadyToSign => "Ready to Sign Datacap",
            AppState::StartSignDatacap => "Start Sign Datacap",
            AppState::Granted => "Granted",
            AppState::TotalDatacapReached => "Total Datacap Reached",
            AppState::Error => "Error",
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
        }
    }

    /// Change Application state to Proposal from Governance Review
    /// Actor input is the actor who is changing the state
    pub fn finish_governance_review(&self, actor: String, current_allocation_id: String) -> Self {
        LifeCycle {
            state: AppState::ReadyToSign,
            validated_by: actor,
            validated_at: Utc::now().to_string(),
            updated_at: Utc::now().to_string(),
            active_request: Some(current_allocation_id),
            ..self.clone()
        }
    }

    pub fn finish_proposal(&self) -> Self {
        LifeCycle {
            state: AppState::StartSignDatacap,
            updated_at: Utc::now().to_string(),
            ..self.clone()
        }
    }

    pub fn finish_approval(&self) -> Self {
        LifeCycle {
            state: AppState::Granted,
            updated_at: Utc::now().to_string(),
            ..self.clone()
        }
    }

    pub fn get_state(&self) -> AppState {
        let res = self.state.clone();
        res
    }

    pub fn start_refill_request(&self, request_id: String) -> Self {
        LifeCycle {
            state: AppState::ReadyToSign,
            updated_at: Utc::now().to_string(),
            active_request: Some(request_id),
            ..self.clone()
        }
    }

    pub fn get_active_allocation_id(self) -> Option<String> {
        self.active_request
    }

    pub fn reached_total_datacap(self) -> Self {
        LifeCycle {
            is_active: false,
            updated_at: Utc::now().to_string(),
            active_request: None,
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
            active_request: None,
            ..self
        }
    }
}
