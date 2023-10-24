use serde::{Serialize, Deserialize};

use self::{
    allocations::{AllocationRequest, ApplicationAllocationTypes, ApplicationAllocationsSigner},
    core_info::ApplicationInfo,
    lifecycle::ApplicationLifecycle,
};

pub(crate) mod allocations;
pub(crate) mod core_info;
pub(crate) mod lifecycle;
pub(crate) mod file;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ApplicationType {
    DA,
    LDN,
    EFIL,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApplicationFile {
    pub id: String,
    pub _type: ApplicationType,
    pub info: ApplicationInfo,
}

impl ApplicationFile {
    pub async fn new(app_info: ApplicationInfo, application_id: String) -> Self {
        ApplicationFile {
            id: application_id,
            _type: ApplicationType::LDN,
            info: app_info,
        }
    }

    pub fn reached_total_datacap(&self) -> Self {
        let new_life_cycle = self
            .info
            .application_lifecycle
            .clone()
            .reached_total_datacap();
        let info = ApplicationInfo {
            core_information: self.info.core_information.clone(),
            application_lifecycle: new_life_cycle,
            datacap_allocations: self.info.datacap_allocations.clone(),
        };
        ApplicationFile {
            id: self.id.clone(),
            _type: self._type.clone(),
            info,
        }
    }


    pub fn complete_governance_review(&self, actor: String, request_id: String) -> Self {
        let new_life_cycle = self
            .info
            .application_lifecycle
            .clone()
            .set_proposal_state(actor, Some(request_id));
        let info = ApplicationInfo {
            core_information: self.info.core_information.clone(),
            application_lifecycle: new_life_cycle,
            datacap_allocations: self.info.datacap_allocations.clone(),
        };
        ApplicationFile {
            id: self.id.clone(),
            _type: self._type.clone(),
            info,
        }
    }

    pub fn start_new_allocation(&self, request: AllocationRequest) -> Self {
        match request.request_type {
            ApplicationAllocationTypes::New => {
                let new_allocation = self.info.datacap_allocations.clone().new(request.clone());
                let new_life_cycle = self
                    .info
                    .application_lifecycle
                    .clone()
                    .set_proposal_state(request.actor.clone(), Some(request.id));
                let info = ApplicationInfo {
                    core_information: self.info.core_information.clone(),
                    application_lifecycle: new_life_cycle,
                    datacap_allocations: new_allocation,
                };
                return ApplicationFile {
                    id: self.id.clone(),
                    _type: self._type.clone(),
                    info,
                };
            }
            ApplicationAllocationTypes::Refill => {
                let new_allocation = self.info.datacap_allocations.clone().add_new_request(request.clone());
                let new_life_cycle = self
                    .info
                    .application_lifecycle
                    .clone()
                    .set_proposal_state(request.actor.clone(), Some(request.id));
                let info = ApplicationInfo {
                    core_information: self.info.core_information.clone(),
                    application_lifecycle: new_life_cycle,
                    datacap_allocations: new_allocation,
                };
                return ApplicationFile {
                    id: self.id.clone(),
                    _type: self._type.clone(),
                    info,
                };
            }
            ApplicationAllocationTypes::Removal => {
                unimplemented!()
            }
        }
    }

    pub fn add_signer_to_allocation(
        &self,
        signer: ApplicationAllocationsSigner,
        request_id: String,
        app_lifecycle: ApplicationLifecycle,
    ) -> Self {
        let new_allocation = self
            .info
            .datacap_allocations
            .clone()
            .add_signer(request_id, signer);
        let info = ApplicationInfo {
            core_information: self.info.core_information.clone(),
            application_lifecycle: app_lifecycle,
            datacap_allocations: new_allocation,
        };
        ApplicationFile {
            id: self.id.clone(),
            _type: self._type.clone(),
            info,
        }
    }

    pub fn add_signer_to_allocation_and_complete(
        &self,
        signer: ApplicationAllocationsSigner,
        request_id: String,
        app_lifecycle: ApplicationLifecycle,
    ) -> Self {
        let new_allocation = self
            .info
            .datacap_allocations
            .clone()
            .add_signer_and_complete(request_id, signer);
        let info = ApplicationInfo {
            core_information: self.info.core_information.clone(),
            application_lifecycle: app_lifecycle,
            datacap_allocations: new_allocation,
        };
        ApplicationFile {
            id: self.id.clone(),
            _type: self._type.clone(),
            info,
        }
    }
}
