//! Join Macros for Database Operations
//!
//! This module provides macros for performing JOIN operations between resources in the database.
//! The macros automatically handle table naming, join conditions, and result mapping
//! based on the `DatabaseResource` trait implementation.

/// Performs a JOIN operation between two resources based on their ID fields.
///
/// This macro creates a SQL JOIN query between two tables and automatically:
/// - Converts resource names to snake_case and pluralizes them for table names
/// - Creates join conditions using `{resource}_id` naming convention
/// - Applies WHERE conditions to filter results
/// - Maps results back to the primary resource type
///
/// # Arguments
/// * `$resource` - The primary resource type (must implement DatabaseResource)
/// * `$join_resource` - The resource to join with (must implement DatabaseResource)
/// * `$params` - Vector of `(&str, DatabaseValue)` tuples for WHERE conditions
///
/// # Returns
/// `Result<Vec<Resource>, Error>` - Vector of joined resources or database error
///
/// # Example
/// ```rust
/// // Join users with their sessions
/// let params = vec![("status", "active".into())];
/// let active_users_with_sessions = join_all_resources_where_fields_on!(
///     User, Session, params
/// ).await?;
/// ```
///
/// # Features
/// - **Automatic Table Naming**: Converts resource names to proper table names
/// - **Smart Join Conditions**: Uses `{resource}_id` convention for joins
/// - **Conditional Filtering**: Supports WHERE clauses for result filtering
/// - **Type Safety**: Works with any struct implementing DatabaseResource
/// - **Result Mapping**: Automatically maps database rows to resource structs
/// - **SQL Injection Protection**: Uses parameter binding for security
///
/// # Generated SQL Examples
///
/// ## Basic JOIN with WHERE Conditions
/// ```sql
/// SELECT * FROM users
/// JOIN sessions ON session_id = user_id
/// WHERE status = $1
/// ```
///
/// ## Complex JOIN with Multiple Conditions
/// ```sql
/// SELECT * FROM organizations
/// JOIN organization_locations ON organization_location_id = organization_id
/// WHERE plan_type = $1 AND is_active = $2
/// ```
///
/// # Join Logic
/// The macro automatically determines join fields based on resource names:
/// - Primary resource: `users` table with `user_id` join field
/// - Join resource: `sessions` table with `session_id` join field
/// - Join condition: `sessions.session_id = users.user_id`
///
/// # Usage Notes
/// - The join is always performed on the primary resource's ID field
/// - WHERE conditions are applied to the joined result set
/// - Results are returned as the primary resource type
/// - The macro assumes a standard naming convention for foreign keys
#[macro_export]
macro_rules! join_all_resources_where_fields_on {
    ($resource:ty, $join_resource:ty, $params:expr) => {{
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            let resource_name = camel_to_snake_case(stringify!($resource).to_string());
            let resource_table_name = pluralize(&resource_name, 2, false);
            let resource_join_name = format!("{}_id", resource_name);

            let join_resource_name = camel_to_snake_case(stringify!($join_resource).to_string());
            let join_resource_table_name = pluralize(&join_resource_name, 2, false);
            let join_resource_join_name = format!("{}_id", join_resource_name);

            let pool = get_connection().await;

            let params: Vec<(&str, DatabaseValue)> = $params.clone();
            let fields = params
                .iter()
                .map(|field| field.0.to_string())
                .collect::<Vec<String>>();
            let values = params
                .iter()
                .map(|field| field.1.to_string())
                .collect::<Vec<String>>();

            let mut query = format!(
                "SELECT * FROM {} JOIN {} ON {} = {}",
                resource_table_name,
                join_resource_table_name,
                join_resource_join_name,
                resource_join_name
            );

            query.push_str(" WHERE ");
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

            match query.fetch_all(&pool).await {
                Ok(rows) => Ok(rows
                    .iter()
                    .map(|row| <$resource as DatabaseResource>::from_row(row).unwrap())
                    .collect::<Vec<$resource>>()),
                Err(e) => Err(e),
            }
        }
    }};
}
