use self::file::{AllocationRequest, Allocations, LifeCycle, Notary};

pub mod allocation;
pub mod client;
pub mod datacap;
pub mod file;
pub mod lifecycle;
pub mod project;

impl file::ApplicationFile {
    pub async fn new(
        issue_number: String,
        multisig_address: String,
        version: u8,
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
            lifecycle: new_life_cycle,
            allocation,
            ..self.clone()
        }
    }

    pub fn complete_governance_review(&self, actor: String, request: AllocationRequest) -> Self {
        let new_life_cycle = self
            .lifecycle
            .clone()
            .finish_governance_review(actor, request.id.clone());
        let allocations = Allocations::init(request.clone());
        Self {
            lifecycle: new_life_cycle,
            allocation: allocations,
            ..self.clone()
        }
    }

    pub fn start_refill_request(&mut self, request: AllocationRequest) -> Self {
        let new_life_cycle = self
            .lifecycle
            .clone()
            .start_refill_request(request.id.clone());
        let allocations = self.allocation.clone().push(request.clone());
        Self {
            lifecycle: new_life_cycle,
            allocation: allocations,
            ..self.clone()
        }
    }

    pub fn add_signer_to_allocation(
        &self,
        signer: Notary,
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

    pub fn add_signer_to_allocation_and_complete(
        &self,
        signer: Notary,
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
}
