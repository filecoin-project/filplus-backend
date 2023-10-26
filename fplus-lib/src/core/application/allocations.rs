#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApplicationAllocationsSigner {
    pub signing_address: String,
    pub time_of_signature: String,
    pub message_cid: String,
    pub username: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApplicationAllocationsSigners(Vec<ApplicationAllocationsSigner>);

impl Default for ApplicationAllocationsSigners {
    fn default() -> Self {
        Self(vec![])
    }
}

impl ApplicationAllocationsSigners {
    pub fn add(&self, signer: ApplicationAllocationsSigner) -> Self {
        let mut res = self.0.clone();
        res.push(signer);
        Self(res)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum ApplicationAllocationTypes {
    New,
    Removal,
    Refill,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AllocationRequest {
    pub actor: String,
    pub id: String,
    pub request_type: ApplicationAllocationTypes,
    pub client_address: String,
    pub created_at: String,
    pub is_active: bool,
    pub allocation_amount: String,
}

impl AllocationRequest {
    pub fn new(
        actor: String,
        id: String,
        request_type: ApplicationAllocationTypes,
        client_address: String,
        created_at: String,
        allocation_amount: String,
    ) -> Self {
        Self {
            actor,
            id,
            request_type,
            client_address,
            created_at,
            allocation_amount,
            is_active: true,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApplicationAllocation {
    pub request_information: AllocationRequest,
    pub signers: ApplicationAllocationsSigners,
}

impl ApplicationAllocation {
    pub fn new(request_information: AllocationRequest) -> Self {
        Self {
            request_information,
            signers: ApplicationAllocationsSigners::default(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApplicationAllocations(Vec<ApplicationAllocation>);

impl Default for ApplicationAllocations {
    fn default() -> Self {
        Self(vec![])
    }
}

impl ApplicationAllocations {
    pub fn new(&self, request_information: AllocationRequest) -> Self {
        let allocation = ApplicationAllocation::new(request_information);
        Self(vec![allocation])
    }

    // should be changed to option
    pub fn find_one(&self, request_id: String) -> Option<ApplicationAllocation> {
        let curr: Vec<ApplicationAllocation> = self.0.clone();
        let mut allocation: Option<ApplicationAllocation> = None;
        for alloc in curr.iter() {
            if alloc.request_information.id == request_id {
                allocation = Some(alloc.clone());
                break;
            }
        }
        allocation
    }

    // should be changed to option
    pub fn is_active(&self, request_id: String) -> bool {
        let curr: Vec<ApplicationAllocation> = self.0.clone();
        let mut is_active = false;
        for alloc in curr.iter() {
            if alloc.request_information.id == request_id {
                is_active = alloc.request_information.is_active;
                break;
            }
        }
        is_active
    }

    pub fn add_signer(&self, request_id: String, signer: ApplicationAllocationsSigner) -> Self {
        let mut res: Vec<ApplicationAllocation> = self.0.clone();
        for allocation in res.iter_mut() {
            if allocation.request_information.id == request_id
                && allocation.request_information.is_active
            {
                allocation.signers = allocation.signers.add(signer);
                break;
            }
        }
        Self(res)
    }

    pub fn add_signer_and_complete(
        &self,
        request_id: String,
        signer: ApplicationAllocationsSigner,
    ) -> Self {
        let mut res: Vec<ApplicationAllocation> = self.0.clone();
        for allocation in res.iter_mut() {
            if allocation.request_information.id == request_id
                && allocation.request_information.is_active
            {
                allocation.signers = allocation.signers.add(signer);
                allocation.request_information.is_active = false;
                break;
            }
        }
        Self(res)
    }

    pub fn complete_allocation(&self, request_id: String) -> Self {
        let mut res: Vec<ApplicationAllocation> = self.0.clone();
        for allocation in res.iter_mut() {
            if allocation.request_information.id == request_id
                && allocation.request_information.is_active
            {
                allocation.request_information.is_active = false;
            }
        }
        Self(res)
    }

    pub fn add_new_request(&mut self, request: AllocationRequest) -> Self {
        let allocation = ApplicationAllocation::new(request);
        self.0.push(allocation);
        self.clone()
    }
}
