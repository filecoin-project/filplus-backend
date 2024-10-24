use self::file::{AllocationRequest, Allocations, LifeCycle, Verifier, Version};

pub mod allocation;
pub mod file;
pub mod gitcoin_interaction;
pub mod lifecycle;

impl file::ApplicationFile {
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

    pub fn add_signer_to_allocation(
        &self,
        signer: Verifier,
        request_id: String,
        app_lifecycle: LifeCycle,
    ) -> Self {
        let new_allocation = self.allocation.clone().add_signer(request_id, signer);
        Self {
            allocation: new_allocation,
            lifecycle: app_lifecycle,
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
