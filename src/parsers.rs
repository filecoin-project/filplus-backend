use crate::core::ParsedApplicationDataFields;
use markdown::{mdast::Node, to_mdast, ParseOptions};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ParsedLDN {
    pub name: String,
    pub region: String,
    pub website: String,
    pub datacap_requested: String,
    pub datacap_weekly_allocation: String,
    pub address: String,
    // pub custom_notary: String,
    pub identifier: String,
    pub data_type: String,
}

pub fn parse_ldn_app_body(body: &str) -> ParsedLDN {
    let tree: Node = to_mdast(body, &ParseOptions::default()).unwrap();
    let mut name: Option<String> = None;
    let mut website: Option<String> = None;
    // let mut custom_notary: Option<String> = None;
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
            // ParsedApplicationDataFields::CustomNotary => {
            //     custom_notary = Some(value);
            // }
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
        // custom_notary: custom_notary.unwrap_or_else(|| "No Custom Notary".to_string()),
        identifier: identifier.unwrap_or_else(|| "No Identifier".to_string()),
        data_type: data_type.unwrap_or_else(|| "No Data Type".to_string()),
    };
    parsed_ldn
}

#[cfg(test)]
mod tests {
    // use crate::github::GithubWrapper;

    // use super::*;

    // #[tokio::test]
    // async fn test_parse_ldn_app_body() {
    //     let gh = GithubWrapper::new();
    //     let issue = gh.list_issue(2).await.unwrap();
    //     let parsed_ldn = parse_ldn_app_body(&issue.body.unwrap());
    //     assert_eq!(parsed_ldn.name, "ciao");
    //     assert_eq!(parsed_ldn.region, "Afghanistan");
    //     assert_eq!(parsed_ldn.website, "ciao");
    //     assert_eq!(parsed_ldn.data_type, "Public, Open Commercial/Enterprise");
    //     assert_eq!(parsed_ldn.datacap_requested, "100TiB");
    //     assert_eq!(parsed_ldn.datacap_weekly_allocation, "10TiB");
    //     assert_eq!(
    //         parsed_ldn.address,
    //         "f1t476jko3mh67btetymtgyaysw4stj5b5hfv46bq"
    //     );
    //     // assert_eq!(
    //     // parsed_ldn.custom_notary,
    //     // "f1t476jko3mh67btetymtgyaysw4stj5b5hfv46bq"
    //     // );
    //     assert_eq!(parsed_ldn.identifier, "No response");
    // }
}
