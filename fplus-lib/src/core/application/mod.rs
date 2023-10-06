use self::{
    allocations::{AllocationRequest, ApplicationAllocationTypes, ApplicationAllocationsSigner},
    core_info::ApplicationInfo,
    lifecycle::ApplicationLifecycle,
};

pub(crate) mod allocations;
pub(crate) mod core_info;
pub(crate) mod lifecycle;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum ApplicationType {
    DA,
    LDN,
    EFIL,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
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

    pub fn complete_governance_review(&self, actor: String) -> Self {
        let new_life_cycle = self
            .info
            .application_lifecycle
            .clone()
            .set_proposal_state(actor);
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
                    .set_proposal_state(request.actor.clone());
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
            ApplicationAllocationTypes::Refill => {
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
}
