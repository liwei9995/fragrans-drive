//! Serialize/deserialize `Option<DateTime<Utc>>` for both BSON DateTime and RFC 3339 strings.
//! Legacy rows may store strings; new rows use BSON DateTime.

use chrono::{DateTime, Utc};
use mongodb::bson::Bson;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<Bson>::deserialize(deserializer)?;
    let Some(bson) = opt else { return Ok(None) };
    match bson {
        Bson::DateTime(dt) => Ok(Some(dt.to_chrono())),
        Bson::String(s) => s
            .parse()
            .map(Some)
            .map_err(|e: chrono::ParseError| serde::de::Error::custom(e.to_string())),
        Bson::Null => Ok(None),
        other => Err(serde::de::Error::custom(format!(
            "expected DateTime or RFC3339 string, got {:?}",
            other
        ))),
    }
}

pub fn serialize<S>(val: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let bson = val.map(mongodb::bson::DateTime::from_chrono);
    bson.serialize(serializer)
}
