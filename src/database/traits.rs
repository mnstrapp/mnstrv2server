//! Database Resource Traits
//!
//! This module defines the `DatabaseResource` trait which must be implemented by any
//! struct that wants to use the database macros. The trait provides metadata about
//! the resource's database behavior and capabilities.
//!
//! ## Required Implementation
//!
//! All structs that use database macros must implement `DatabaseResource` with
//! appropriate values for each method based on their database schema and requirements.

use sqlx::{Error, postgres::PgRow};

/// Trait that must be implemented by any struct used with database macros.
///
/// This trait provides metadata about how a resource should behave in database operations.
/// It controls automatic field generation, timestamp handling, and deletion behavior.
///
/// # Required Methods
///
/// - `from_row()` - Convert database row to struct
/// - `has_id()` - Whether resource has auto-generated ID
/// - `is_archivable()` - Whether resource supports soft deletion
/// - `is_updatable()` - Whether resource has updated_at timestamps
/// - `is_creatable()` - Whether resource has created_at timestamps
/// - `is_expirable()` - Whether resource has expires_at timestamps
/// - `is_verifiable()` - Whether resource supports verification
///
/// # Example Implementation
///
/// ```rust
/// use crate::database::traits::DatabaseResource;
/// use sqlx::{postgres::PgRow, Error};
///
/// pub struct User {
///     pub id: String,
///     pub email: Option<String>,
///     pub phone: Option<String>,
///     pub name: String,
///     pub created_at: String,
///     pub updated_at: String,
///     pub archived_at: Option<String>,
/// }
///
/// impl DatabaseResource for User {
///     fn from_row(row: &PgRow) -> Result<Self, Error> {
///         Ok(User {
///             id: row.try_get("id")?,
///             email: row.try_get("email")?,
///             phone: row.try_get("phone")?,
///             name: row.try_get("name")?,
///             created_at: row.try_get("created_at")?,
///             updated_at: row.try_get("updated_at")?,
///             archived_at: row.try_get("archived_at")?,
///         })
///     }
///
///     fn has_id() -> bool { true }
///     fn is_archivable() -> bool { true }
///     fn is_updatable() -> bool { true }
///     fn is_creatable() -> bool { true }
///     fn is_expirable() -> bool { false }
///     fn is_verifiable() -> bool { false }
/// }
/// ```
pub trait DatabaseResource {
    /// Converts a database row to the implementing struct.
    ///
    /// This method should extract all fields from the provided `PgRow` and construct
    /// an instance of the implementing struct. Use `row.try_get()` to extract fields
    /// by column name.
    ///
    /// # Arguments
    ///
    /// * `row` - The database row containing the resource data
    ///
    /// # Returns
    ///
    /// `Result<Self, Error>` - The constructed struct or database error
    fn from_row(row: &PgRow) -> Result<Self, Error>
    where
        Self: Sized;

    /// Whether the resource has an auto-generated ID field.
    ///
    /// If this returns `true`, the insert macros will automatically generate a UUID v4
    /// and set it as the `id` field. If `false`, no ID will be generated.
    ///
    /// # Returns
    ///
    /// `bool` - Whether the resource has an auto-generated ID
    fn has_id() -> bool;

    /// Whether the resource supports soft deletion via archiving.
    ///
    /// If this returns `true`, delete operations will set the `archived_at` timestamp
    /// instead of actually deleting the record. If `false`, records will be hard deleted.
    ///
    /// # Returns
    ///
    /// `bool` - Whether the resource supports soft deletion
    fn is_archivable() -> bool;

    /// Whether the resource has `updated_at` timestamps.
    ///
    /// If this returns `true`, update operations will automatically set the `updated_at`
    /// field to the current timestamp. If `false`, no timestamp will be set.
    ///
    /// # Returns
    ///
    /// `bool` - Whether the resource has updated_at timestamps
    fn is_updatable() -> bool;

    /// Whether the resource has `created_at` timestamps.
    ///
    /// If this returns `true`, insert operations will automatically set the `created_at`
    /// field to the current timestamp. If `false`, no timestamp will be set.
    ///
    /// # Returns
    ///
    /// `bool` - Whether the resource has created_at timestamps
    fn is_creatable() -> bool;

    /// Whether the resource has `expires_at` timestamps.
    ///
    /// If this returns `true`, insert and update operations will automatically set the
    /// `expires_at` field to 30 days from the current time. If `false`, no expiration
    /// will be set.
    ///
    /// # Returns
    ///
    /// `bool` - Whether the resource has expires_at timestamps
    fn is_expirable() -> bool;

    /// Whether the resource supports verification.
    ///
    /// This method is currently unused but reserved for future verification features.
    ///
    /// # Returns
    ///
    /// `bool` - Whether the resource supports verification
    #[allow(unused)]
    fn is_verifiable() -> bool;
}
