//! Database Value Types
//!
//! This module provides the `DatabaseValue` enum which represents type-safe database values
//! with proper SQL encoding for PostgreSQL. The enum supports all common database types
//! and provides convenient `From` implementations for easy conversion from Rust types.
//!
//! ## Features
//!
//! - **Type Safety**: Proper SQL type encoding for PostgreSQL
//! - **Null Support**: Handles NULL values appropriately
//! - **Automatic Conversion**: From implementations for common Rust types
//! - **SQLx Integration**: Implements required traits for SQLx compatibility

use sqlx::postgres::PgArgumentBuffer;
use sqlx::{Encode, Postgres, Type, encode::IsNull, error::BoxDynError};
use std::fmt::{self, Display};
use std::iter::FromIterator;
use time::OffsetDateTime;

/// Represents a type-safe database value with proper SQL encoding.
///
/// This enum provides a unified way to handle different database types while maintaining
/// type safety and proper SQL encoding for PostgreSQL. It supports all common database
/// types and provides convenient conversion from Rust types.
///
/// # Variants
///
/// - `None` - Represents a NULL value in the database
/// - `Str(&'static str)` - Static string reference
/// - `String(String)` - Owned string value
/// - `Text(String)` - Text field (same as String but semantically different)
/// - `Int(String)` - Integer value stored as string
/// - `Int64(String)` - 64-bit integer value stored as string
/// - `Float(String)` - Floating point value stored as string
/// - `Boolean(String)` - Boolean value stored as string
/// - `DateTime(String)` - DateTime value stored as ISO8601 string
///
/// # Examples
///
/// ```rust
/// use crate::database::values::DatabaseValue;
///
/// // String values
/// let value: DatabaseValue = "hello".into();
/// let value: DatabaseValue = String::from("world").into();
///
/// // Numeric values
/// let value: DatabaseValue = 42i64.into();
/// let value: DatabaseValue = 3.14f64.into();
///
/// // Boolean values
/// let value: DatabaseValue = true.into();
///
/// // DateTime values
/// let value: DatabaseValue = OffsetDateTime::now_utc().into();
///
/// // Null values
/// let value = DatabaseValue::None;
/// ```
#[derive(Debug, Clone)]
pub enum DatabaseValue {
    /// Represents a NULL value in the database
    #[allow(dead_code)]
    None,
    /// Static string reference
    #[allow(dead_code)]
    Str(&'static str),
    /// Owned string value
    #[allow(dead_code)]
    String(String),
    /// Text field (semantically different from String)
    #[allow(dead_code)]
    Text(String),
    /// Integer value stored as string
    #[allow(dead_code)]
    Int(String),
    #[allow(dead_code)]
    Int32(i32),
    /// 64-bit integer value stored as string
    #[allow(dead_code)]
    Int64(String),
    /// Floating point value stored as string
    #[allow(dead_code)]
    Float(String),
    /// Boolean value stored as string
    #[allow(dead_code)]
    Boolean(String),
    /// DateTime value stored as ISO8601 string
    #[allow(dead_code)]
    DateTime(String),
}

impl Display for DatabaseValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'q> Encode<'q, Postgres> for DatabaseValue {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        match self {
            DatabaseValue::None => Ok(IsNull::Yes),
            DatabaseValue::Str(s) => Encode::<Postgres>::encode_by_ref(s, buf),
            DatabaseValue::String(s) => Encode::<Postgres>::encode_by_ref(s, buf),
            DatabaseValue::Text(s) => Encode::<Postgres>::encode_by_ref(s, buf),
            DatabaseValue::Int(i) => Encode::<Postgres>::encode_by_ref(i, buf),
            DatabaseValue::Int32(i) => Encode::<Postgres>::encode_by_ref(i, buf),
            DatabaseValue::Int64(i) => Encode::<Postgres>::encode_by_ref(i, buf),
            DatabaseValue::Float(f) => Encode::<Postgres>::encode_by_ref(f, buf),
            DatabaseValue::Boolean(b) => Encode::<Postgres>::encode_by_ref(b, buf),
            DatabaseValue::DateTime(dt) => Encode::<Postgres>::encode_by_ref(dt, buf),
        }
    }
}

impl Type<Postgres> for DatabaseValue {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("text")
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        let text_oids = [25, 1043, 1042, 19, 1042];
        ty.oid()
            .map(|oid| text_oids.contains(&oid.0))
            .unwrap_or(false)
    }
}

impl<'a> FromIterator<&'a str> for DatabaseValue {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        DatabaseValue::String(iter.into_iter().collect::<String>())
    }
}

impl FromIterator<String> for DatabaseValue {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        DatabaseValue::String(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<&'a String> for DatabaseValue {
    fn from_iter<I: IntoIterator<Item = &'a String>>(iter: I) -> Self {
        DatabaseValue::String(iter.into_iter().cloned().collect())
    }
}

impl FromIterator<bool> for DatabaseValue {
    fn from_iter<I: IntoIterator<Item = bool>>(iter: I) -> Self {
        DatabaseValue::Boolean(iter.into_iter().map(|b| b.to_string()).collect())
    }
}

impl FromIterator<OffsetDateTime> for DatabaseValue {
    fn from_iter<I: IntoIterator<Item = OffsetDateTime>>(iter: I) -> Self {
        DatabaseValue::DateTime(iter.into_iter().map(|dt| dt.to_string()).collect())
    }
}

impl FromIterator<i32> for DatabaseValue {
    fn from_iter<I: IntoIterator<Item = i32>>(iter: I) -> Self {
        DatabaseValue::Int(iter.into_iter().map(|i| i.to_string()).collect())
    }
}

impl FromIterator<i64> for DatabaseValue {
    fn from_iter<I: IntoIterator<Item = i64>>(iter: I) -> Self {
        DatabaseValue::Int64(iter.into_iter().map(|i| i.to_string()).collect())
    }
}

impl FromIterator<f64> for DatabaseValue {
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> Self {
        DatabaseValue::Float(iter.into_iter().map(|f| f.to_string()).collect())
    }
}

impl From<Option<String>> for DatabaseValue {
    fn from(s: Option<String>) -> Self {
        DatabaseValue::String(s.unwrap_or_default())
    }
}

impl From<&str> for DatabaseValue {
    fn from(s: &str) -> Self {
        DatabaseValue::String(s.to_string())
    }
}

impl From<String> for DatabaseValue {
    fn from(s: String) -> Self {
        DatabaseValue::String(s)
    }
}

impl From<&'_ String> for DatabaseValue {
    fn from(s: &'_ String) -> Self {
        DatabaseValue::String(s.clone())
    }
}

impl From<bool> for DatabaseValue {
    fn from(b: bool) -> Self {
        DatabaseValue::Boolean(b.to_string())
    }
}

impl From<OffsetDateTime> for DatabaseValue {
    fn from(dt: OffsetDateTime) -> Self {
        DatabaseValue::DateTime(dt.to_string())
    }
}

impl From<i32> for DatabaseValue {
    fn from(i: i32) -> Self {
        DatabaseValue::Int(i.to_string())
    }
}

impl From<i64> for DatabaseValue {
    fn from(i: i64) -> Self {
        DatabaseValue::Int64(i.to_string())
    }
}

impl From<f64> for DatabaseValue {
    fn from(f: f64) -> Self {
        DatabaseValue::Float(f.to_string())
    }
}
