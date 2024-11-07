use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifiedClientResponse {
    #[serde(deserialize_with = "number_to_string")]
    pub count: Option<String>,
}

fn number_to_string<'de, D>(de: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let helper: Value = Deserialize::deserialize(de)?;

    match helper {
        Value::Number(n) => Ok(n
            .as_u64()
            .filter(|&number| number != 0)
            .map(|_| n.to_string())),
        Value::String(s) => Ok(Some(s)),
        _ => Ok(None),
    }
}
