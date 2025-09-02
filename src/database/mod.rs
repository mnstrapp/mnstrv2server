//! Database Macros System
//!
//! This module provides a comprehensive set of Rust macros for database operations using SQLx with PostgreSQL.
//! The macros are designed to work with any struct that implements the `DatabaseResource` trait.
//!
//! ## Overview
//!
//! The database macros system provides type-safe, ergonomic database operations with automatic:
//! - Table name generation (camelCase to snake_case + pluralization)
//! - SQL type casting and parameter binding
//! - Timestamp management (created_at, updated_at, expires_at)
//! - Soft deletion via archiving
//! - UUID generation for IDs
//!
//! ## Module Structure
//!
//! - `connection.rs` - Database connection management
//! - `traits.rs` - DatabaseResource trait definition
//! - `values.rs` - DatabaseValue enum for type-safe database values
//! - `query_macros.rs` - Macros for finding and retrieving resources
//! - `insert_macros.rs` - Macros for creating new resources
//! - `update_macros.rs` - Macros for updating existing resources
//! - `delete_macros.rs` - Macros for deleting resources (soft/hard delete)
//! - `join_macros.rs` - Macros for complex queries with table joins
//!
//! ## Quick Start
//!
//! ```rust
//! use crate::database::traits::DatabaseResource;
//! use sqlx::{postgres::PgRow, Error};
//!
//! // Define your struct
//! pub struct User {
//!     pub id: String,
//!     pub email: String,
//!     pub name: String,
//!     pub created_at: String,
//!     pub updated_at: String,
//!     pub archived_at: Option<String>,
//! }
//!
//! // Implement DatabaseResource
//! impl DatabaseResource for User {
//!     fn from_row(row: &PgRow) -> Result<Self, Error> {
//!         Ok(User {
//!             id: row.try_get("id")?,
//!             email: row.try_get("email")?,
//!             name: row.try_get("name")?,
//!             created_at: row.try_get("created_at")?,
//!             updated_at: row.try_get("updated_at")?,
//!             archived_at: row.try_get("archived_at")?,
//!         })
//!     }
//!
//!     fn has_id() -> bool { true }
//!     fn is_archivable() -> bool { true }
//!     fn is_updatable() -> bool { true }
//!     fn is_creatable() -> bool { true }
//!     fn is_expirable() -> bool { false }
//!     fn is_verifiable() -> bool { false }
//! }
//!
//! // Use the macros
//! async fn example() -> Result<(), Error> {
//!     // Create a user
//!     let params = vec![
//!         ("email", "user@example.com".into()),
//!         ("name", "John Doe".into())
//!     ];
//!     let user = insert_resource!(User, params).await?;
//!
//!     // Find the user
//!     let params = vec![("id", user.id.clone().into())];
//!     let found_user = find_one_resource_where_fields!(User, params).await?;
//!
//!     // Update the user
//!     let update_params = vec![("name", "Jane Doe".into())];
//!     let updated_user = update_resource!(User, user.id, update_params).await?;
//!
//!     // Delete the user (soft delete)
//!     let delete_params = vec![("id", user.id.into())];
//!     delete_resource_where_fields!(User, delete_params).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **Type Safety**: All macros work with strongly-typed structs
//! - **Automatic Field Management**: Handles IDs, timestamps, and expiration dates
//! - **Soft Deletion**: Optional archiving instead of hard deletion
//! - **Flexible Queries**: Support for complex WHERE conditions and JOINs
//! - **SQLx Integration**: Built on top of SQLx for PostgreSQL
//! - **Error Handling**: Proper error propagation and handling
//!
//! For detailed documentation on each macro, see the individual module files.

pub mod connection;
pub mod delete_macros;
pub mod insert_macros;
pub mod join_macros;
pub mod query_macros;
pub mod traits;
pub mod update_macros;
pub mod values;
