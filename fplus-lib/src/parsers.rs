use markdown::{mdast::Node, to_mdast, ParseOptions};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ParsedApplicationDataFields {
    Name,
    Region,
    Website,
    DatacapRequested,
    DatacapWeeklyAllocation,
    Address,
    Identifier,
    DataType,
    InvalidField,
}

impl From<String> for ParsedApplicationDataFields {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Data Owner Name" => ParsedApplicationDataFields::Name,
            "Data Owner Country/Region" => ParsedApplicationDataFields::Region,
            "Website" => ParsedApplicationDataFields::Website,
            // "Custom multisig" => ParsedApplicationDataFields::CustomNotary,
            "Identifier" => ParsedApplicationDataFields::Identifier,
            "Data Type of Application" => ParsedApplicationDataFields::DataType,
            "Total amount of DataCap being requested" => {
                ParsedApplicationDataFields::DatacapRequested
            }
            "Weekly allocation of DataCap requested" => {
                ParsedApplicationDataFields::DatacapWeeklyAllocation
            }
            "On-chain address for first allocation" => ParsedApplicationDataFields::Address,
            _ => ParsedApplicationDataFields::InvalidField,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ParsedLDN {
    pub name: String,
    pub region: String,
    pub website: String,
    pub datacap_requested: String,
    pub datacap_weekly_allocation: String,
    pub address: String,
    pub identifier: String,
    pub data_type: String,
}

pub fn parse_ldn_app_body(body: &str) -> ParsedLDN {
    let tree: Node = to_mdast(body, &ParseOptions::default()).unwrap();
    let mut name: Option<String> = None;
    let mut website: Option<String> = None;
    let mut data_type: Option<String> = None;
    let mut identifier: Option<String> = None;
    let mut datacap_weekly_allocation: Option<String> = None;
    let mut address: Option<String> = None;
    let mut datacap_requested: Option<String> = None;
    let mut region: Option<String> = None;
    for (index, i) in tree.children().unwrap().into_iter().enumerate().step_by(2) {
        let prop: ParsedApplicationDataFields = i.to_string().into();
        let tree = tree.children().unwrap().into_iter();
        let value = match tree.skip(index + 1).next() {
            Some(v) => v.to_string(),
            None => continue,
        };
        match prop {
            ParsedApplicationDataFields::Name => {
                name = Some(value);
            }
            ParsedApplicationDataFields::Region => {
                region = Some(value);
            }
            ParsedApplicationDataFields::Website => {
                website = Some(value);
            }
            ParsedApplicationDataFields::DatacapRequested => {
                datacap_requested = Some(value);
            }
            ParsedApplicationDataFields::DatacapWeeklyAllocation => {
                datacap_weekly_allocation = Some(value);
            }
            ParsedApplicationDataFields::Address => {
                address = Some(value);
            }
            ParsedApplicationDataFields::Identifier => {
                identifier = Some(value);
            }
            ParsedApplicationDataFields::DataType => {
                data_type = Some(value);
            }
            ParsedApplicationDataFields::InvalidField => {
                continue;
            }
        }
    }
    let parsed_ldn = ParsedLDN {
        name: name.unwrap_or_else(|| "No Name".to_string()),
        region: region.unwrap_or_else(|| "No Region".to_string()),
        website: website.unwrap_or_else(|| "No Website".to_string()),
        datacap_requested: datacap_requested.unwrap_or_else(|| "No Datacap Requested".to_string()),
        datacap_weekly_allocation: datacap_weekly_allocation
            .unwrap_or_else(|| "No Datacap Weekly Allocation".to_string()),
        address: address.unwrap_or_else(|| "No Address".to_string()),
        identifier: identifier.unwrap_or_else(|| "No Identifier".to_string()),
        data_type: data_type.unwrap_or_else(|| "No Data Type".to_string()),
    };
    parsed_ldn
}

#[cfg(test)]
mod tests {
    use crate::external_services::github::GithubWrapper;

    use super::*;

    #[tokio::test]
    async fn test_parser() {
        let gh = GithubWrapper::new();
        let issue = gh.list_issue(63).await.unwrap();
        let parsed_ldn = parse_ldn_app_body(&issue.body.unwrap());
        assert_eq!(parsed_ldn.name, "Stojan");
        assert_eq!(parsed_ldn.region, "Afghanistan");
        assert_eq!(parsed_ldn.website, "https://pangeo-data.github.io/pangeo-cmip6-cloud/");
        assert_eq!(parsed_ldn.data_type, "Public, Open Dataset (Research/Non-Profit)");
        assert_eq!(parsed_ldn.datacap_requested, "15PiB");
        assert_eq!(parsed_ldn.datacap_weekly_allocation, "1PiB");
        assert_eq!(
            parsed_ldn.address,
            "f1473tjqo3p5atezygb2koobcszvy5vftalcomcrq"
        );
        assert_eq!(parsed_ldn.identifier, "No response");
    }
}
