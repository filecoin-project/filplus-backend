use self::{
    allocations::{AllocationRequest, ApplicationAllocationTypes, ApplicationAllocationsSigner, ApplicationAllocation},
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

impl ApplicationType {
    pub fn as_str(&self) -> &str {
        match *self {
            ApplicationType::DA => "da",
            ApplicationType::LDN => "ldn-v3",
            ApplicationType::EFIL => "e-fil",
        }
    }
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

    pub fn find_one_allocation(&self, request_id: String) -> Option<ApplicationAllocation> {
          self.info.datacap_allocations.find_one(request_id)
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
                let new_allocation = self
                    .info
                    .datacap_allocations
                    .clone()
                    .new(request.clone());
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

    pub fn complete_allocation(&self, request_id: String) -> Self {
        let new_allocation = self
            .info
            .datacap_allocations
            .complete_allocation(request_id);

        let info = ApplicationInfo {
            core_information: self.info.core_information.clone(),
            application_lifecycle: self.info.application_lifecycle.clone(),
            datacap_allocations: new_allocation,
        };
        ApplicationFile {
            id: self.id.clone(),
            _type: self._type.clone(),
            info,
        }
    }
}

// core_information: ApplicationCoreInfo::new(
//     parsed_ldn.name,
//     parsed_ldn.region,
//     "TODO".to_string(),
//     parsed_ldn.website,
//     "TODO".to_string(),
// ),
// let info = ApplicationInfo {

//     application_lifecycle: ApplicationLifecycle::set_governance_review_state(),
//     datacap_allocations: ApplicationAllocations::default(),
// };
//

//     /// when u add a trigger u move to proposal state
//     pub fn add_trigger(&self) -> Self {
//         let new_life_cycle: ApplicationLifecycle =
//             self.info.application_lifecycle.clone().set_proposal_state();
//         let info = ApplicationInfo {
//             core_information: self.info.core_information.clone(),
//             application_lifecycle: new_life_cycle, // only update new lifecycle
//             datacap_allocations: self.info.datacap_allocations.clone(),
//         };
//         ApplicationFile {
//             id: self.id,
//             _type: self._type.clone(),
//             info,
//         }
//     }

//     pub fn add_proposal(
//         &self,
//         uuid: String,
//         client_address: String,
//         notary_address: String,
//         time_of_signature: String,
//         message_cid: String,
//     ) -> Self {
//         let new_life_cycle: ApplicationLifecycle =
//             self.info.application_lifecycle.clone().set_approval_state();
//         let info = ApplicationInfo {
//             core_information: self.info.core_information.clone(),
//             application_lifecycle: new_life_cycle,
//             datacap_allocations: self.info.datacap_allocations.proposal(
//                 uuid,
//                 client_address,
//                 notary_address,
//                 time_of_signature,
//                 message_cid,
//             ),
//         };

//         ApplicationFile {
//             id: self.id,
//             _type: self._type.clone(),
//             info,
//         }
//     }

//     pub fn add_approval(
//         &self,
//         uuid: String,
//         client_address: String,
//         notary_address: String,
//         time_of_signature: String,
//         allocation_amount: String,
//         message_cid: String,
//     ) -> Self {
//         let new_life_cycle: ApplicationLifecycle = self
//             .info
//             .application_lifecycle
//             .clone()
//             .set_confirmed_state();
//         let info = ApplicationInfo {
//             core_information: self.info.core_information.clone(),
//             application_lifecycle: new_life_cycle, // only update new lifecycle
//             datacap_allocations: self.info.datacap_allocations.clone().approval(
//                 uuid,
//                 client_address,
//                 notary_address,
//                 allocation_amount,
//                 time_of_signature,
//                 message_cid,
//             ),
//         };

//         ApplicationFile {
//             id: self.id,
//             _type: self._type.clone(),
//             info,
//         }
//     }
