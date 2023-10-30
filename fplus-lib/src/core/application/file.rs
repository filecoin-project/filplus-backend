use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DatacapGroup {
    #[serde(rename = "da")]
    DA,
    #[serde(rename = "ldn-v3")]
    LDN,
    #[serde(rename = "e-fil")]
    EFIL,
}

impl FromStr for DatacapGroup {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "da" => Ok(Self::DA),
            "ldn-v3" => Ok(Self::LDN),
            "e-fil" => Ok(Self::EFIL),
            _ => Err(format!("{} is not a valid datacap group", s)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApplicationFile {
    #[serde(rename = "Version")]
    pub version: u8,
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Issue Number")]
    pub issue_number: String,
    #[serde(rename = "Client")]
    pub client: Client,
    #[serde(rename = "Project")]
    pub project: Project,
    #[serde(rename = "Datacap")]
    pub datacap: Datacap,
    #[serde(rename = "Lifecycle")]
    pub lifecycle: LifeCycle,
    #[serde(rename = "Allocation Requests")]
    pub allocation: Allocations,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Client {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Region")]
    pub region: String,
    #[serde(rename = "Industry")]
    pub industry: String,
    #[serde(rename = "Website")]
    pub website: String,
    #[serde(rename = "Social Media")]
    pub social_media: String,
    #[serde(rename = "Role")]
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Datacap {
    #[serde(rename = "Type")]
    pub _group: DatacapGroup,
    #[serde(rename = "Data Type")]
    pub data_type: DataType,
    #[serde(rename = "Total requested amount")]
    pub total_requested_amount: String,
    #[serde(rename = "Single size dataset")]
    pub single_size_dataset: String,
    #[serde(rename = "Replicas")]
    pub replicas: u8,
    #[serde(rename = "Weekly Allocation")]
    pub weekly_allocation: String,
}

impl Default for Datacap {
    fn default() -> Self {
        Self {
            _group: DatacapGroup::LDN,
            data_type: DataType::Slingshot,
            total_requested_amount: "0".to_string(),
            single_size_dataset: "0".to_string(),
            replicas: 0,
            weekly_allocation: "0".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DataType {
    #[serde(rename = "Slingshot")]
    Slingshot,
    #[serde(rename = "Public, Open Dataset (Research/Non-Profit)")]
    PublicOpenDatasetResearchNonProfit,
    #[serde(rename = "Public, Open Commercial/Enterprise")]
    PublicOpenCommercialEnterprise,
    #[serde(rename = "Private Commercial/Enterprise")]
    PrivateCommercialEnterprise,
    #[serde(rename = "Private Non-Profit / Social impact")]
    PrivateNonProfitSocialImpact,
}

impl FromStr for DataType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Slingshot" => Ok(Self::Slingshot),
            "Public, Open Dataset (Research/Non-Profit)" => {
                Ok(Self::PublicOpenDatasetResearchNonProfit)
            }
            "Public, Open Commercial/Enterprise" => Ok(Self::PublicOpenCommercialEnterprise),
            "Private Commercial/Enterprise" => Ok(Self::PrivateCommercialEnterprise),
            "Private Non-Profit / Social impact" => Ok(Self::PrivateNonProfitSocialImpact),
            _ => Err(format!("{} is not a valid data type", s)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AssociatedProjects {
    Yes(String),
    No,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PublicDataset {
    Yes,
    No(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RetrivalFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Sporadic,
    Never,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StorageModularity {
    IPFS,
    Lotus,
    Singularity,
    Graphsplit,
    Other(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StorageProviders {
    AWSCloud,
    GoogleCloud,
    AzureCloud,
    InternalStorage,
    Other(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Project {
    #[serde(rename = "Project Id")]
    pub project_id: String,
    #[serde(rename = "Brief history of your project and organization")]
    pub history: String,
    #[serde(rename = "Is this project associated with other projects/ecosystem stakeholders?")]
    pub associated_projects: String,
    #[serde(rename = "Describe the data being stored onto Filecoin")]
    pub stored_data_desc: String,
    #[serde(rename = "Where was the data currently stored in this dataset sourced from} ")]
    pub previous_stoarge: String,
    #[serde(rename = "How do you plan to prepare the dataset")]
    pub dataset_prepare: String,
    #[serde(
        rename = "Please share a sample of the data (a link to a file, an image, a table, etc., are good ways to do this.)"
    )]
    pub data_sample_link: String,
    #[serde(
        rename = "Confirm that this is a public dataset that can be retrieved by anyone on the network (i.e., no specific permissions or access rights are required to view the data)"
    )]
    pub public_dataset: String,
    #[serde(rename = "What is the expected retrieval frequency for this data")]
    pub retrival_frequency: String,
    #[serde(rename = "For how long do you plan to keep this dataset stored on Filecoin")]
    pub dataset_life_span: String,
    #[serde(rename = "In which geographies do you plan on making storage deals")]
    pub geographis: String,
    #[serde(rename = "How will you be distributing your data to storage providers")]
    pub distribution: String,
    #[serde(
        rename = "Please list the provider IDs and location of the storage providers you will be working with. Note that it is a requirement to list a minimum of 5 unique provider IDs, and that your client address will be verified against this list in the future"
    )]
    pub providers: String,
    #[serde(
        rename = "Can you confirm that you will follow the Fil+ guideline (Data owner should engage at least 4 SPs and no single SP ID should receive >30% of a client's allocated DataCap)"
    )]
    pub filplus_guideline: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DatasetLifeSpan {
    LessThanAYear,
    OneToOneAndHalfYears,
    OneAndHalfToTwoYears,
    TwoToThreeYears,
    MoreThanThreeYears,
    Permanently,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Provider {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Location")]
    pub location: String,
    #[serde(rename = "SPOrg")]
    pub spo_org: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AppState {
    Submitted,
    GovernanceReview,
    ReadyToSign,
    StartSignDatacap,
    Granted,
    TotalDatacapReached,
    Error,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LifeCycle {
    #[serde(rename = "State")]
    pub state: AppState,
    #[serde(rename = "Validated At")]
    pub validated_at: String,
    #[serde(rename = "Validated By")]
    pub validated_by: String,
    #[serde(rename = "Active")]
    pub is_active: bool,
    #[serde(rename = "Updated At")]
    pub updated_at: String,
    #[serde(rename = "Active Request ID")]
    pub active_request: Option<String>,
    #[serde(rename = "On Chain Address")]
    pub client_on_chain_address: String,
    #[serde(rename = "Multisig Address")]
    pub multisig_address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Allocations(pub Vec<Allocation>);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum AllocationRequestType {
    First,
    Removal,
    Refill(u8),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Allocation {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Request Type")]
    pub request_type: AllocationRequestType,
    #[serde(rename = "Created At")]
    pub created_at: String,
    #[serde(rename = "Updated At")]
    pub updated_at: String,
    #[serde(rename = "Active")]
    pub is_active: bool,
    #[serde(rename = "Allocation Amount")]
    pub amount: String,
    #[serde(rename = "Signers")]
    pub signers: Notaries,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Notaries(pub Vec<Notary>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Notary {
    #[serde(rename(serialize = "Github Username"))]
    pub github_username: String,
    #[serde(rename(serialize = "Signing Address"))]
    pub signing_address: String,
    #[serde(rename(serialize = "Created At"))]
    pub created_at: String,
    #[serde(rename(serialize = "Message CID"))]
    pub message_cid: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AllocationRequest {
    pub actor: String,
    pub id: String,
    pub kind: AllocationRequestType,
    pub is_active: bool,
    pub allocation_amount: String,
}