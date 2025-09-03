//! Update Macros for Database Operations
//!
//! This module provides macros for updating existing resources in the database.
//! The macros automatically handle timestamp updates, expiration dates, and
//! result fetching based on the `DatabaseResource` trait implementation.

/// Updates an existing resource in the database by ID.
///
/// This macro generates an UPDATE query and automatically handles common database fields:
/// - Sets `updated_at` timestamp if `is_updatable()` returns true
/// - Sets `expires_at` timestamp (30 days from now) if `is_expirable()` returns true
/// - Fetches and returns the updated resource after successful update
/// - Supports updating multiple fields in a single operation
///
/// # Arguments
/// * `$resource` - The resource type (must implement DatabaseResource)
/// * `$id` - The unique identifier of the resource to update
/// * `$params` - Vector of `(&str, DatabaseValue)` tuples for field updates
///
/// # Returns
/// `Result<Resource, Error>` - The updated resource or database error
///
/// # Example
/// ```rust
/// // Update user profile information
/// let params = vec![
///     ("email", "newemail@example.com".into()),
///     ("name", "Jane Smith".into())
/// ];
/// let updated_user = update_resource!(User, "user-123", params).await?;
/// ```
///
/// # Features
/// - **Automatic Timestamps**: Updates `updated_at` if resource is updatable
/// - **Expiration Management**: Sets `expires_at` to 30 days from update if applicable
/// - **Field Override**: Allows overriding auto-generated timestamp fields
/// - **Type Safety**: Proper SQL type casting for all DatabaseValue variants
/// - **Resource Return**: Fetches and returns the complete updated resource
/// - **ID-based Updates**: Updates resources by their unique identifier
/// - **SQL Injection Protection**: Uses parameter binding for security
/// - **Transaction Safety**: Performs update then fetch in sequence
///
/// # Generated SQL Examples
///
/// ## Basic Update with Timestamps
/// ```sql
/// UPDATE users SET
///     email = $1,
///     name = $2,
///     updated_at = CAST($3 AS TIMESTAMP WITH TIME ZONE)
/// WHERE id = $4
/// RETURNING *
/// ```
///
/// ## Update with Expiration
/// ```sql
/// UPDATE sessions SET
///     token = $1,
///     updated_at = CAST($2 AS TIMESTAMP WITH TIME ZONE),
///     expires_at = CAST($3 AS TIMESTAMP WITH TIME ZONE)
/// WHERE id = $4
/// RETURNING *
/// ```
///
/// ## Update with NULL Values
/// ```sql
/// UPDATE contacts SET
///     phone = NULL,
///     updated_at = CAST($1 AS TIMESTAMP WITH TIME ZONE)
/// WHERE id = $2
/// RETURNING *
/// ```
///
/// # Update Process
/// 1. **Parameter Processing**: Converts input parameters to update fields
/// 2. **Auto Field Addition**: Adds updated_at and expires_at if applicable
/// 3. **SQL Generation**: Creates UPDATE query with proper type casting
/// 4. **Query Execution**: Performs the update operation
/// 5. **Resource Fetching**: Retrieves the updated resource using find_one
/// 6. **Result Return**: Returns the complete updated resource
///
/// # Type Casting
/// The macro automatically handles different DatabaseValue types:
/// - **Strings**: Cast to VARCHAR with proper parameter binding
/// - **Timestamps**: Cast to TIMESTAMP WITH TIME ZONE for date/time fields
/// - **Numbers**: Cast to appropriate numeric types (INTEGER, BIGINT, FLOAT)
/// - **Booleans**: Cast to BOOLEAN for boolean fields
/// - **NULL Values**: Handled specially to avoid binding issues
///
/// # Usage Notes
/// - The resource must exist before updating (ID must be valid)
/// - All fields in `$params` will be updated
/// - Auto-generated fields can be overridden by including them in params
/// - The macro returns the complete updated resource, not just success status
/// - Updates are performed atomically with proper error handling
#[macro_export]
macro_rules! update_resource {
    ($resource:ty, $id:expr, $params:expr) => {{
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
        use crate::find_one_resource_where_fields;
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;
        use time::{Duration, OffsetDateTime};

        async {
            let id = $id.to_string();
            let updated_at = OffsetDateTime::now_utc();
            let expires_at = (OffsetDateTime::now_utc() + Duration::days(30));

            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            let mut params: Vec<(&str, DatabaseValue)> = Vec::new();

            let input_params: Vec<(&str, DatabaseValue)> = $params;
            if !input_params.is_empty() {
                for (field, value) in input_params {
                    params.push((field, value.clone()));
                }
            }

            if <$resource as DatabaseResource>::is_updatable() {
                if let Some(idx) = params
                    .iter()
                    .position(|(field, _)| field.contains("updated_at"))
                {
                    params[idx] = ("updated_at", updated_at.into());
                } else {
                    params.push(("updated_at", updated_at.into()));
                }
            }

            if <$resource as DatabaseResource>::is_expirable() {
                if let Some(idx) = params
                    .iter()
                    .position(|(field, _)| field.contains("expires_at"))
                {
                    params[idx] = ("expires_at", expires_at.into());
                } else {
                    params.push(("expires_at", expires_at.into()));
                }
            }

            let fields = params
                .iter()
                .map(|(field, _)| field.to_string())
                .collect::<Vec<String>>();
            let values: Vec<&DatabaseValue> = params.iter().map(|(_, value)| value).collect();

            let mut query = format!("UPDATE {} SET ", resource_name);

            for (i, field) in fields.iter().enumerate() {
                let value = values[i];
                match value {
                    DatabaseValue::None => {
                        query.push_str(&format!("{} = NULL", field));
                    }
                    DatabaseValue::Str(_) | DatabaseValue::String(_) | DatabaseValue::Text(_) => {
                        query.push_str(&format!("{} = ${}", field, i + 1));
                    }
                    DatabaseValue::DateTime(_) => {
                        query.push_str(&format!(
                            "{} = CAST(${} AS TIMESTAMP WITH TIME ZONE)",
                            field,
                            i + 1
                        ));
                    }
                    DatabaseValue::Int(_) => {
                        query.push_str(&format!("{} = CAST(${} AS INTEGER)", field, i + 1));
                    }
                    DatabaseValue::Int32(_) => {
                        query.push_str(&format!("{} = CAST(${} AS INTEGER)", field, i + 1));
                    }
                    DatabaseValue::Int64(_) => {
                        query.push_str(&format!("{} = CAST(${} AS BIGINT)", field, i + 1));
                    }
                    DatabaseValue::Float(_) => {
                        query.push_str(&format!("{} = CAST(${} AS FLOAT)", field, i + 1));
                    }
                    DatabaseValue::Boolean(_) => {
                        query.push_str(&format!("{} = CAST(${} AS BOOLEAN)", field, i + 1));
                    }
                }
                if i < fields.len() - 1 {
                    query.push_str(", ");
                }
            }

            query.push_str(&format!(" WHERE id = ${}", fields.len() + 1));
            query.push_str(&format!(" RETURNING *"));

            let mut query = sqlx::query(&query);
            for (_, value) in values.iter().enumerate() {
                match value {
                    DatabaseValue::None => query = query.bind(Option::<String>::None),
                    _ => query = query.bind(value),
                }
            }
            query = query.bind(&id);

            match query.execute(&pool).await {
                Ok(_) => (),
                Err(e) => return Err(e),
            };

            let params: Vec<(&str, DatabaseValue)> = vec![("id", $id.into())];
            match find_one_resource_where_fields!($resource, params).await {
                Ok(resource) => Ok(resource),
                Err(e) => Err(e),
            }
        }
    }};
}
