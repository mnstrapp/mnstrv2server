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
#[macro_export]
macro_rules! update_resource {
    ($resource:ty, $id:expr, $params:expr) => {{
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
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

            let mut query = sqlx::query(sqlx::AssertSqlSafe(query));
            for (_, value) in values.iter().enumerate() {
                match value {
                    DatabaseValue::None => query = query.bind(Option::<String>::None),
                    _ => query = query.bind(value),
                }
            }
            query = query.bind(&id);

            match query.fetch_one(&pool).await {
                Ok(row) => Ok(<$resource as DatabaseResource>::from_row(&row)?),
                Err(e) => Err(anyhow::Error::msg(e.to_string())),
            }
        }
    }};
}

/// Updates a batch of resources in the database by ID.
///
/// This macro generates an UPDATE query and automatically handles common database fields:
/// - Sets `updated_at` timestamp if `is_updatable()` returns true
/// - Sets `expires_at` timestamp (30 days from now) if `is_expirable()` returns true
/// - Fetches and returns the updated resources after successful update
/// - Supports updating multiple fields in a single operation
/// - Supports updating multiple resources in a single operation
///
/// # Arguments
/// * `$resource` - The resource type (must implement DatabaseResource)
/// * `$resources` - Vector of `Vec<(&str, DatabaseValue)>` tuples for resource updates
///
/// # Returns
/// `Result<Vec<Resource>, Error>` - Vector of updated resources or database error
///
/// # Example
/// ```rust
/// let resources = vec![
///     vec![("id", "123".into()), ("name", "John Doe".into())],
///     vec![("id", "456".into()), ("name", "Jane Smith".into())],
/// ];
/// let updated_resources = update_resource_batch!(User, resources).await?;
/// ```
#[macro_export]
macro_rules! update_resource_batch {
    ($resource:ty, $resources:expr) => {{
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;
        use time::{Duration, OffsetDateTime};

        async {
            let pool = get_connection().await;
            let resources: Vec<Vec<(&str, DatabaseValue)>> = $resources.clone();
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let updated_at = OffsetDateTime::now_utc();
            let expires_at = (OffsetDateTime::now_utc() + Duration::days(30));

            if resources.is_empty() {
                return Ok(Vec::<$resource>::new());
            }

            let resource = resources[0].clone();

            let mut fields = resource
                .clone()
                .iter()
                .map(|(field, _)| field.to_string())
                .collect::<Vec<String>>();

            if <$resource as DatabaseResource>::is_updatable() {
                if let Some(_) = fields.iter().position(|field| field == "updated_at") {
                    fields.push("updated_at".to_string());
                } else {
                    fields.push("updated_at".to_string());
                }
            }

            if <$resource as DatabaseResource>::is_expirable() {
                if let Some(_) = fields.iter().position(|field| field == "expires_at") {
                    fields.push("expires_at".to_string());
                } else {
                    fields.push("expires_at".to_string());
                }
            }

            let mut query = format!("UPDATE {} as t SET ", resource_name);

            for (i, field) in fields.iter().enumerate() {
                query.push_str(&format!("{} = v.{}", field, field));
                if i < fields.len() - 1 {
                    query.push_str(", ");
                }
            }

            query.push_str(" FROM (VALUES ");

            let mut values: Vec<DatabaseValue> = Vec::new();
            let mut resource_ids: Vec<String> = Vec::new();

            for (i, resource) in resources.iter().enumerate() {
                let mut input_params: Vec<(&str, DatabaseValue)> = resource.clone();
                if input_params.is_empty() {
                    return Err(anyhow::Error::msg("Params are empty"));
                }

                let id = input_params.iter().find(|(field, _)| field == &"id");
                if id.is_none() {
                    return Err(anyhow::Error::msg("ID not found"));
                };
                resource_ids.push(id.unwrap().1.to_string());

                if <$resource as DatabaseResource>::is_updatable() {
                    if let Some(idx) = input_params
                        .iter()
                        .position(|(field, _)| field == &"updated_at")
                    {
                        input_params[idx] = ("updated_at", updated_at.into());
                    } else {
                        input_params.push(("updated_at", updated_at.into()));
                    }
                }

                if <$resource as DatabaseResource>::is_expirable() {
                    if let Some(idx) = input_params
                        .iter()
                        .position(|(field, _)| field == &"expires_at")
                    {
                        input_params[idx] = ("expires_at", expires_at.into());
                    } else {
                        input_params.push(("expires_at", expires_at.into()));
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
                            value_query
                                .push_str(&format!("CAST(${} AS TIMESTAMP WITH TIME ZONE)", idx));
                        }
                        DatabaseValue::Int(_) => {
                            value_query.push_str(&format!("CAST(${} AS INTEGER)", idx));
                        }
                        DatabaseValue::Int32(_) => {
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

            query.push_str(&format!(") as v({})", fields.join(", ")));
            query.push_str(&format!(" WHERE t.id = v.id RETURNING *"));

            let mut query = sqlx::query(sqlx::AssertSqlSafe(query));
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
