use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
use sqlx::{Error, Row, postgres::PgRow};
use time::OffsetDateTime;

use crate::{
    database::{traits::DatabaseResource, values::DatabaseValue},
    find_all_resources_where_fields, find_one_resource_where_fields, insert_resource,
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};

#[derive(Debug, Serialize, Deserialize, GraphQLObject, Clone)]
pub struct Mnstr {
    pub id: String,
    pub user_id: String,

    #[graphql(name = "name")]
    pub mnstr_name: String,

    #[graphql(name = "description")]
    pub mnstr_description: String,

    #[graphql(name = "qrCode")]
    pub mnstr_qr_code: String,

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

    pub current_level: i32,
    pub current_experience: i32,
    pub current_health: i32,
    pub max_health: i32,
    pub current_attack: i32,
    pub max_attack: i32,
    pub current_defense: i32,
    pub max_defense: i32,
    pub current_speed: i32,
    pub max_speed: i32,
    pub current_intelligence: i32,
    pub max_intelligence: i32,
    pub current_magic: i32,
    pub max_magic: i32,
    // Relationships
}

impl Mnstr {
    pub fn new(
        user_id: String,
        mnstr_name: String,
        mnstr_description: String,
        mnstr_qr_code: String,
    ) -> Self {
        Self {
            id: "".to_string(),
            user_id,
            mnstr_name,
            mnstr_description,
            mnstr_qr_code,
            created_at: None,
            updated_at: None,
            archived_at: None,
            current_level: 0,
            current_experience: 0,
            current_health: 0,
            max_health: 0,
            current_attack: 0,
            max_attack: 0,
            current_defense: 0,
            max_defense: 0,
            current_speed: 0,
            max_speed: 0,
            current_intelligence: 0,
            max_intelligence: 0,
            current_magic: 0,
            max_magic: 0,
        }
    }

    pub async fn create(&mut self) -> Option<anyhow::Error> {
        let mnstr = match insert_resource!(
            Mnstr,
            vec![
                ("user_id", self.user_id.clone().into()),
                ("mnstr_name", self.mnstr_name.clone().into()),
                ("mnstr_description", self.mnstr_description.clone().into()),
                ("mnstr_qr_code", self.mnstr_qr_code.clone().into())
            ]
        )
        .await
        {
            Ok(mnstr) => mnstr,
            Err(e) => return Some(e.into()),
        };
        *self = mnstr;
        None
    }

    pub async fn find_one(id: String) -> Result<Self, anyhow::Error> {
        let mut mnstr =
            match find_one_resource_where_fields!(Mnstr, vec![("id", id.clone().into())]).await {
                Ok(mnstr) => mnstr,
                Err(e) => return Err(e.into()),
            };
        mnstr
            .get_relationships()
            .await
            .ok_or(anyhow::anyhow!("Failed to get relationships"))?;
        Ok(mnstr)
    }

    pub async fn find_one_by(params: Vec<(&str, DatabaseValue)>) -> Result<Self, anyhow::Error> {
        let mut mnstr = match find_one_resource_where_fields!(Mnstr, params).await {
            Ok(mnstr) => mnstr,
            Err(e) => return Err(e.into()),
        };
        mnstr
            .get_relationships()
            .await
            .ok_or(anyhow::anyhow!("Failed to get relationships"))?;
        Ok(mnstr)
    }

    pub async fn find_all() -> Result<Vec<Self>, anyhow::Error> {
        let mut mnstrs = match find_all_resources_where_fields!(Mnstr, vec![]).await {
            Ok(mnstrs) => mnstrs,
            Err(e) => return Err(e.into()),
        };
        for mnstr in mnstrs.iter_mut() {
            mnstr
                .get_relationships()
                .await
                .ok_or(anyhow::anyhow!("Failed to get relationships"))?;
        }
        Ok(mnstrs)
    }

    pub async fn find_all_by(
        params: Vec<(&str, DatabaseValue)>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let mut mnstrs = match find_all_resources_where_fields!(Mnstr, params).await {
            Ok(mnstrs) => mnstrs,
            Err(e) => return Err(e.into()),
        };
        for mnstr in mnstrs.iter_mut() {
            mnstr
                .get_relationships()
                .await
                .ok_or(anyhow::anyhow!("Failed to get relationships"))?;
        }
        Ok(mnstrs)
    }

    pub async fn get_relationships(&mut self) -> Option<Error> {
        None
    }
}

impl DatabaseResource for Mnstr {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        let created_at = row.get("created_at");
        let updated_at = row.get("updated_at");
        let archived_at = match row.get("archived_at") {
            Some(archived_at) => archived_at,
            None => None,
        };

        Ok(Mnstr {
            id: row.get("id"),
            user_id: row.get("user_id"),
            mnstr_name: row.get("mnstr_name"),
            mnstr_description: row.get("mnstr_description"),
            mnstr_qr_code: row.get("mnstr_qr_code"),
            created_at,
            updated_at,
            archived_at,
            current_level: row.get("current_level"),
            current_experience: row.get("current_experience"),
            current_health: row.get("current_health"),
            max_health: row.get("max_health"),
            current_attack: row.get("current_attack"),
            max_attack: row.get("max_attack"),
            current_defense: row.get("current_defense"),
            max_defense: row.get("max_defense"),
            current_speed: row.get("current_speed"),
            max_speed: row.get("max_speed"),
            current_intelligence: row.get("current_intelligence"),
            max_intelligence: row.get("max_intelligence"),
            current_magic: row.get("current_magic"),
            max_magic: row.get("max_magic"),
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
