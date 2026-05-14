#[macro_export]
macro_rules! upsert_resource {
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

            let pool = get_connection().await;

            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );

            let mut params: Vec<(String, DatabaseValue)> = Vec::new();
            for (field, value) in input_params.into_iter() {
                params.push((field.to_string(), value.clone()))
            }

            if <$resource as DatabaseResource>::has_id() {
                params.push(("id".to_string(), Uuid::new_v4().to_string().into()));
            }

            if <$resource as DatabaseResource>::is_creatable() {
                if let Some(idx) = params
                    .iter()
                    .position(|(field, _)| field.contains("created_at"))
                {
                    params[idx] = ("created_at".to_string(), OffsetDateTime::now_utc().into());
                } else {
                    params.push(("created_at".to_string(), OffsetDateTime::now_utc().into()));
                }
            }

            if <$resource as DatabaseResource>::is_updatable() {
                if let Some(idx) = params
                    .iter()
                    .position(|(field, _)| field.contains("updated_at"))
                {
                    params[idx] = ("updated_at".to_string(), OffsetDateTime::now_utc().into());
                } else {
                    params.push(("updated_at".to_string(), OffsetDateTime::now_utc().into()));
                }
            }

            if <$resource as DatabaseResource>::is_expirable() {
                if let Some(idx) = params
                    .iter()
                    .position(|(field, _)| field.contains("expires_at"))
                {
                    params[idx] = (
                        "expires_at".to_string(),
                        (OffsetDateTime::now_utc() + Duration::days(30)).into(),
                    );
                } else {
                    params.push((
                        "expires_at".to_string(),
                        (OffsetDateTime::now_utc() + Duration::days(30)).into(),
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
            query.push_str(") ON CONFLICT (id) DO UPDATE SET ");
            for (i, field) in fields.iter().enumerate() {
                query.push_str(format!("{} = EXCLUDED.{}", field, field).as_str());
                if i < fields.len() - 1 {
                    query.push_str(", ");
                }
            }
            query.push_str(" RETURNING *");
            let mut query = sqlx::query(&query);
            for (_, value) in values.iter().enumerate() {
                match value {
                    DatabaseValue::None => query = query.bind(Option::<String>::None),
                    _ => query = query.bind(value),
                }
            }
            match query.fetch_one(&pool).await {
                Ok(row) => Ok(<$resource as DatabaseResource>::from_row(&row)?),
                Err(e) => Err(e.into()),
            }
        }
    }};
}

#[macro_export]
macro_rules! upsert_resource_batch {
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

            let mut fields = resources[0]
                .clone()
                .iter()
                .map(|(field, _)| field.to_string())
                .collect::<Vec<String>>();

            if <$resource as DatabaseResource>::has_id() {
                if let Some(idx) = fields.iter().position(|field| field == "id") {
                    fields[idx] = "id".to_string();
                } else {
                    fields.push("id".to_string());
                }
            }

            if <$resource as DatabaseResource>::is_creatable() {
                if let Some(idx) = fields.iter().position(|field| field == "created_at") {
                    fields[idx] = "created_at".to_string();
                } else {
                    fields.push("created_at".to_string());
                }
            }

            if <$resource as DatabaseResource>::is_updatable() {
                if let Some(idx) = fields.iter().position(|field| field == "updated_at") {
                    fields[idx] = "updated_at".to_string();
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

            let mut query = format!("INSERT INTO {} (", resource_name);

            for (i, field) in fields.iter().enumerate() {
                query.push_str(field);
                if i < fields.len() - 1 {
                    query.push_str(", ");
                }
            }

            query.push_str(") VALUES (");

            let mut values: Vec<DatabaseValue> = Vec::new();

            for (idx, resource) in resources.iter().enumerate() {
                let mut input_params: Vec<(&str, DatabaseValue)> = resource.clone();
                if input_params.is_empty() {
                    return Err(anyhow::Error::msg("Params are empty"));
                }

                let id = Uuid::new_v4().to_string();

                if <$resource as DatabaseResource>::has_id() {
                    if let None = input_params.iter().position(|(field, _)| field == &"id") {
                        input_params[idx] = ("id", id.clone().into());
                    } else {
                        input_params.push(("id", id.clone().into()));
                    }
                }

                if <$resource as DatabaseResource>::is_creatable() {
                    if let None = input_params
                        .iter()
                        .position(|(field, _)| field == &"created_at")
                    {
                        input_params[idx] = ("created_at", created_at.clone().into());
                    } else {
                        input_params.push(("created_at", created_at.clone().into()));
                    }
                }

                if <$resource as DatabaseResource>::is_updatable() {
                    if let None = input_params
                        .iter()
                        .position(|(field, _)| field == &"updated_at")
                    {
                        input_params[idx] = ("updated_at", updated_at.clone().into());
                    } else {
                        input_params.push(("updated_at", updated_at.clone().into()));
                    }
                }

                if <$resource as DatabaseResource>::is_expirable() {
                    if let None = input_params
                        .iter()
                        .position(|(field, _)| field == &"expires_at")
                    {
                        input_params[idx] = ("expires_at", expires_at.clone().into());
                    } else {
                        input_params.push(("expires_at", expires_at.clone().into()));
                    }
                }

                let mut idxs: Vec<usize> = Vec::new();
                for (_, value) in input_params {
                    values.push(value.clone());
                    idxs.push(values.len() - 1);
                }

                let mut value_query = String::from("(");

                for (j, _) in fields.iter().enumerate() {
                    let idx = idxs[j];
                    let value = resource[idx].1.clone();
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
                    value_query.push_str(")");
                    if idx < resources.len() - 1 {
                        value_query.push_str(", ");
                    }
                }
                query.push_str(&value_query);
            }

            query.push_str(") ON CONFLICT (id) DO UPDATE SET ");
            for (i, field) in fields.iter().enumerate() {
                query.push_str(format!("{} = EXCLUDED.{}", field, field).as_str());
                if i < fields.len() - 1 {
                    query.push_str(", ");
                }
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
