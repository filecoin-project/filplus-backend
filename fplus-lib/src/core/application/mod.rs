use file::{AppState, SpsChangeRequest, SpsChangeRequests};

use self::file::{AllocationRequest, Allocations, LifeCycle, Verifier, Version};

pub mod allocation;
pub mod file;
pub mod gitcoin_interaction;
pub mod lifecycle;
pub mod sps_change;

impl file::ApplicationFile {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        issue_number: String,
        multisig_address: String,
        version: Version,
        id: String,
        client: file::Client,
        project: file::Project,
        datacap: file::Datacap,
    ) -> Self {
        let allocation = Allocations::default();
        let lifecycle = LifeCycle::submitted(id.clone(), multisig_address.clone());
        Self {
            version,
            issue_number,
            id,
            client,
            project,
            datacap,
            lifecycle,
            allocation,
            client_contract_address: None,
            allowed_sps: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn edited(
        issue_number: String,
        version: Version,
        id: String,
        client: file::Client,
        project: file::Project,
        datacap: file::Datacap,
        allocation: file::Allocations,
        lifecycle: file::LifeCycle,
        client_contract_address: Option<String>,
        allowed_sps: Option<file::SpsChangeRequests>,
    ) -> Self {
        //set lifecycle.edited = true
        let lifecycle = LifeCycle {
            edited: Some(true),
            ..lifecycle
        };
        Self {
            version,
            issue_number,
            id,
            client,
            project,
            datacap,
            lifecycle,
            allocation,
            client_contract_address,
            allowed_sps,
        }
    }

    pub fn reached_total_datacap(&self) -> Self {
        let new_life_cycle = self.lifecycle.clone().reached_total_datacap();
        Self {
            lifecycle: new_life_cycle,
            ..self.clone()
        }
    }

    pub fn move_back_to_governance_review(&self) -> Self {
        let new_life_cycle = self.lifecycle.clone().move_back_to_governance_review(); // move back to submitted state
        let allocation = Allocations::default(); // empty allocations
        Self {
            client_contract_address: None,
            lifecycle: new_life_cycle,
            allocation,
            ..self.clone()
        }
    }

    pub fn complete_governance_review(
        &self,
        actor: String,
        request: AllocationRequest,
        client_contract_address: Option<String>,
    ) -> Self {
        let new_life_cycle = self
            .lifecycle
            .clone()
            .finish_governance_review(actor, request.id.clone());
        let allocations = Allocations::init(request.clone());
        Self {
            lifecycle: new_life_cycle,
            allocation: allocations,
            client_contract_address,
            ..self.clone()
        }
    }

    pub fn start_refill_request(&mut self, request: AllocationRequest) -> Self {
        let new_life_cycle = self
            .lifecycle
            .clone()
            .start_refill_request(request.actor.clone(), request.id.clone());
        let allocations = self.allocation.clone().push(request.clone());
        Self {
            lifecycle: new_life_cycle,
            allocation: allocations,
            ..self.clone()
        }
    }

    pub fn handle_changing_sps_request(
        &mut self,
        validated_by: &String,
        sps_change_request: &SpsChangeRequest,
        app_state: &AppState,
        request_id: &String,
    ) -> Self {
        let new_life_cycle =
            self.lifecycle
                .clone()
                .update_lifecycle_after_sign(app_state, validated_by, request_id);
        let sps_change_requests = self
            .allowed_sps
            .clone()
            .unwrap_or_default()
            .add_change_request(sps_change_request);
        Self {
            lifecycle: new_life_cycle,
            allowed_sps: Some(sps_change_requests),
            ..self.clone()
        }
    }

    pub fn update_changing_sps_request(
        &mut self,
        validated_by: &String,
        sps_change_requests: &SpsChangeRequests,
        app_state: &AppState,
        request_id: &String,
    ) -> Self {
        let new_life_cycle =
            self.lifecycle
                .clone()
                .update_lifecycle_after_sign(app_state, validated_by, request_id);
        Self {
            lifecycle: new_life_cycle,
            allowed_sps: Some(sps_change_requests.clone()),
            ..self.clone()
        }
    }

    pub fn add_signer_to_allocation(&self, signer: Verifier, request_id: &str) -> Self {
        let allocation_after_sign = self.allocation.clone().add_signer(request_id, signer);
        Self {
            allocation: allocation_after_sign,
            ..self.clone()
        }
    }

    pub fn move_back_to_ready_to_sign(&self) -> Self {
        let updated_allocation = self
            .clone()
            .allocation
            .remove_signers_in_active_allocation();
        let updated_lifecycle = self.clone().lifecycle.move_back_to_ready_to_sign();
        Self {
            lifecycle: updated_lifecycle,
            allocation: updated_allocation,
            ..self.clone()
        }
    }

    pub fn add_signer_to_allocation_and_complete(
        &self,
        signer: Verifier,
        request_id: String,
        app_lifecycle: LifeCycle,
    ) -> Self {
        let new_allocation = self
            .allocation
            .clone()
            .add_signer_and_complete(request_id, signer);
        Self {
            allocation: new_allocation,
            lifecycle: app_lifecycle,
            ..self.clone()
        }
    }

    pub fn move_back_to_submit_state(self) -> Self {
        let new_life_cycle = self.lifecycle.clone().move_back_to_submit_state();
        Self {
            lifecycle: new_life_cycle,
            ..self.clone()
        }
    }

    pub fn kyc_request(&self) -> Self {
        let new_life_cycle = self.lifecycle.clone().kyc_request();
        Self {
            lifecycle: new_life_cycle,
            ..self.clone()
        }
    }
}

impl std::str::FromStr for file::ApplicationFile {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
