use std::str::FromStr;

use futures::stream::All;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
#[serde(untagged)]
pub enum Version {
    Number(u8),
    Text(String),
}

// DONT CHANGE ME UNLESS YOU HAVE GUN POINTED TO YOUR HEAD
// INCLUDES ALL THE NESTED OBJECTS, IE `CLIENT`, `PROJECT`, `DATACAP`, `LIFECYCLE`, `ALLOCATION`
//
// In occasions where you need to add new question or modify the ISSUE_TEMPLATE
// you should implemet a new struct, for example `ParsedClient` and then convert
// `ParsedClient` into `Client`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApplicationFile {
    #[serde(rename = "Version")]
    pub version: Version,
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
    #[serde(rename = "Social Media Type")]
    pub social_media_type: String,
    #[serde(rename = "Role")]
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Datacap {
    #[serde(rename = "Type")]
    pub _group: DatacapGroup,
    #[serde(rename = "Data Type")]
    pub data_type: DataType,
    #[serde(rename = "Total Requested Amount")]
    pub total_requested_amount: String,
    #[serde(rename = "Single Size Dataset")]
    pub single_size_dataset: String,
    #[serde(rename = "Replicas")]
    pub replicas: u8,
    #[serde(rename = "Weekly Allocation")]
    pub weekly_allocation: String,
    #[serde(rename = "Custom multisig", skip_serializing, default)]
    pub custom_multisig: String,
    #[serde(rename = "Identifier", skip_serializing, default)]
    pub identifier: String,
}

impl Default for Datacap {
    fn default() -> Self {
        Self {
            _group: DatacapGroup::LDN,
            data_type: DataType::Slingshot,
            total_requested_amount: "".to_string(),
            single_size_dataset: "".to_string(),
            replicas: 0,
            weekly_allocation: "".to_string(),
            custom_multisig: "".to_string(),
            identifier: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    #[serde(rename = "Brief history of your project and organization")]
    pub history: String,
    #[serde(rename = "Is this project associated with other projects/ecosystem stakeholders?")]
    pub associated_projects: String,
    #[serde(rename = "Describe the data being stored onto Filecoin")]
    pub stored_data_desc: String,
    #[serde(rename = "Where was the data currently stored in this dataset sourced from")]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum AppState {
    AdditionalInfoRequired,
    AdditionalInfoSubmitted,
    Submitted,
    KYCRequested,
    ChangesRequested,
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
    pub edited: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Allocations(pub Vec<Allocation>);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum AllocationRequestType {
    First,
    Removal,
    Refill(u8),
}

impl ToString for AllocationRequestType {
    fn to_string(&self) -> String {
        match self {
            AllocationRequestType::First => "First".to_string(),
            AllocationRequestType::Removal => "Removal".to_string(),
            AllocationRequestType::Refill(_) => "Refill".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Allocation {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Request Type")]
    pub request_type: String,
    #[serde(rename = "Created At")]
    pub created_at: String,
    #[serde(rename = "Updated At")]
    pub updated_at: String,
    #[serde(rename = "Active")]
    pub is_active: bool,
    #[serde(rename = "Allocation Amount")]
    pub amount: String,
    #[serde(rename = "Signers")]
    pub signers: Verifiers,
}

impl ApplicationFile {
    pub fn remove_active_allocation(&mut self) {
        self.allocation.0.retain(|alloc| !alloc.is_active);
        if self.allocation.0.len() == 0 {
            self.lifecycle.validated_at = "".to_string();
            self.lifecycle.validated_by = "".to_string();
            self.lifecycle.active_request = Some("".to_string());
        } else {
            self.lifecycle.active_request = Some(self.allocation.0[self.allocation.0.len() - 1].id.clone());
        }
    }

    pub fn get_active_allocation(&self) -> Option<&Allocation> {
        self.allocation.0.iter().find(|alloc| alloc.is_active)
    }

    pub fn adjust_active_allocation_amount(&mut self, new_amount: String) -> Result<(), &'static str> {
        // Find the first active allocation
        if let Some(allocation) = self.allocation.0.iter_mut().find(|alloc| alloc.is_active) {
            // Update the amount
            allocation.amount = new_amount;
            Ok(())
        } else {
            // Return an error if no active allocation is found
            Err("No active allocation found")
        }
    }

    pub fn get_last_request_allowance(&self) -> Option<Allocation> {
        let request_id = self.lifecycle.active_request.clone()?;
        self.allocation.find_one(request_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Verifiers(pub Vec<Verifier>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerifierInput {
    pub github_username: String,
    pub signing_address: String,
    pub created_at: String,
    pub message_cid: String,
}

impl From<VerifierInput> for Verifier {
    fn from(input: VerifierInput) -> Self {
        Self {
            github_username: input.github_username,
            signing_address: input.signing_address,
            created_at: input.created_at,
            message_cid: input.message_cid,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Verifier {
    #[serde(rename = "Github Username")]
    pub github_username: String,
    #[serde(rename = "Signing Address")]
    pub signing_address: String,
    #[serde(rename = "Created At")]
    pub created_at: String,
    #[serde(rename = "Message CID")]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidVerifierList {
    pub verifiers: Vec<String>,
}

impl ValidVerifierList {
    pub fn is_valid(&self, member: &str) -> bool {
        let lowercased_verifiers: Vec<String> = self.verifiers.iter().map(|v| v.to_lowercase()).collect();
        let lowercased_member = member.to_lowercase();
        lowercased_verifiers.contains(&lowercased_member)
    }
}

#[derive(Serialize)]
pub struct LDNActorsResponse {
    pub verifier_gh_handles: Vec<String>,
}


pub trait DeepCompare {
    fn compare(&self, other: &Self) -> Vec<String>;
}

impl DeepCompare for ApplicationFile {
    fn compare(&self, other: &Self) -> Vec<String> {
        let mut differences = Vec::new();
        if self.id != other.id {
            differences.push(format!("ID: {} vs {}", self.id, other.id));
        }
        if self.issue_number != other.issue_number {
            differences.push(format!("Issue Number: {} vs {}", self.issue_number, other.issue_number));
        }
        differences.append(&mut self.client.compare(&other.client));
        differences.append(&mut self.datacap.compare(&other.datacap));
        differences.append(&mut self.project.compare(&other.project));
        differences.append(&mut self.lifecycle.compare(&other.lifecycle));
        differences
    }
}

impl DeepCompare for Client {
    fn compare(&self, other: &Self) -> Vec<String> {
        let mut differences = Vec::new();
        if self.name != other.name {
            differences.push(format!("Client Name: {} vs {}", self.name, other.name));
        }
        if self.region != other.region {
            differences.push(format!("Client Region: {} vs {}", self.region, other.region));
        }
        if self.industry != other.industry {
            differences.push(format!("Client Industry: {} vs {}", self.industry, other.industry));
        }
        if self.website != other.website {
            differences.push(format!("Client Website: {} vs {}", self.website, other.website));
        }
        if self.social_media != other.social_media {
            differences.push(format!("Client Social Media: {} vs {}", self.social_media, other.social_media));
        }
        if self.social_media_type != other.social_media_type {
            differences.push(format!("Client Social Media Type: {} vs {}", self.social_media_type, other.social_media_type));
        }
        if self.role != other.role {
            differences.push(format!("Client Role: {} vs {}", self.role, other.role));
        }
        differences
    }
}

impl DeepCompare for Datacap {
    fn compare(&self, other: &Self) -> Vec<String> {
        let mut differences = Vec::new();
        if self._group != other._group {
            differences.push(format!("Datacap Group: {:?} vs {:?}", self._group, other._group));
        }
        if self.data_type != other.data_type {
            differences.push(format!("Datacap Group: {:?} vs {:?}", self._group, other._group));
        }
        if self.total_requested_amount != other.total_requested_amount {
            differences.push(format!("Total Requested Amount: {} vs {}", self.total_requested_amount, other.total_requested_amount));
        }
        if self.single_size_dataset != other.single_size_dataset {
            differences.push(format!("Single Size Dataset: {} vs {}", self.single_size_dataset, other.single_size_dataset));
        }
        if self.replicas != other.replicas {
            differences.push(format!("Replicas: {} vs {}", self.replicas, other.replicas));
        }
        if self.weekly_allocation != other.weekly_allocation {
            differences.push(format!("Weekly Allocation: {} vs {}", self.weekly_allocation, other.weekly_allocation));
        }
        if self.custom_multisig != other.custom_multisig {
            differences.push(format!("Custom Multisig: {} vs {}", self.custom_multisig, other.custom_multisig));
        }
        if self.identifier != other.identifier {
            differences.push(format!("Identifier: {} vs {}", self.identifier, other.identifier));
        }
        differences
    }
}

impl DeepCompare for Project {
    fn compare(&self, other: &Self) -> Vec<String> {
        let mut differences = Vec::new();
        if self.history != other.history {
            differences.push(format!("Brief history of your project and organization: {} vs {}", self.history, other.history));
        }
        if self.associated_projects != other.associated_projects {
            differences.push(format!("Is this project associated with other projects/ecosystem stakeholders?: {} vs {}", self.associated_projects, other.associated_projects));
        }
        if self.stored_data_desc != other.stored_data_desc {
            differences.push(format!("Describe the data being stored onto Filecoin: {} vs {}", self.stored_data_desc, other.stored_data_desc));
        }
        if self.previous_stoarge != other.previous_stoarge {
            differences.push(format!("Where was the data currently stored in this dataset sourced from: {} vs {}", self.previous_stoarge, other.previous_stoarge));
        }
        if self.dataset_prepare != other.dataset_prepare {
            differences.push(format!("How do you plan to prepare the dataset: {} vs {}", self.dataset_prepare, other.dataset_prepare));
        }
        if self.data_sample_link != other.data_sample_link {
            differences.push(format!("Please share a sample of the data: {} vs {}", self.data_sample_link, other.data_sample_link));
        }
        if self.public_dataset != other.public_dataset {
            differences.push(format!("Confirm that this is a public dataset that can be retrieved by anyone on the network: {} vs {}", self.public_dataset, other.public_dataset));
        }
        if self.retrival_frequency != other.retrival_frequency {
            differences.push(format!("What is the expected retrieval frequency for this data: {} vs {}", self.retrival_frequency, other.retrival_frequency));
        }
        if self.dataset_life_span != other.dataset_life_span {
            differences.push(format!("For how long do you plan to keep this dataset stored on Filecoin: {} vs {}", self.dataset_life_span, other.dataset_life_span));
        }
        if self.geographis != other.geographis {
            differences.push(format!("In which geographies do you plan on making storage deals: {} vs {}", self.geographis, other.geographis));
        }
        if self.distribution != other.distribution {
            differences.push(format!("How will you be distributing your data to storage providers: {} vs {}", self.distribution, other.distribution));
        }
        if self.providers != other.providers {
            differences.push(format!("Please list the provider IDs and location of the storage providers you will be working with: {} vs {}", self.providers, other.providers));
        }
        if self.filplus_guideline != other.filplus_guideline {
            differences.push(format!("Can you confirm that you will follow the Fil+ guideline: {} vs {}", self.filplus_guideline, other.filplus_guideline));
        }
        differences
    }
}

impl DeepCompare for LifeCycle {
    fn compare(&self, other: &Self) -> Vec<String> {
        let mut differences = Vec::new();
        if self.state != other.state {
            differences.push(format!("State: {:?} vs {:?}", self.state, other.state));
        }
        if self.validated_at != other.validated_at {
            differences.push(format!("Validated At: {} vs {}", self.validated_at, other.validated_at));
        }
        if self.validated_by != other.validated_by {
            differences.push(format!("Validated By: {} vs {}", self.validated_by, other.validated_by));
        }
        if self.is_active != other.is_active {
            differences.push(format!("Active: {} vs {}", self.is_active, other.is_active));
        }
        if self.updated_at != other.updated_at {
            differences.push(format!("Updated At: {} vs {}", self.updated_at, other.updated_at));
        }
        if self.active_request != other.active_request {
            differences.push(format!("Active Request ID: {:?} vs {:?}", self.active_request, other.active_request));
        }
        if self.client_on_chain_address != other.client_on_chain_address {
            differences.push(format!("On Chain Address: {} vs {}", self.client_on_chain_address, other.client_on_chain_address));
        }
        if self.multisig_address != other.multisig_address {
            differences.push(format!("Multisig Address: {} vs {}", self.multisig_address, other.multisig_address));
        }
        differences
    }
}