use super::{allocations::ApplicationAllocations, lifecycle::ApplicationLifecycle};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApplicationInfo {
    pub core_information: ApplicationCoreInfo,
    pub application_lifecycle: ApplicationLifecycle,
    pub datacap_allocations: ApplicationAllocations,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApplicationCoreInfo {
    pub data_owner_name: String,
    pub data_owner_github_handle: String,
    pub data_owner_region: String,
    pub data_owner_industry: String,
    pub data_owner_address: String,
    pub datacap_weekly_allocation: String,
    pub requested_amount: String,
    pub website: String,
    pub social_media: String,
}

impl ApplicationCoreInfo {
    pub fn new(
        data_owner_name: String,
        data_owner_region: String,
        data_owner_github_handle: String,
        data_owner_industry: String,
        data_owner_address: String,
        requested_amount: String,
        datacap_weekly_allocation: String,
        website: String,
        social_media: String,
    ) -> Self {
        ApplicationCoreInfo {
            data_owner_name,
            data_owner_region,
            data_owner_github_handle,
            data_owner_address,
            requested_amount,
            datacap_weekly_allocation,
            data_owner_industry,
            website,
            social_media,
        }
    }
}

impl ApplicationInfo {
    pub fn new(
        core_information: ApplicationCoreInfo,
        application_lifecycle: ApplicationLifecycle,
        datacap_allocations: ApplicationAllocations,
    ) -> Self {
        ApplicationInfo {
            core_information,
            application_lifecycle,
            datacap_allocations,
        }
    }
}
