use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DatacapType {
    #[serde(rename = "da")]
    DA,
    #[serde(rename = "ldn-v3")]
    LDN,
    #[serde(rename = "e-fil")]
    EFIL,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApplicationFile {
    #[serde(rename = "Version")]
    pub version: u8,
    #[serde(rename = "Project Id")]
    pub project_id: u8,
    #[serde(rename = "Applicant")]
    pub applicant: String,
    #[serde(rename = "Client")]
    pub client: Client,
    #[serde(rename = "Datacap")]
    pub datacap: Datacap,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SocialMedia {
    Slack(String),
    Twitter(String),
    Facebook(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DatacapAmount {
    GiB(u64),
    TiB(u64),
    PiB(u64),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub social_media: SocialMedia,
    #[serde(rename = "On Chain Address")]
    pub on_chain_address: String,
    #[serde(rename = "Role")]
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Datacap {
    #[serde(rename = "Type")]
    pub _type: DatacapType,
    #[serde(rename = "Data Type")]
    pub data_type: DataType,
    #[serde(rename = "Total requested amount")]
    pub total_requested_amount: DatacapAmount,
    #[serde(rename = "Single size dataset")]
    pub single_size_dataset: DatacapAmount,
    #[serde(rename = "Replicas")]
    pub replicas: u8,
    #[serde(rename = "Weekly Allocation")]
    pub weekly_allocation: DatacapAmount,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
enum AssociatedProjects {
    Yes(String),
    No,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum StorageProviders {
    AWSCloud,
    GoogleCloud,
    AzureCloud,
    InternalStorage,
    Other(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Project {
    #[serde(rename = "Brief history of your project and organization")]
    history: String,
    #[serde(rename = "Is this project associated with other projects/ecosystem stakeholders?")]
    associated_projects: AssociatedProjects,
    #[serde(rename = "Describe the data being stored onto Filecoin")]
    stored_data_desc: String,
    #[serde(rename = "Where was the data currently stored in this dataset sourced from} ")]
    previous_stoarge: StorageProviders,
}

// {
//   "applicationInfo": {
//     "coreInformation": {
//       "Custom multisig": ""
//     },
//     "useCaseDetails": {
//       "If you answered 'Other' in the previous question, enter the details here": "",
//       "How do you plan to prepare the dataset": "IPFS | Lotus | Singularity | Graphsplit | other/custom tool",
//       "If you answered 'other/custom tool' in the previous question, enter the details here": "",
//       "Please share a sample of the data (a link to a file, an image, a table, etc., are good ways to do this.)": "",
//       "Confirm that this is a public dataset that can be retrieved by anyone on the network (i.e., no specific permissions or access rights are required to view the data)": true,
//       "If you chose not to confirm, what was the reason": "",
//       "What is the expected retrieval frequency for this data": "Daily | Weekly | Monthly | Yearly | Sporadic | Never",
//       "For how long do you plan to keep this dataset stored on Filecoin": "Less than a year | 1 to 1.5 years | 1.5 to 2 years | 2 to 3 years | More than 3 years | Permanently"
//     },
//     "datacapAllocationPlan": {
//       "In which geographies do you plan on making storage deals": [
//       ],
//       "How will you be distributing your data to storage providers": "",
//       "How do you plan to choose storage providers": "",
//       "If you answered 'Other' in the previous question, what is the tool or platform you plan to use": "",
//       "Please list the provider IDs and location of the storage providers you will be working with. Note that it is a requirement to list a minimum of 5 unique provider IDs, and that your client address will be verified against this list in the future": [{"providerID": "", "location": "",  "SPOrg",""}],
//       "How do you plan to make deals to your storage providers": "",
//       "If you answered 'Others/custom tool' in the previous question, enter the details here": "",
//       "Can you confirm that you will follow the Fil+ guideline (Data owner should engage at least 4 SPs and no single SP ID should receive >30% of a client's allocated DataCap)": ""
//     }
//   },
//   "applicationLifecycle": {
//     "state": "submitted | ready to sign | start sign datacap | granted | total datacap reached | governance review needed | error",
//     "validatedTime": 0,
//     "firstAllocationTime": 0,
//     "isTrigered": false,
//     "isActive": true,
//     "timeOfNewState": 0
//   },
//   "dataCapAllocations": [
//     {
//       "dataCapTranche": {
//         "trancheID": 0,
//         "clientAddress": "f1...",
//         "timeOfRequest": 0,
//         "timeOfAllocation": 0,
//         "notaryAddress": "",
//         "allocationAmount": 0,
//         "signers": [
//           {
//             "githubUsername:" "",
//             "signingAddress": "",
//             "timeOfSignature": 0,
//             "messageCID": ""
//           },
//           {
//             "githubUsername:" "",
//             "signingAddress": "",
//             "timeOfSignature": 0,
//             "messageCID": ""
//           }
//         ],
//         "pr": 0,
//       }
//     }
//   ]
// }
