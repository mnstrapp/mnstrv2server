use rocket::serde;
use serde::{Deserialize, Serialize};
use sqlx::{Error, Row, postgres::PgRow};
use time::OffsetDateTime;

use crate::{
    database::{traits::DatabaseResource, values::DatabaseValue},
    delete_resource_where_fields, find_all_resources_where_fields, find_one_resource_where_fields,
    insert_resource, update_resource,
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Battle {
    pub id: String,
    pub challenger_id: String,
    pub challenger_name: String,
    pub challenger_mnstr_id: Option<String>,
    pub opponent_id: String,
    pub opponent_name: String,
    pub opponent_mnstr_id: Option<String>,
    pub winner_id: Option<String>,
    pub winner_mnstr_id: Option<String>,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub created_at: Option<OffsetDateTime>,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub updated_at: Option<OffsetDateTime>,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub archived_at: Option<OffsetDateTime>,
}

impl Battle {
    pub fn new(
        challenger_id: String,
        challenger_name: String,
        opponent_id: String,
        opponent_name: String,
    ) -> Self {
        Self {
            id: "".to_string(),
            challenger_id,
            challenger_name,
            challenger_mnstr_id: None,
            opponent_id,
            opponent_name,
            opponent_mnstr_id: None,
            winner_id: None,
            winner_mnstr_id: None,
            created_at: None,
            updated_at: None,
            archived_at: None,
        }
    }

    pub async fn create(&mut self) -> Option<anyhow::Error> {
        let params = vec![
            ("challenger_id", self.challenger_id.clone().into()),
            ("challenger_name", self.challenger_name.clone().into()),
            ("opponent_id", self.opponent_id.clone().into()),
            ("opponent_name", self.opponent_name.clone().into()),
        ];
        let battle = match insert_resource!(Battle, params).await {
            Ok(battle) => battle,
            Err(e) => return Some(e.into()),
        };
        *self = battle;
        None
    }

    pub async fn update(&mut self) -> Option<anyhow::Error> {
        let params = vec![
            (
                "challenger_mnstr_id",
                self.challenger_mnstr_id.clone().into(),
            ),
            ("opponent_mnstr_id", self.opponent_mnstr_id.clone().into()),
            ("winner_id", self.winner_id.clone().into()),
            ("winner_mnstr_id", self.winner_mnstr_id.clone().into()),
        ];
        let battle = match update_resource!(Battle, self.id.clone(), params).await {
            Ok(battle) => battle,
            Err(e) => return Some(e.into()),
        };
        *self = battle;
        None
    }

    pub async fn delete(&mut self) -> Option<anyhow::Error> {
        let params = vec![("id", self.id.clone().into())];
        match delete_resource_where_fields!(Battle, params).await {
            Ok(_) => (),
            Err(e) => return Some(e.into()),
        };
        None
    }

    pub async fn find_one(id: String) -> Result<Self, anyhow::Error> {
        let battle =
            match find_one_resource_where_fields!(Battle, vec![("id", id.clone().into())]).await {
                Ok(battle) => battle,
                Err(e) => return Err(e.into()),
            };
        Ok(battle)
    }

    pub async fn find_one_by(params: Vec<(&str, DatabaseValue)>) -> Result<Self, anyhow::Error> {
        let battle = match find_one_resource_where_fields!(Battle, params).await {
            Ok(battle) => battle,
            Err(e) => return Err(e.into()),
        };
        Ok(battle)
    }

    pub async fn find_all() -> Result<Vec<Self>, anyhow::Error> {
        let battles = match find_all_resources_where_fields!(Battle, vec![]).await {
            Ok(battles) => battles,
            Err(e) => return Err(e.into()),
        };
        Ok(battles)
    }

    pub async fn find_all_by(
        params: Vec<(&str, DatabaseValue)>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let battles = match find_all_resources_where_fields!(Battle, params).await {
            Ok(battles) => battles,
            Err(e) => return Err(e.into()),
        };
        Ok(battles)
    }
}

impl DatabaseResource for Battle {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        let created_at = row.get("created_at");
        let updated_at = row.get("updated_at");
        let archived_at = match row.get("archived_at") {
            Some(archived_at) => archived_at,
            None => None,
        };
        let winner_id = match row.get("winner_id") {
            Some(winner_id) => winner_id,
            None => None,
        };
        let winner_mnstr_id = match row.get("winner_mnstr_id") {
            Some(winner_mnstr_id) => winner_mnstr_id,
            None => None,
        };

        Ok(Battle {
            id: row.get("id"),
            challenger_id: row.get("challenger_id"),
            challenger_name: row.get("challenger_name"),
            challenger_mnstr_id: row.get("challenger_mnstr_id"),
            opponent_id: row.get("opponent_id"),
            opponent_name: row.get("opponent_name"),
            opponent_mnstr_id: row.get("opponent_mnstr_id"),
            winner_id,
            winner_mnstr_id,
            created_at,
            updated_at,
            archived_at,
        })
    }
    fn has_id() -> bool {
        true
    }
    fn is_archivable() -> bool {
        true
    }
    fn is_updatable() -> bool {
        true
    }
    fn is_creatable() -> bool {
        true
    }
    fn is_expirable() -> bool {
        false
    }
    fn is_verifiable() -> bool {
        false
    }
}
