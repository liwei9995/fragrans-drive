//! API JSON 响应序列化：_id 为 hex 字符串，时间为毫秒时间戳字符串。

use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Serializer};

pub fn serialize_object_id_as_hex<S>(
    id: &Option<ObjectId>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match id {
        Some(oid) => oid.to_string().serialize(serializer),
        None => serializer.serialize_none(),
    }
}

pub fn serialize_optional_datetime_as_ms_string<S>(
    dt: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match dt {
        Some(d) => d.timestamp_millis().to_string().serialize(serializer),
        None => serializer.serialize_none(),
    }
}

/// 序列化为 RFC 3339 字符串（如 "2023-03-22T16:48:12.233Z"），用于 storage list 等接口。
pub fn serialize_optional_datetime_as_rfc3339<S>(
    dt: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match dt {
        Some(d) => d.to_rfc3339().serialize(serializer),
        None => serializer.serialize_none(),
    }
}
