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
///     ("phone", "1234567890".into()),
///     ("name", "John Doe".into()),
///     ("password_hash", "hashed_password".into())
/// ];
/// let new_user = insert_resource!(User, params).await?;
/// ```
#[macro_export]
macro_rules! insert_resource {
    ($resource:ty, $params:expr) => {{
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;
        use time::{Duration, OffsetDateTime};
        use uuid::Uuid;

        async {
            let input_params: Vec<(&str, DatabaseValue)> = $params;
            if input_params.is_empty() {
                return Err(anyhow::Error::msg("No params provided"));
            }

            let id = Uuid::new_v4().to_string();
            let created_at = OffsetDateTime::now_utc();
            let updated_at = created_at.clone();
            let expires_at = (OffsetDateTime::now_utc() + Duration::days(30));

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
                if let Some(idx) = params.iter().position(|(field, _)| field == "id") {
                    params[idx] = ("id".to_string(), id.clone().into());
                } else {
                    params.push(("id".to_string(), id.clone().into()));
                }
            }

            if <$resource as DatabaseResource>::is_creatable() {
                if let Some(idx) = params
                    .iter()
                    .position(|(field, _)| field.contains("created_at"))
                {
                    params[idx] = (
                        "created_at".to_string(),
                        DatabaseValue::DateTime(created_at.clone().to_string()),
                    );
                } else {
                    params.push((
                        "created_at".to_string(),
                        DatabaseValue::DateTime(created_at.clone().to_string()),
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
                        DatabaseValue::DateTime(updated_at.clone().to_string()),
                    );
                } else {
                    params.push((
                        "updated_at".to_string(),
                        DatabaseValue::DateTime(updated_at.clone().to_string()),
                    ));
                }
            }

            if <$resource as DatabaseResource>::is_expirable() {
                if let Some(idx) = params
                    .iter()
                    .position(|(field, _)| field.contains("expires_at"))
                {
                    params[idx] = (
                        "expires_at".to_string(),
                        DatabaseValue::DateTime(expires_at.clone().to_string()),
                    );
                } else {
                    params.push((
                        "expires_at".to_string(),
                        DatabaseValue::DateTime(expires_at.clone().to_string()),
                    ));
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
                        query.push_str(&format!("CAST(${} AS TIMESTAMP WITHOUT TIME ZONE)", i + 1));
                    }
                    DatabaseValue::Int(_) | DatabaseValue::Int32(_) => {
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
                    Err(anyhow::Error::msg(e.to_string()))
                }
            }
        }
    }};
}

#[macro_export]
macro_rules! insert_resource_batch {
    ($resource:ty, $resources:expr) => {{
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;
        use time::{Duration, OffsetDateTime};
        use uuid::Uuid;

        async {
            let pool = get_connection().await;
            let resources: Vec<Vec<(&str, DatabaseValue)>> = $resources.clone();
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );

            let created_at = OffsetDateTime::now_utc();
            let updated_at = created_at.clone();
            let expires_at = (OffsetDateTime::now_utc() + Duration::days(30));

            if resources.is_empty() {
                return Ok(Vec::<$resource>::new());
            }

            let mut fields: Vec<&str> = resources[0]
                .clone()
                .iter()
                .map(|(field, _)| *field)
                .collect::<Vec<&str>>();

            if <$resource as DatabaseResource>::has_id() {
                if let Some(idx) = fields.iter().position(|field| field == &"id") {
                    fields[idx] = "id";
                } else {
                    fields.push("id");
                }
            }

            if <$resource as DatabaseResource>::is_creatable() {
                if let Some(idx) = fields.iter().position(|field| field == &"created_at") {
                    fields[idx] = "created_at";
                } else {
                    fields.push("created_at");
                }
            }

            if <$resource as DatabaseResource>::is_updatable() {
                if let Some(idx) = fields.iter().position(|field| field == &"updated_at") {
                    fields[idx] = "updated_at";
                } else {
                    fields.push("updated_at");
                }
            }

            if <$resource as DatabaseResource>::is_expirable() {
                if let Some(idx) = fields.iter().position(|field| field == &"expires_at") {
                    fields[idx] = "expires_at";
                } else {
                    fields.push("expires_at");
                }
            }

            let mut query = format!("INSERT INTO {} (", resource_name);

            for (i, field) in fields.iter().enumerate() {
                query.push_str(field);
                if i < fields.len() - 1 {
                    query.push_str(", ");
                }
            }

            query.push_str(") VALUES ");

            let mut values: Vec<DatabaseValue> = Vec::new();

            for (i, resource) in resources.iter().enumerate() {
                let mut input_params: Vec<(&str, DatabaseValue)> = resource.clone();
                if input_params.is_empty() {
                    return Err(anyhow::Error::msg("Params are empty"));
                }

                let id = Uuid::new_v4().to_string();

                if <$resource as DatabaseResource>::has_id() {
                    if let Some(idx) = input_params.iter().position(|(field, _)| field == &"id") {
                        input_params[idx] = ("id", id.clone().into());
                    } else {
                        input_params.push(("id", id.clone().into()));
                    }
                }

                if <$resource as DatabaseResource>::is_creatable() {
                    if let Some(idx) = input_params
                        .iter()
                        .position(|(field, _)| field == &"created_at")
                    {
                        input_params[idx] = (
                            "created_at",
                            DatabaseValue::DateTime(created_at.clone().to_string()),
                        );
                    } else {
                        input_params.push((
                            "created_at",
                            DatabaseValue::DateTime(created_at.clone().to_string()),
                        ));
                    }
                }

                if <$resource as DatabaseResource>::is_updatable() {
                    if let Some(idx) = input_params
                        .iter()
                        .position(|(field, _)| field == &"updated_at")
                    {
                        input_params[idx] = (
                            "updated_at",
                            DatabaseValue::DateTime(updated_at.clone().to_string()),
                        );
                    } else {
                        input_params.push((
                            "updated_at",
                            DatabaseValue::DateTime(updated_at.clone().to_string()),
                        ));
                    }
                }

                if <$resource as DatabaseResource>::is_expirable() {
                    if let Some(idx) = input_params
                        .iter()
                        .position(|(field, _)| field == &"expires_at")
                    {
                        input_params[idx] = (
                            "expires_at",
                            DatabaseValue::DateTime(expires_at.clone().to_string()),
                        );
                    } else {
                        input_params.push((
                            "expires_at",
                            DatabaseValue::DateTime(expires_at.clone().to_string()),
                        ));
                    }
                }

                let mut idxs: Vec<usize> = Vec::new();
                for (_, value) in input_params.clone() {
                    values.push(value.clone());
                    idxs.push(values.len());
                }

                let mut value_query = String::from("(");

                for (j, _) in fields.iter().enumerate() {
                    let idx = idxs[j];
                    let value = input_params[j].1.clone();
                    match value {
                        DatabaseValue::None => {
                            value_query.push_str("NULL");
                        }
                        DatabaseValue::Str(_)
                        | DatabaseValue::String(_)
                        | DatabaseValue::Text(_) => {
                            value_query.push_str(&format!("${}", idx));
                        }
                        DatabaseValue::DateTime(_) => {
                            value_query.push_str(&format!(
                                "CAST(${} AS TIMESTAMP WITHOUT TIME ZONE)",
                                idx
                            ));
                        }
                        DatabaseValue::Int(_) | DatabaseValue::Int32(_) => {
                            value_query.push_str(&format!("CAST(${} AS INTEGER)", idx));
                        }
                        DatabaseValue::Int64(_) => {
                            value_query.push_str(&format!("CAST(${} AS BIGINT)", idx));
                        }
                        DatabaseValue::Float(_) => {
                            value_query.push_str(&format!("CAST(${} AS FLOAT)", idx));
                        }
                        DatabaseValue::Boolean(_) => {
                            value_query.push_str(&format!("CAST(${} AS BOOLEAN)", idx));
                        }
                    }
                    if j < fields.len() - 1 {
                        value_query.push_str(", ");
                    }
                }
                value_query.push_str(")");
                if i < resources.len() - 1 {
                    value_query.push_str(", ");
                }
                query.push_str(&value_query);
            }

            query.push_str(" RETURNING *");

            let mut query = sqlx::query(&query);
            for (_, value) in values.iter().enumerate() {
                query = query.bind(value);
            }

            match query.fetch_all(&pool).await {
                Ok(rows) => Ok(rows
                    .into_iter()
                    .map(|row| <$resource as DatabaseResource>::from_row(&row))
                    .collect::<Result<Vec<$resource>, _>>()?),
                Err(e) => Err(anyhow::Error::msg(e.to_string())),
            }
        }
    }};
}
