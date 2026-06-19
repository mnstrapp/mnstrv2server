//! Query Macros for Database Operations
//!
//! This module provides macros for finding and retrieving resources from the database.
//! All macros work with any struct that implements the `DatabaseResource` trait.

/// Finds all resources matching the specified field conditions.
///
/// # Arguments
/// * `$resource` - The resource type (must implement DatabaseResource)
/// * `$params` - Vector of `(&str, DatabaseValue)` tuples for field conditions
///
/// # Returns
/// `Result<Vec<Resource>, Error>` - Vector of matching resources or database error
///
/// # Example
/// ```rust
/// let params = vec![
///     ("user_id", "123".into()),
///     ("status", "active".into())
/// ];
/// let results = find_all_resources_where_fields!(User, params).await?;
/// ```
#[macro_export]
macro_rules! find_all_resources_where_fields {
    ($resource:ty, $params:expr) => {{
        find_all_resources_where_fields!(
            $resource,
            $params,
            Option::<String>::None,
            Option::<String>::None
        )
    }};
    ($resource:ty, $params:expr, None, None) => {{
        find_all_resources_where_fields!(
            $resource,
            $params,
            Option::<String>::None,
            Option::<String>::None
        )
    }};
    ($resource:ty, $params:expr, None, $order_direction:expr) => {{
        find_all_resources_where_fields!(
            $resource,
            $params,
            Option::<String>::None,
            $order_direction
        )
    }};
    ($resource:ty, $params:expr, $order_by:expr, None) => {{ find_all_resources_where_fields!($resource, $params, $order_by, Option::<String>::None) }};
    ($resource:ty, $params:expr, $order_by:expr, $order_direction:expr) => {{
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            let params: Vec<(&str, DatabaseValue)> = $params.clone();
            let fields = params
                .iter()
                .map(|field| field.0.to_string())
                .collect::<Vec<String>>();
            let values = params
                .iter()
                .map(|field| field.1.clone())
                .collect::<Vec<DatabaseValue>>();

            let mut query = format!("SELECT * FROM {}", resource_name);
            if fields.len() > 0 {
                query.push_str(" WHERE ");
            }
            for (i, field) in fields.iter().enumerate() {
                query.push_str(&format!("{} = ${}", field, i + 1));
                if i < fields.len() - 1 {
                    query.push_str(" AND ");
                }
            }

            let order_by = match $order_by {
                Some(order_by) => order_by.to_string(),
                None => "updated_at".to_string(),
            };

            let order_direction = match $order_direction {
                Some(order_direction) => order_direction.to_string(),
                None => "DESC".to_string(),
            };

            query.push_str(&format!(" ORDER BY {} {}", order_by, order_direction));

            let mut query = sqlx::query(sqlx::AssertSqlSafe(query));
            for value in values.iter() {
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

/// Finds all unarchived resources matching the specified field conditions.
///
/// This macro generates a SELECT query that only returns resources where `archived_at IS NULL`.
///
/// # Arguments
/// * `$resource` - The resource type (must implement DatabaseResource)
/// * `$params` - Vector of `(&str, DatabaseValue)` tuples for field conditions
///
/// # Returns
/// `Result<Vec<Resource>, Error>` - Vector of unarchived resources or database error
///
/// # Example
/// ```rust
/// let params = vec![("organization_id", "456".into())];
/// let active_users = find_all_unarchived_resources_where_fields!(User, params).await?;
/// ```
#[macro_export]
macro_rules! find_all_unarchived_resources_where_fields {
    ($resource:ty, $params:expr) => {{
        find_all_unarchived_resources_where_fields!(
            $resource,
            $params,
            Option::<String>::None,
            Option::<String>::None
        )
    }};
    ($resource:ty, $params:expr, None, None) => {{
        find_all_unarchived_resources_where_fields!(
            $resource,
            $params,
            Option::<String>::None,
            Option::<String>::None
        )
    }};
    ($resource:ty, $params:expr, None, $order_direction:expr) => {{
        find_all_unarchived_resources_where_fields!(
            $resource,
            $params,
            Option::<String>::None,
            $order_direction
        )
    }};
    ($resource:ty, $params:expr, $order_by:expr, None) => {{
        find_all_unarchived_resources_where_fields!(
            $resource,
            $params,
            $order_by,
            Option::<String>::None
        )
    }};
    ($resource:ty, $params:expr, $order_by:expr, $order_direction:expr) => {{
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            let params: Vec<(&str, DatabaseValue)> = $params.clone();
            let fields = params
                .iter()
                .map(|field| field.0.to_string())
                .collect::<Vec<String>>();
            let values = params.iter().map(|field| &field.1).collect::<Vec<_>>();

            let mut query = format!("SELECT * FROM {} WHERE archived_at IS NULL", resource_name);
            if fields.len() > 0 {
                query.push_str(" AND ");
            }
            for (i, field) in fields.iter().enumerate() {
                query.push_str(&format!("{} = ${}", field, i + 1));
                if i < fields.len() - 1 {
                    query.push_str(" AND ");
                }
            }

            let mut query = sqlx::query(sqlx::AssertSqlSafe(query));
            for (_, value) in values.iter().enumerate() {
                query = query.bind(value);
            }

            let order_by = match $order_by {
                Some(order_by) => order_by.to_string(),
                None => "updated_at".to_string(),
            };

            let order_direction = match $order_direction {
                Some(order_direction) => order_direction.to_string(),
                None => "DESC".to_string(),
            };

            query.push_str(&format!(" ORDER BY {} {}", order_by, order_direction));

            match query.fetch_all(&pool).await {
                Ok(rows) => rows
                    .into_iter()
                    .map(|row| <$resource as DatabaseResource>::from_row(&row))
                    .collect::<Result<Vec<$resource>, _>>(),
                Err(e) => Err(anyhow::Error::msg(e.to_string())),
            }
        }
    }};
}

/// Finds all archived resources matching the specified field conditions.
///
/// This macro generates a SELECT query that only returns resources where `archived_at IS NOT NULL`.
///
/// # Arguments
/// * `$resource` - The resource type (must implement DatabaseResource)
/// * `$params` - Vector of `(&str, DatabaseValue)` tuples for field conditions
///
/// # Returns
/// `Result<Vec<Resource>, Error>` - Vector of archived resources or database error
///
/// # Example
/// ```rust
/// let params = vec![("organization_id", "456".into())];
/// let deleted_users = find_all_archived_resources_where_fields!(User, params).await?;
/// ```
#[macro_export]
macro_rules! find_all_archived_resources_where_fields {
    ($resource:ty, $params:expr) => {{
        find_all_archived_resources_where_fields!(
            $resource,
            $params,
            Option::<String>::None,
            Option::<String>::None
        )
    }};
    ($resource:ty, $params:expr, None, None) => {{
        find_all_archived_resources_where_fields!(
            $resource,
            $params,
            Option::<String>::None,
            Option::<String>::None
        )
    }};
    ($resource:ty, $params:expr, None, $order_direction:expr) => {{
        find_all_archived_resources_where_fields!(
            $resource,
            $params,
            Option::<String>::None,
            $order_direction
        )
    }};
    ($resource:ty, $params:expr, $order_by:expr, None) => {{
        find_all_archived_resources_where_fields!(
            $resource,
            $params,
            $order_by,
            Option::<String>::None
        )
    }};
    ($resource:ty, $params:expr, $order_by:expr, $order_direction:expr) => {{
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            let params: Vec<(&str, DatabaseValue)> = $params.clone();
            let fields = params
                .iter()
                .map(|field| field.0.to_string())
                .collect::<Vec<String>>();
            let values = params.iter().map(|field| &field.1).collect::<Vec<_>>();
            let mut query = format!(
                "SELECT * FROM {} WHERE archived_at IS NOT NULL",
                resource_name
            );
            if fields.len() > 0 {
                query.push_str(" AND ");
            }
            for (i, field) in fields.iter().enumerate() {
                query.push_str(&format!("{} = ${}", field, i + 1));
                if i < fields.len() - 1 {
                    query.push_str(" AND ");
                }
            }

            let order_by = match $order_by {
                Some(order_by) => order_by.to_string(),
                None => "updated_at".to_string(),
            };

            let order_direction = match $order_direction {
                Some(order_direction) => order_direction.to_string(),
                None => "DESC".to_string(),
            };

            query.push_str(&format!(" ORDER BY {} {}", order_by, order_direction));

            let mut query = sqlx::query(sqlx::AssertSqlSafe(query));
            for (_, value) in values.iter().enumerate() {
                query = query.bind(value);
            }

            match query.fetch_all(&pool).await {
                Ok(rows) => rows
                    .into_iter()
                    .map(|row| <$resource as DatabaseResource>::from_row(&row))
                    .collect::<Result<Vec<$resource>, _>>(),
                Err(e) => Err(anyhow::Error::msg(e.to_string())),
            }
        }
    }};
}

/// Finds a single resource matching the specified field conditions.
///
/// This macro generates a SELECT query with WHERE clauses and LIMIT 1 to return
/// exactly one resource. If multiple resources match, only the first one is returned.
///
/// # Arguments
/// * `$resource` - The resource type (must implement DatabaseResource)
/// * `$params` - Vector of `(&str, DatabaseValue)` tuples for field conditions
///
/// # Returns
/// `Result<Resource, Error>` - Single matching resource or database error
///
/// # Example
/// ```rust
/// let params = vec![("email", "user@example.com".into())];
/// let user = find_one_resource_where_fields!(User, params).await?;
/// ```
#[macro_export]
macro_rules! find_one_resource_where_fields {
    ($resource:ty, $params:expr) => {{
        find_one_resource_where_fields!(
            $resource,
            $params,
            Option::<String>::None,
            Option::<String>::None
        )
    }};
    ($resource:ty, $params:expr, None, None) => {{
        find_one_resource_where_fields!(
            $resource,
            $params,
            Option::<String>::None,
            Option::<String>::None
        )
    }};
    ($resource:ty, $params:expr, None, $order_direction:expr) => {{
        find_one_resource_where_fields!(
            $resource,
            $params,
            Option::<String>::None,
            $order_direction
        )
    }};
    ($resource:ty, $params:expr, $order_by:expr, None) => {{ find_one_resource_where_fields!($resource, $params, $order_by, Option::<String>::None) }};
    ($resource:ty, $params:expr, $order_by:expr, $order_direction:expr) => {{
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            let params: Vec<(&str, DatabaseValue)> = $params.clone();
            let fields = params
                .iter()
                .map(|field| field.0.to_string())
                .collect::<Vec<String>>();
            let values = params.iter().map(|field| &field.1).collect::<Vec<_>>();
            let mut query = format!("SELECT * FROM {}", resource_name);
            if fields.len() > 0 {
                query.push_str(" WHERE ");
            }
            for (i, field) in fields.iter().enumerate() {
                query.push_str(&format!("{} = ${}", field, i + 1));
                if i < fields.len() - 1 {
                    query.push_str(" AND ");
                }
            }

            query.push_str(" LIMIT 1");

            let mut query = sqlx::query(sqlx::AssertSqlSafe(query));
            for (_, value) in values.iter().enumerate() {
                query = query.bind(value);
            }

            match query.fetch_one(&pool).await {
                Ok(row) => Ok(<$resource as DatabaseResource>::from_row(&row)?),
                Err(e) => Err(anyhow::Error::msg(e.to_string())),
            }
        }
    }};
}

/// Finds a single unarchived resource matching the specified field conditions.
///
/// This macro generates a SELECT query that returns exactly one unarchived resource
/// (where `archived_at IS NULL`) with LIMIT 1 for efficiency.
///
/// # Arguments
/// * `$resource` - The resource type (must implement DatabaseResource)
/// * `$params` - Vector of `(&str, DatabaseValue)` tuples for field conditions
///
/// # Returns
/// `Result<Resource, Error>` - Single unarchived resource or database error
///
/// # Example
/// ```rust
/// let params = vec![("id", "789".into())];
/// let active_user = find_one_unarchived_resource_where_fields!(User, params).await?;
/// ```
#[macro_export]
macro_rules! find_one_unarchived_resource_where_fields {
    ($resource:ty, $params:expr) => {{
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            let params: Vec<(&str, DatabaseValue)> = $params.clone();
            let fields = params
                .iter()
                .map(|field| field.0.to_string())
                .collect::<Vec<String>>();
            let values = params.iter().map(|field| &field.1).collect::<Vec<_>>();
            let mut query = format!("SELECT * FROM {} WHERE archived_at IS NULL", resource_name);
            if fields.len() > 0 {
                query.push_str(" AND ");
            }
            for (i, field) in fields.iter().enumerate() {
                query.push_str(&format!("{} = ${}", field, i + 1));
                if i < fields.len() - 1 {
                    query.push_str(" AND ");
                }
            }

            query.push_str(" LIMIT 1");

            let mut query = sqlx::query(sqlx::AssertSqlSafe(query));
            for (_, value) in values.iter().enumerate() {
                query = query.bind(value);
            }

            match query.fetch_one(&pool).await {
                Ok(row) => Ok(<$resource as DatabaseResource>::from_row(&row)?),
                Err(e) => Err(e),
            }
        }
    }};
}

/// Finds a single archived resource matching the specified field conditions.
///
/// This macro generates a SELECT query that returns exactly one archived resource
/// (where `archived_at IS NOT NULL`) with LIMIT 1 for efficiency.
///
/// # Arguments
/// * `$resource` - The resource type (must implement DatabaseResource)
/// * `$params` - Vector of `(&str, DatabaseValue)` tuples for field conditions
///
/// # Returns
/// `Result<Resource, Error>` - Single archived resource or database error
///
/// # Example
/// ```rust
/// let params = vec![("id", "789".into())];
/// let deleted_user = find_one_archived_resource_where_fields!(User, params).await?;
/// ```
#[macro_export]
macro_rules! find_one_archived_resource_where_fields {
    ($resource:ty, $params:expr) => {{
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            let mut query = format!(
                "SELECT * FROM {} WHERE archived_at IS NOT NULL",
                resource_name
            );
            if fields.len() > 0 {
                query.push_str(" AND ");
            }

            let params: Vec<(&str, DatabaseValue)> = $params.clone();
            let fields = params
                .iter()
                .map(|field| field.0.to_string())
                .collect::<Vec<String>>();

            for (i, field) in fields.iter().enumerate() {
                query.push_str(&format!("{} = ${}", field, i + 1));
                if i < fields.len() - 1 {
                    query.push_str(" AND ");
                }
            }

            query.push_str(" LIMIT 1");

            let mut query = sqlx::query(&query);
            for (_, value) in params.iter().enumerate() {
                query = query.bind(value.1.clone());
            }

            match query.fetch_one(&pool).await {
                Ok(row) => Ok(<$resource as DatabaseResource>::from_row(&row)?),
                Err(e) => Err(anyhow::Error::msg(e.to_string())),
            }
        }
    }};
}

/// Finds all resources matching the specified field conditions with LIKE operator.
///
/// This macro generates a SELECT query that returns all resources where the specified fields
/// contain the search term (case-insensitive).
///
/// # Arguments
/// * `$resource` - The resource type (must implement DatabaseResource)
/// * `$params` - Vector of `(&str, DatabaseValue)` tuples for field conditions
///
/// # Returns
/// `Result<Vec<Resource>, Error>` - Vector of matching resources or database error
///
/// # Example
/// ```rust
/// let search_term = "john";
/// let params = vec![
///     ("first_name", search_term.clone().into()),
///     ("last_name", search_term.clone().into()),
///     ("type", search_term.clone().into()),
///     ("dob", search_term.clone().into()),
///     ("source_id", search_term.clone().into()),
/// ];
/// let results = find_all_resources_where_fields_like!(OrganizationContact, params).await?;
/// ```
#[macro_export]
macro_rules! find_all_resources_where_fields_like {
    ($resource:ty, $params:expr, $search_term:expr) => {{
        find_all_resources_where_fields_like!(
            $resource,
            $params,
            $search_term,
            Option::<String>::None,
            Option::<String>::None
        )
    }};
    ($resource:ty, $params:expr, $search_term:expr, None, None) => {{
        find_all_resources_where_fields_like!(
            $resource,
            $params,
            $search_term,
            Option::<String>::None,
            Option::<String>::None
        )
    }};
    ($resource:ty, $params:expr, $search_term:expr, None, $order_direction:expr) => {{
        find_all_resources_where_fields_like!(
            $resource,
            $params,
            $search_term,
            Option::<String>::None,
            $order_direction
        )
    }};
    ($resource:ty, $params:expr, $search_term:expr, $order_by:expr, None) => {{
        find_all_resources_where_fields_like!(
            $resource,
            $params,
            $search_term,
            $order_by,
            Option::<String>::None
        )
    }};
    ($resource:ty, $params:expr, $search_term:expr, $order_by:expr, $order_direction:expr) => {{
        use crate::database::{connection::get_connection, traits::DatabaseResource};
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            let params: Vec<&str> = $params.clone();

            let mut query = format!("SELECT * FROM {}", resource_name);
            if params.len() > 0 {
                query.push_str(" WHERE ");
            }
            for (i, field) in params.iter().enumerate() {
                query.push_str(&format!("{} ILIKE ${}", field, i + 1));
                if i < params.len() - 1 {
                    query.push_str(" OR ");
                }
            }

            let order_by = match $order_by {
                Some(order_by) => order_by.to_string(),
                None => "updated_at".to_string(),
            };

            let order_direction = match $order_direction {
                Some(order_direction) => order_direction.to_string(),
                None => "DESC".to_string(),
            };

            query.push_str(&format!(" ORDER BY {} {}", order_by, order_direction));

            let mut query = sqlx::query(sqlx::AssertSqlSafe(query));
            for _ in params.iter() {
                query = query.bind(format!("%{}%", $search_term));
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

/// Finds all resources matching the specified field conditions with IN operator.
///
/// This macro generates a SELECT query that returns all resources where the specified field
/// is in the list of values.
///
/// # Arguments
/// * `$resource` - The resource type (must implement DatabaseResource)
/// * `$field` - The field to check for IN condition
/// * `$values` - Vector of values to check for IN condition
///
/// # Returns
/// `Result<Vec<Resource>, Error>` - Vector of matching resources or database error
///
/// # Example
/// ```rust
/// let values = vec!["123", "456", "789"];
/// let results = find_all_resources_where_fields_in!(User, "id", values).await?;
/// ```
#[macro_export]
macro_rules! find_all_resources_where_fields_in {
    ($resource:ty, $field:expr, $values:expr) => {{
        use crate::database::{connection::get_connection, traits::DatabaseResource};
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            let mut query = format!("SELECT * FROM {}", resource_name);
            query.push_str(&format!(" WHERE {} IN (", $field));
            for (i, _) in $values.iter().enumerate() {
                query.push_str(&format!("${}", i + 1));
                if i < $values.len() - 1 {
                    query.push_str(", ");
                }
            }
            query.push_str(")");

            let mut query = sqlx::query(sqlx::AssertSqlSafe(query));
            for value in $values.iter() {
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
