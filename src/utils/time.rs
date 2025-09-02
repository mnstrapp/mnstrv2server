//! Time-related utility functions for serialization and deserialization of `OffsetDateTime`.
//!
//! This module provides functions to handle the conversion between RFC 3339 formatted strings
//! and `OffsetDateTime` objects when working with serde serialization/deserialization.
//! It's particularly useful when dealing with JSON or other data formats that need to
//! represent timestamps.

use serde::{self, Deserialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// Serializes an `Option<OffsetDateTime>` into a string using RFC 3339 format.
///
/// # Arguments
///
/// * `date_time` - The optional timestamp to serialize
/// * `serializer` - The serializer to use
///
/// # Returns
///
/// Returns the serialized string if successful, or an error if the serialization fails.
///
/// # Example
///
/// ```
/// use serde_json::json;
/// use time::OffsetDateTime;
///
/// #[derive(Serialize)]
/// struct Timestamp {
///     #[serde(serialize_with = "serialize_offset_date_time")]
///     created_at: Option<OffsetDateTime>
/// }
/// ```
#[allow(unused)]
pub fn serialize_offset_date_time<S>(
    date_time: &Option<OffsetDateTime>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match date_time {
        Some(dt) => {
            serializer.serialize_str(&dt.format(&Rfc3339).map_err(serde::ser::Error::custom)?)
        }
        None => serializer.serialize_none(),
    }
}

/// Deserializes a string in RFC 3339 format into an `Option<OffsetDateTime>`.
///
/// # Arguments
///
/// * `deserializer` - The deserializer to use
///
/// # Returns
///
/// Returns `Ok(Some(OffsetDateTime))` if the string is valid and present,
/// `Ok(None)` if the input is null/None, or an error if the deserialization fails.
///
/// # Example
///
/// ```
/// use serde_json::json;
/// use time::OffsetDateTime;
///
/// #[derive(Deserialize)]
/// struct Timestamp {
///     #[serde(deserialize_with = "deserialize_offset_date_time")]
///     created_at: Option<OffsetDateTime>
/// }
/// ```
#[allow(unused)]
pub fn deserialize_offset_date_time<'de, D>(
    deserializer: D,
) -> Result<Option<OffsetDateTime>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => Ok(Some(
            OffsetDateTime::parse(&s, &Rfc3339).map_err(serde::de::Error::custom)?,
        )),
        None => Ok(None),
    }
}
