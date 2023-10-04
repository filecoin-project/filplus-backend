use chrono::prelude::*;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApplicationLifecycle {
    state: ApplicationFileState,
    pub validated_time: String,
    pub initial_pr_number: u64,
    pub validated_by: String,
    pub first_allocation_time: String,
    pub is_active: bool,
    pub time_of_new_state: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ApplicationFileState {
    Validation,
    GovernanceReview,
    Proposal,
    Approval,
    Confirmed,
}

impl ApplicationFileState {
    pub fn as_str(&self) -> &str {
        match *self {
            ApplicationFileState::Validation => "Validation",
            ApplicationFileState::GovernanceReview => "Governance Review",
            ApplicationFileState::Proposal => "Proposal",
            ApplicationFileState::Approval => "Approval",
            ApplicationFileState::Confirmed => "Confirmed",
        }
    }
}

impl ApplicationLifecycle {
    pub fn governance_review_state(pr_number: u64) -> Self {
        ApplicationLifecycle {
            state: ApplicationFileState::GovernanceReview,
            validated_time: Utc::now().to_string(),
            first_allocation_time: "".to_string(),
            validated_by: "".to_string(),
            is_active: true,
            initial_pr_number: pr_number,
            time_of_new_state: Utc::now().to_string(),
        }
    }

    /// Change Application state to Proposal from Governance Review
    /// Actor input is the actor who is changing the state
    pub fn set_proposal_state(&self, actor: String) -> Self {
        ApplicationLifecycle {
            state: ApplicationFileState::Proposal,
            validated_time: self.validated_time.clone(),
            first_allocation_time: "".to_string(),
            validated_by: actor,
            is_active: true,
            initial_pr_number: self.initial_pr_number.clone(),
            time_of_new_state: Utc::now().to_string(),
        }
    }

    pub fn set_approval_state(&self) -> Self {
        ApplicationLifecycle {
            state: ApplicationFileState::Approval,
            validated_time: self.validated_time.clone(),
            validated_by: self.validated_by.clone(),
            first_allocation_time: "".to_string(),
            is_active: true,
            time_of_new_state: Utc::now().to_string(),
            initial_pr_number: self.initial_pr_number.clone(),
        }
    }

    pub fn set_confirmed_state(&self) -> Self {
        ApplicationLifecycle {
            state: ApplicationFileState::Confirmed,
            validated_time: self.validated_time.clone(),
            validated_by: self.validated_by.clone(),
            first_allocation_time: Utc::now().to_string(),
            is_active: false,
            time_of_new_state: Utc::now().to_string(),
            initial_pr_number: self.initial_pr_number.clone(),
        }
    }

    pub fn get_state(&self) -> ApplicationFileState {
        let res = self.state.clone();
        res
    }

    pub fn get_initial_pr_number(&self) -> u64 {
        let res = self.initial_pr_number.clone();
        res
    }
}
