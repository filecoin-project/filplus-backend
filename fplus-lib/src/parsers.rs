use std::str::FromStr;

use markdown::{mdast::Node, to_mdast, ParseOptions};
use serde::{Deserialize, Serialize};

use crate::core::application::file::{Client, DataType, Datacap, DatacapGroup, Project};

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
    TotalRequestedType,
    SingleSizeDatasetAmount,
    SingleSizeDatasetType,
    Replicas,
    CustomMultisig,
    Identifier,
    WeeklyAllocationAmount,
    WeeklyAllocationType,
    InvalidField,
}

impl From<String> for ParsedApplicationDataFields {
    fn from(s: String) -> Self {
        match s.as_str() {
	  "Version" => ParsedApplicationDataFields::Version,
	  "On Chain Address" => ParsedApplicationDataFields::Address,
	  // Client Info
	  "Name" => ParsedApplicationDataFields::Name,
	  "Region" => ParsedApplicationDataFields::Region,
	  "Industry" => ParsedApplicationDataFields::Industry,
	  "Website" => ParsedApplicationDataFields::Website,
	  "Social Media" => ParsedApplicationDataFields::SocialMedia,
	  "Social Media Type" => ParsedApplicationDataFields::SocialMediaType,
	  "Role" => ParsedApplicationDataFields::Role,
	  // Project Info
	  "Project ID" => ParsedApplicationDataFields::ProjectID,
	  "Brief history of your project and organization" => {
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
	  "How do you plan to prepare the dataset" => {
		ParsedApplicationDataFields::DataPrepare
	  },
	  "Please share a sample of the data (a link to a file, an image, a table, etc., are good ways to do this." => {
		ParsedApplicationDataFields::DataSampleLink
	  },
	  "Confirm that this is a public dataset that can be retrieved by anyone on the network (i.e., no specific permissions or access rights are required to view the data)" => {
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
	  "Please list the provider IDs and location of the storage providers you will be working with. Note that it is a requirement to list a minimum of 5 unique provider IDs, and that your client address will be verified against this list in the future" => {
		ParsedApplicationDataFields::ProviderIDs
	  },
      "Can you confirm that you will follow the Fil+ guideline (Data owner should engage at least 4 SPs and no single SP ID should receive >30% of a client's allocated DataCap)" => {
		ParsedApplicationDataFields::FilplusGuideline
	  },
	  // Datacap info
	  "Group" => ParsedApplicationDataFields::DatacapGroup,
	  "Type" => ParsedApplicationDataFields::Type,
	  "Total Requested Amount" => ParsedApplicationDataFields::TotalRequestedAmount,
      "Total Requested Type" => ParsedApplicationDataFields::TotalRequestedType,
	  "Single Size Dataset Amount" => ParsedApplicationDataFields::SingleSizeDatasetAmount,
      "Single Size Dataset Type" => ParsedApplicationDataFields::SingleSizeDatasetType,
	  "Replicas" => ParsedApplicationDataFields::Replicas,
	  "Custom multisig" => ParsedApplicationDataFields::CustomMultisig,
	  "Identifier" => ParsedApplicationDataFields::Identifier,
      "Weekly Allocation Amount" => ParsedApplicationDataFields::WeeklyAllocationAmount,
      "Weekly Allocation Type" => ParsedApplicationDataFields::WeeklyAllocationType,
	  // Invalid field
	  _ => ParsedApplicationDataFields::InvalidField,
	}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ParsedIssue {
    pub version: u8,
    pub id: String,
    pub client: Client,
    pub project: Project,
    pub datacap: Datacap,
}

impl ParsedIssue {
    pub fn from_issue_body(body: &str) -> Self {
        let tree: Node = to_mdast(body, &ParseOptions::default()).unwrap();
        let mut data: IssueValidData = IssueValidData::default();
        let children = tree.children().unwrap();
        let child_iter = children.iter();
        
        for chunk in child_iter.collect::<Vec<_>>().chunks_exact(2) {
            if let (Some(prop_node), Some(value_node)) = (chunk.get(0), chunk.get(1)) {
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
        let datacap = Datacap::from(data.clone());
        let version = data
            .clone()
            .0
            .into_iter()
            .find(|(prop, _)| prop.0 == "Version")
            .unwrap()
            .1
             .0
            .parse::<u8>()
            .unwrap();
        let id = data
            .0
            .into_iter()
            .find(|(prop, _)| prop.0 == "On Chain Address")
            .unwrap()
            .1
             .0;

        Self {
            id,
            version,
            client,
            project,
            datacap,
        }
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
                ParsedApplicationDataFields::ProjectID => {
                    project.project_id = value.0;
                }
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

impl From<IssueValidData> for Datacap {
    fn from(data: IssueValidData) -> Self {
        let mut datacap = Datacap::default();
        for (prop, value) in data.0 {
            match prop.0.into() {
                ParsedApplicationDataFields::DatacapGroup => {
                    datacap._group = DatacapGroup::from_str(&value.0).unwrap();
                }
                ParsedApplicationDataFields::Type => {
                    datacap.data_type = DataType::from_str(&value.0).unwrap();
                }
                ParsedApplicationDataFields::TotalRequestedType => {
                    datacap.total_requested_amount = value.0;
                }
                ParsedApplicationDataFields::TotalRequestedAmount => {
                    datacap.total_requested_amount = value.0 + &datacap.total_requested_amount;
                }
                ParsedApplicationDataFields::SingleSizeDatasetType => {
                    datacap.single_size_dataset = value.0;
                }
                ParsedApplicationDataFields::SingleSizeDatasetAmount => {
                    datacap.single_size_dataset = value.0 + &datacap.single_size_dataset;
                }
                ParsedApplicationDataFields::Replicas => {
                    datacap.replicas = value.0.parse::<u8>().unwrap();
                }
                ParsedApplicationDataFields::WeeklyAllocationType => {
                    datacap.weekly_allocation = value.0;
                }
                ParsedApplicationDataFields::WeeklyAllocationAmount => {
                    datacap.weekly_allocation = value.0 + &datacap.weekly_allocation;
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
        datacap
    }
}

#[cfg(test)]
mod tests {
    use crate::external_services::github::GithubWrapper;

    #[tokio::test]
    async fn test_parser() {
        let gh = GithubWrapper::new();
        let issue = gh.list_issue(471).await.unwrap();
        let parsed_ldn = super::ParsedIssue::from_issue_body(&issue.body.unwrap());

        assert_eq!(parsed_ldn.version, 1);
        assert!(!parsed_ldn.id.is_empty());

        assert!(!parsed_ldn.client.name.is_empty());
        assert!(!parsed_ldn.client.industry.is_empty());
        assert!(!parsed_ldn.client.region.is_empty());
        assert!(!parsed_ldn.client.website.is_empty());
        assert!(!parsed_ldn.client.social_media.is_empty());
        assert!(!parsed_ldn.client.social_media_type.is_empty());
        assert!(!parsed_ldn.client.role.is_empty());
        assert!(!parsed_ldn.project.project_id.is_empty());
        assert!(!parsed_ldn.project.history.is_empty());
        assert!(!parsed_ldn.project.associated_projects.is_empty());

        assert!(!parsed_ldn.datacap.total_requested_amount.is_empty());
    }
}
