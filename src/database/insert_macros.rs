//! Insert Macros for Database Operations
//!
//! This module provides macros for creating new resources in the database.
//! The macros automatically handle common fields like IDs, timestamps, and expiration dates
//! based on the `DatabaseResource` trait implementation.

/// Creates a new resource in the database.
///
/// This macro generates an INSERT query and automatically handles common database fields:
/// - Generates UUID if `has_id()` returns true
/// - Sets `created_at` timestamp if `is_creatable()` returns true
/// - Sets `updated_at` timestamp if `is_updatable()` returns true
/// - Sets `expires_at` timestamp (30 days from now) if `is_expirable()` returns true
///
/// # Arguments
/// * `$resource` - The resource type (must implement DatabaseResource)
/// * `$params` - Vector of `(&str, DatabaseValue)` tuples for field values
///
/// # Returns
/// `Result<Resource, Error>` - The created resource or database error
///
/// # Example
/// ```rust
/// // Basic insertion
/// let params = vec![
///     ("email", "newuser@example.com".into()),
///     ("name", "John Doe".into()),
///     ("password_hash", "hashed_password".into())
/// ];
/// let new_user = insert_resource!(User, params).await?;
/// ```
///
/// # Features
/// - **Auto ID Generation**: Creates UUID v4 if `has_id()` returns true
/// - **Timestamp Management**: Automatically sets created_at, updated_at timestamps
/// - **Expiration Handling**: Sets expires_at to 30 days from creation if applicable
/// - **Type Safety**: Proper SQL type casting for all DatabaseValue variants
/// - **Resource Return**: Returns the complete created resource
/// - **Field Override**: Allows overriding auto-generated fields
/// - **SQL Injection Protection**: Uses parameter binding for security
///
/// # Generated SQL Examples
///
/// ## Basic Insert with Auto Fields
/// ```sql
/// INSERT INTO users (
///     email, name, password_hash, id, created_at, updated_at
/// ) VALUES (
///     Cast($1 AS VARCHAR), Cast($2 AS VARCHAR), Cast($3 AS VARCHAR),
///     Cast($4 AS VARCHAR), CAST($5 AS TIMESTAMP), CAST($6 AS TIMESTAMP)
/// ) RETURNING *
/// ```
///
/// ## Insert with Expiration
/// ```sql
/// INSERT INTO sessions (
///     user_id, token, expires_at, id, created_at, updated_at
/// ) VALUES (
///     Cast($1 AS VARCHAR), Cast($2 AS VARCHAR), CAST($3 AS TIMESTAMP),
///     Cast($4 AS VARCHAR), CAST($5 AS TIMESTAMP), CAST($6 AS TIMESTAMP)
/// ) RETURNING *
/// ```
#[macro_export]
macro_rules! insert_resource {
    ($resource:ty, $params:expr) => {{
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;
        use time::{Duration, OffsetDateTime, format_description::well_known::Iso8601};
        use uuid::Uuid;

        let input_params: Vec<(&str, DatabaseValue)> = $params;
        async {
            let id = Uuid::new_v4().to_string();
            let created_at = OffsetDateTime::now_utc()
                .format(&Iso8601::DEFAULT)
                .unwrap()
                .to_string();
            let updated_at = created_at.clone();
            let expires_at = (OffsetDateTime::now_utc() + Duration::days(30))
                .format(&Iso8601::DEFAULT)
                .unwrap()
                .to_string();

            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            let mut params: Vec<(String, DatabaseValue)> = Vec::new();
            for (field, value) in input_params.into_iter() {
                params.push((field.to_string(), value.clone()))
            }

            if <$resource as DatabaseResource>::has_id() {
                params.push(("id".to_string(), DatabaseValue::String(id.clone())));
            }

            if <$resource as DatabaseResource>::is_creatable() {
                if let Some(idx) = params
                    .iter()
                    .position(|(field, _)| field.contains("created_at"))
                {
                    params[idx] = (
                        "created_at".to_string(),
                        DatabaseValue::DateTime(created_at.clone()),
                    );
                } else {
                    params.push((
                        "created_at".to_string(),
                        DatabaseValue::DateTime(created_at.clone()),
                    ));
                }
            }

            if <$resource as DatabaseResource>::is_updatable() {
                if let Some(idx) = params
                    .iter()
                    .position(|(field, _)| field.contains("updated_at"))
                {
                    params[idx] = (
                        "updated_at".to_string(),
                        DatabaseValue::DateTime(updated_at.clone()),
                    );
                } else {
                    params.push((
                        "updated_at".to_string(),
                        DatabaseValue::DateTime(updated_at.clone()),
                    ));
                }
            }

            if <$resource as DatabaseResource>::is_expirable() {
                if let Some(idx) = params
                    .iter()
                    .position(|(field, _)| field.contains("expires_at"))
                {
                    params[idx] = ("expires_at".to_string(), expires_at.into());
                } else {
                    params.push(("expires_at".to_string(), expires_at.into()));
                }
            }

            let fields: Vec<String> = params.iter().map(|(field, _)| field.clone()).collect();
            let values: Vec<DatabaseValue> =
                params.iter().map(|(_, value)| (*value).clone()).collect();

            let mut query = format!("INSERT INTO {} (", resource_name);

            for (i, field) in fields.iter().enumerate() {
                query.push_str(field);
                if i < fields.len() - 1 {
                    query.push_str(", ");
                }
            }

            query.push_str(") VALUES (");
            for (i, value) in values.iter().enumerate() {
                match value {
                    DatabaseValue::None => {
                        query.push_str("NULL");
                    }
                    DatabaseValue::Str(_) | DatabaseValue::String(_) => {
                        query.push_str(&format!("Cast(${} AS VARCHAR)", i + 1));
                    }
                    DatabaseValue::Text(_) => {
                        query.push_str(&format!("Cast(${} AS TEXT)", i + 1));
                    }
                    DatabaseValue::DateTime(_) => {
                        query.push_str(&format!("CAST(${} AS VARCHAR)", i + 1));
                    }
                    DatabaseValue::Int(_) => {
                        query.push_str(&format!("CAST(${} AS INTEGER)", i + 1));
                    }
                    DatabaseValue::Int32(_) => {
                        query.push_str(&format!("CAST(${} AS INTEGER)", i + 1));
                    }
                    DatabaseValue::Int64(_) => {
                        query.push_str(&format!("CAST(${} AS BIGINT)", i + 1));
                    }
                    DatabaseValue::Float(_) => {
                        query.push_str(&format!("CAST(${} AS FLOAT)", i + 1));
                    }
                    DatabaseValue::Boolean(_) => {
                        query.push_str(&format!("CAST(${} AS BOOLEAN)", i + 1));
                    }
                }
                if i < values.len() - 1 {
                    query.push_str(", ");
                }
            }
            query.push_str(") RETURNING *");

            let mut query = sqlx::query(&query);
            for (_, value) in values.iter().enumerate() {
                query = query.bind(value);
            }

            match query.fetch_one(&pool).await {
                Ok(row) => Ok(<$resource as DatabaseResource>::from_row(&row)?),
                Err(e) => {
                    println!("Error fetching row: {:?}", e);
                    Err(e)
                }
            }
        }
    }};
}
