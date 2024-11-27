use std::str::FromStr;

use markdown::{mdast::Node, to_mdast, ParseOptions};
use serde::{Deserialize, Serialize};

use crate::{
    config::get_env_var_or_default,
    core::application::file::{Client, DataType, Datacap, DatacapGroup, Project, Version},
    error::LDNError,
};

#[derive(Serialize, Deserialize, Debug)]
pub enum ParsedApplicationDataFields {
    Version,
    Address,
    // Client Info
    Name,
    Region,
    Industry,
    Website,
    SocialMedia,
    SocialMediaType,
    Role,
    // Project Info
    ProjectID,
    ProjectBriefHistory,
    AssociatedProjects,
    DataDesc,
    DataSrc,
    DataPrepare,
    DataSampleLink,
    ConfirmPublicDataset,
    RetrivalFreq,
    DataLifeSpan,
    DataGeographies,
    DataDistribution,
    ProviderIDs,
    FilplusGuideline,
    // Datacap Info
    DatacapGroup,
    Type,
    TotalRequestedAmount,
    SingleSizeDataset,
    Replicas,
    WeeklyAllocation,
    CustomMultisig,
    Identifier,
    InvalidField,
}

impl From<String> for ParsedApplicationDataFields {
    fn from(s: String) -> Self {
        match s.as_str() {
	  "On-chain address for first allocation" => ParsedApplicationDataFields::Address,
	  // Client Info
	  "Data Owner Name" => ParsedApplicationDataFields::Name,
	  "Data Owner Country/Region" => ParsedApplicationDataFields::Region,
	  "Data Owner Industry" => ParsedApplicationDataFields::Industry,
	  "Dataset Name" => ParsedApplicationDataFields::Name,
	  "Dataset Country/Region" => ParsedApplicationDataFields::Region,
	  "Dataset Industry" => ParsedApplicationDataFields::Industry,
	  "Website" => ParsedApplicationDataFields::Website,
	  "Social Media Handle" => ParsedApplicationDataFields::SocialMedia,
	  "Social Media Type" => ParsedApplicationDataFields::SocialMediaType,
	  "What is your role related to the dataset" => ParsedApplicationDataFields::Role,
	  // Project Info
      "Share a brief history of your project and organization" => {
		ParsedApplicationDataFields::ProjectBriefHistory
	  }
	  "Is this project associated with other projects/ecosystem stakeholders?" => {
		ParsedApplicationDataFields::AssociatedProjects
	  }
	  "Describe the data being stored onto Filecoin" => {
		ParsedApplicationDataFields::DataDesc
	  },
	  "Where was the data currently stored in this dataset sourced from"=> {
		ParsedApplicationDataFields::DataSrc
	  },
	  "If you are a data preparer, how will the data be prepared? Please include tooling used and technical details?" => {
		ParsedApplicationDataFields::DataPrepare
	  },
	  "Please share a sample of the data" => {
		ParsedApplicationDataFields::DataSampleLink
	  },
	  "Confirm that this is a public dataset that can be retrieved by anyone on the Network" => {
		ParsedApplicationDataFields::ConfirmPublicDataset
	  },
	  "What is the expected retrieval frequency for this data" => {
		ParsedApplicationDataFields::RetrivalFreq
	  },
	  "For how long do you plan to keep this dataset stored on Filecoin" => {
		ParsedApplicationDataFields::DataLifeSpan
	  },
	  "In which geographies do you plan on making storage deals" => {
		ParsedApplicationDataFields::DataGeographies
	  },
	  "How will you be distributing your data to storage providers" => {
		ParsedApplicationDataFields::DataDistribution
	  },
	  "Please list the provider IDs and location of the storage providers you will be working with." => {
		ParsedApplicationDataFields::ProviderIDs
	  },
      "Can you confirm that you will follow the Fil+ guideline" => {
		ParsedApplicationDataFields::FilplusGuideline
	  },
	  // Datacap info
	  "Group" => ParsedApplicationDataFields::DatacapGroup,
	  "Type" => ParsedApplicationDataFields::Type,
	  "Total amount of DataCap being requested" => ParsedApplicationDataFields::TotalRequestedAmount,
	  "Expected size of single dataset (one copy)" => ParsedApplicationDataFields::SingleSizeDataset,
	  "Number of replicas to store" => ParsedApplicationDataFields::Replicas,
	  "Weekly allocation of DataCap requested" => ParsedApplicationDataFields::WeeklyAllocation,
	  "Custom multisig" => ParsedApplicationDataFields::CustomMultisig,
	  "Identifier" => ParsedApplicationDataFields::Identifier,
	  // Invalid field
	  _ => ParsedApplicationDataFields::InvalidField,
	}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ParsedIssue {
    pub version: Version,
    pub id: String,
    pub client: Client,
    pub project: Project,
    pub datacap: Datacap,
}

impl ParsedIssue {
    pub fn from_issue_body(body: &str) -> Result<Self, LDNError> {
        let tree: Node = to_mdast(body, &ParseOptions::default())
            .map_err(|e| LDNError::Load(format!("Failed to get node: {}", e)))?;
        let mut data: IssueValidData = IssueValidData::default();
        let children = tree.children().ok_or(LDNError::Load(
            "Failed to get children from node.".to_string(),
        ))?;
        let child_iter = children.iter();

        for chunk in child_iter.collect::<Vec<_>>().chunks_exact(2) {
            if let (Some(prop_node), Some(value_node)) = (chunk.first(), chunk.get(1)) {
                let prop = prop_node.to_string();
                let value = value_node.to_string();

                match prop.clone().into() {
                    ParsedApplicationDataFields::InvalidField => {
                        continue;
                    }
                    _ => data.0.push((Prop(prop), Value(value))),
                }
            }
        }
        let client = Client::from(data.clone());
        let project = Project::from(data.clone());
        let datacap = Datacap::try_from(data.clone())?;
        let id = data
            .0
            .into_iter()
            .find(|(prop, _)| prop.0 == "On-chain address for first allocation")
            .ok_or(LDNError::Load(
                "Failed to get on-chain address for first allocation.".to_string(),
            ))?
            .1
             .0;

        let version = get_env_var_or_default("ISSUE_TEMPLATE_VERSION");

        Ok(Self {
            id,
            version: Version::Text(version),
            client,
            project,
            datacap,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Prop(pub String);
#[derive(Debug, Clone, Default)]
pub struct Value(pub String);

#[derive(Debug, Clone, Default)]
pub struct IssueValidData(pub Vec<(Prop, Value)>);

impl From<IssueValidData> for Project {
    fn from(data: IssueValidData) -> Self {
        let mut project = Project::default();
        for (prop, value) in data.0 {
            match prop.0.into() {
                ParsedApplicationDataFields::ProjectBriefHistory => {
                    project.history = value.0;
                }
                ParsedApplicationDataFields::AssociatedProjects => {
                    project.associated_projects = value.0;
                }
                ParsedApplicationDataFields::DataDesc => {
                    project.stored_data_desc = value.0;
                }
                ParsedApplicationDataFields::DataSrc => {
                    project.previous_stoarge = value.0;
                }
                ParsedApplicationDataFields::DataPrepare => {
                    project.dataset_prepare = value.0;
                }
                ParsedApplicationDataFields::DataSampleLink => {
                    project.data_sample_link = value.0;
                }
                ParsedApplicationDataFields::ConfirmPublicDataset => {
                    project.public_dataset = value.0;
                }
                ParsedApplicationDataFields::RetrivalFreq => {
                    project.retrival_frequency = value.0;
                }
                ParsedApplicationDataFields::DataLifeSpan => {
                    project.dataset_life_span = value.0;
                }
                ParsedApplicationDataFields::DataGeographies => {
                    project.geographis = value.0;
                }
                ParsedApplicationDataFields::DataDistribution => {
                    project.distribution = value.0;
                }
                ParsedApplicationDataFields::ProviderIDs => {
                    project.providers = value.0;
                }
                ParsedApplicationDataFields::FilplusGuideline => {
                    project.filplus_guideline = value.0;
                }
                _ => {}
            }
        }
        project
    }
}

impl From<IssueValidData> for Client {
    fn from(data: IssueValidData) -> Self {
        let mut client = Client::default();
        for (prop, value) in data.0 {
            match prop.0.into() {
                ParsedApplicationDataFields::Name => {
                    client.name = value.0;
                }
                ParsedApplicationDataFields::Region => {
                    client.region = value.0;
                }
                ParsedApplicationDataFields::Industry => {
                    client.industry = value.0;
                }
                ParsedApplicationDataFields::Website => {
                    client.website = value.0;
                }
                ParsedApplicationDataFields::SocialMedia => {
                    client.social_media = value.0;
                }
                ParsedApplicationDataFields::SocialMediaType => {
                    client.social_media_type = value.0;
                }
                ParsedApplicationDataFields::Role => {
                    client.role = value.0;
                }
                _ => {}
            }
        }
        client
    }
}

impl TryFrom<IssueValidData> for Datacap {
    type Error = LDNError;
    fn try_from(data: IssueValidData) -> Result<Self, Self::Error> {
        let mut datacap = Datacap::default();
        for (prop, value) in data.0 {
            match prop.0.into() {
                ParsedApplicationDataFields::DatacapGroup => {
                    datacap._group = DatacapGroup::from_str(&value.0).map_err(|e| {
                        LDNError::Load(format!("Failed to get DataCap group: {}", e))
                    })?;
                }
                ParsedApplicationDataFields::Type => {
                    datacap.data_type = DataType::from_str(&value.0).map_err(|e| {
                        LDNError::Load(format!("Failed to get DataCap type: {}", e))
                    })?;
                }
                ParsedApplicationDataFields::TotalRequestedAmount => {
                    datacap.total_requested_amount = value.0;
                }
                ParsedApplicationDataFields::SingleSizeDataset => {
                    datacap.single_size_dataset = value.0;
                }
                ParsedApplicationDataFields::Replicas => {
                    datacap.replicas = value.0.parse::<u8>().map_err(|e| {
                        LDNError::Load(format!("Failed to parse replicas to u8: {}", e))
                    })?;
                }
                ParsedApplicationDataFields::WeeklyAllocation => {
                    datacap.weekly_allocation = value.0;
                }
                ParsedApplicationDataFields::CustomMultisig => {
                    datacap.custom_multisig = value.0;
                }
                ParsedApplicationDataFields::Identifier => {
                    datacap.identifier = value.0;
                }
                _ => {}
            }
        }
        log::info!("Datacap: {:?}", datacap);

        Ok(datacap)
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::external_services::github::github_async_new;

//     static OWNER: &str = "keyko-io";
//     static REPO: &str = "test-philip-second";

//     #[tokio::test]
//     async fn test_parser() {
//         let _ = fplus_database::setup().await;
//         let gh = github_async_new(OWNER.to_string(), REPO.to_string()).await;
//         let issue = gh.list_issue(37).await.unwrap();
//         let parsed_ldn = super::ParsedIssue::from_issue_body(&issue.body.unwrap());
//         dbg!(&parsed_ldn);

//         assert_eq!(parsed_ldn.version, 1);
//         assert!(!parsed_ldn.id.is_empty());

//         assert!(!parsed_ldn.client.name.is_empty());
//         assert!(!parsed_ldn.client.industry.is_empty());
//         assert!(!parsed_ldn.client.region.is_empty());
//         assert!(!parsed_ldn.client.website.is_empty());
//         assert!(!parsed_ldn.client.social_media.is_empty());
//         assert!(!parsed_ldn.client.social_media_type.is_empty());
//         assert!(!parsed_ldn.client.role.is_empty());
//         assert!(!parsed_ldn.project.history.is_empty());
//         assert!(!parsed_ldn.project.associated_projects.is_empty());

//         assert!(!parsed_ldn.datacap.total_requested_amount.is_empty());
//     }
// }
