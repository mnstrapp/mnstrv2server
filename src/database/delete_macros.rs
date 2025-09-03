//! Delete Macros for Database Operations
//!
//! This module provides macros for deleting resources from the database.
//! The macros support both soft deletion (archiving) and hard deletion based on
//! the `DatabaseResource` trait implementation.

/// Deletes resources matching the specified field conditions.
///
/// This macro performs either soft deletion or hard deletion based on the `is_archivable()`
/// trait method:
/// - If `is_archivable()` returns true: Sets `archived_at` timestamp (soft delete)
/// - If `is_archivable()` returns false: Performs actual DELETE query (hard delete)
///
/// # Arguments
/// * `$resource` - The resource type (must implement DatabaseResource)
/// * `$params` - Vector of `(&str, DatabaseValue)` tuples for field conditions
///
/// # Returns
/// `Result<(), Error>` - Success or database error
///
/// # Example
/// ```rust
/// // Soft delete (if resource is archivable)
/// let params = vec![("organization_id", "456".into())];
/// delete_resource_where_fields!(User, params).await?;
///
/// // Hard delete (if resource is not archivable)
/// let params = vec![("id", "789".into())];
/// delete_resource_where_fields!(TemporaryToken, params).await?;
/// ```
///
/// # Features
/// - **Soft Delete Support**: Archives resources by setting archived_at timestamp
/// - **Hard Delete Support**: Performs actual DELETE queries for non-archivable resources
/// - **Conditional Deletion**: Supports multiple field conditions for precise targeting
/// - **Type Safety**: Works with any struct implementing DatabaseResource
/// - **SQL Injection Protection**: Uses parameter binding for security
///
/// # Generated SQL Examples
///
/// ## Soft Delete (Archivable Resource)
/// ```sql
/// UPDATE users SET archived_at = CAST($3 AS TIMESTAMP WITH TIME ZONE)
/// WHERE organization_id = $1 AND status = $2
/// ```
///
/// ## Hard Delete (Non-Archivable Resource)
/// ```sql
/// DELETE FROM temporary_tokens
/// WHERE id = $1 AND expires_at < $2
/// ```
#[macro_export]
macro_rules! delete_resource_where_fields {
    ($resource:ty, $params:expr) => {{
        use crate::database::connection::get_connection;
        use crate::database::traits::DatabaseResource;
        use crate::database::values::DatabaseValue;
        use crate::utils::strings::camel_to_snake_case;
        use anyhow::anyhow;
        use pluralizer::pluralize;
        use time::OffsetDateTime;

        async {
            let archived_at = OffsetDateTime::now_utc();

            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            let params: Vec<(&str, DatabaseValue)> = $params.clone();

            let fields: Vec<String> = params.iter().map(|field| field.0.to_string()).collect();
            let values: Vec<DatabaseValue> = params.iter().map(|field| field.1.clone()).collect();

            let mut query: String;
            if <$resource as DatabaseResource>::is_archivable() {
                query = format!(
                    "UPDATE {} SET archived_at = CAST(${} AS TIMESTAMP WITH TIME ZONE) WHERE ",
                    resource_name,
                    fields.len() + 1
                );
            } else {
                query = format!("DELETE FROM {} WHERE ", resource_name);
            }

            for (i, field) in fields.iter().enumerate() {
                query.push_str(&format!("{} = ${}", field, i + 1));
                if i < fields.len() - 1 {
                    query.push_str(" AND ");
                }
            }

            let mut query = sqlx::query(&query);
            for (_, value) in values.iter().enumerate() {
                query = query.bind(value);
            }
            if <$resource as DatabaseResource>::is_archivable() {
                query = query.bind(archived_at);
            }

            match query.execute(&pool).await {
                Ok(_) => (),
                Err(e) => return Err(anyhow!(e)),
            };

            Ok(())
        }
    }};
}
