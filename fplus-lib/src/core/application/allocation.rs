use chrono::Utc;

use super::file::{
    Allocation, AllocationRequest, AllocationRequestType, Allocations, Verifier, Verifiers,
};
use crate::helpers::parse_size_to_bytes;

impl Verifiers {
    pub fn add(&self, signer: Verifier) -> Self {
        let mut res = self.0.clone();
        res.push(signer);
        Self(res)
    }
}

impl AllocationRequest {
    pub fn new(
        actor: String,
        id: String,
        kind: AllocationRequestType,
        allocation_amount: String,
    ) -> Self {
        Self {
            actor,
            id,
            kind,
            allocation_amount,
            is_active: true,
        }
    }
}

impl Allocation {
    pub fn new(request_information: AllocationRequest) -> Self {
        Self {
            id: request_information.id,
            request_type: request_information.kind.to_string(),
            created_at: Utc::now().to_string(),
            updated_at: Utc::now().to_string(),
            is_active: true,
            amount: request_information.allocation_amount,
            signers: Verifiers::default(),
        }
    }

    pub fn remove_signers(&self) -> Self {
        Self {
            updated_at: Utc::now().to_string(),
            signers: Verifiers::default(),
            ..self.clone()
        }
    }
}

impl Allocations {
    pub fn init(request_information: AllocationRequest) -> Self {
        let allocation = Allocation::new(request_information);
        Self(vec![allocation])
    }

    // should be changed to option
    pub fn active(&self) -> Option<Allocation> {
        let curr: Vec<Allocation> = self.0.clone();
        let mut allocation: Option<Allocation> = None;
        for alloc in curr.iter() {
            if alloc.is_active {
                allocation = Some(alloc.clone());
                break;
            }
        }
        allocation
    }

    // should be changed to option
    pub fn find_one(&self, request_id: String) -> Option<Allocation> {
        let curr: Vec<Allocation> = self.0.clone();
        let mut allocation: Option<Allocation> = None;
        for alloc in curr.iter() {
            if alloc.id == request_id {
                allocation = Some(alloc.clone());
                break;
            }
        }
        allocation
    }

    // should be changed to option
    pub fn is_active(&self, request_id: String) -> bool {
        let curr: Vec<Allocation> = self.0.clone();
        let mut is_active = false;
        for alloc in curr.iter() {
            if alloc.id == request_id {
                is_active = alloc.is_active;
                break;
            }
        }
        is_active
    }

    pub fn add_signer(&self, request_id: &str, signer: Verifier) -> Self {
        let mut res: Vec<Allocation> = self.0.clone();
        for allocation in res.iter_mut() {
            if allocation.id == request_id && allocation.is_active {
                allocation.signers = allocation.signers.add(signer);
                break;
            }
        }
        Self(res)
    }

    pub fn remove_signers_in_active_allocation(&mut self) -> Self {
        if let Some(alloc) = self.0.iter_mut().find(|alloc| alloc.is_active) {
            *alloc = alloc.remove_signers();
        }
        self.clone()
    }

    pub fn add_signer_and_complete(&self, request_id: String, signer: Verifier) -> Self {
        let mut res: Vec<Allocation> = self.0.clone();
        for allocation in res.iter_mut() {
            if allocation.id == request_id && allocation.is_active {
                allocation.signers = allocation.signers.add(signer);
                allocation.is_active = false;
                break;
            }
        }
        Self(res)
    }

    pub fn complete_allocation(&self, request_id: String) -> Self {
        let mut res: Vec<Allocation> = self.0.clone();
        for allocation in res.iter_mut() {
            if allocation.id == request_id && allocation.is_active {
                allocation.is_active = false;
            }
        }
        Self(res)
    }

    pub fn push(&mut self, request: AllocationRequest) -> Self {
        let allocation = Allocation::new(request);
        self.0.push(allocation);
        self.clone()
    }

    pub fn total_requested(&self) -> u64 {
        let mut total_amount: u64 = 0;
        for allocation in self.0.iter() {
            if let Some(amount) = parse_size_to_bytes(&allocation.amount) {
                total_amount += amount;
            }
        }
        total_amount
    }
}
