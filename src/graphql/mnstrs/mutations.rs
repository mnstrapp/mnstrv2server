use juniper::{FieldError, GraphQLInputObject};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{database::values::DatabaseValue, graphql::Ctx, models::{mnstr::{DEFAULT_STAT_VALUE, Mnstr}, session::Session}, utils::sessions::{get_user_from_token}};

#[derive(Debug, Serialize, Deserialize, GraphQLInputObject, Clone)]
pub struct BatchMnstrInput {
    pub mnstrs: Vec<MnstrInput>,
}

#[derive(Debug, Serialize, Deserialize, GraphQLInputObject, Clone)]
pub struct MnstrInput {
    pub id: Option<String>,
    pub user_id: String,
    pub mnstr_name: Option<String>,
    pub mnstr_description: Option<String>,
    pub mnstr_qr_code: Option<String>,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
    pub archived_at: Option<OffsetDateTime>,
    pub current_level: Option<i32>,
    pub current_experience: Option<i32>,
    pub current_health: Option<i32>,
    pub max_health: Option<i32>,
    pub current_attack: Option<i32>,
    pub max_attack: Option<i32>,
    pub current_defense: Option<i32>,
    pub max_defense: Option<i32>,
    pub current_speed: Option<i32>,
    pub max_speed: Option<i32>,
    pub current_intelligence: Option<i32>,
    pub max_intelligence: Option<i32>,
    pub current_magic: Option<i32>,
    pub max_magic: Option<i32>,
    pub experience_to_next_level: Option<i32>,
}
pub struct MnstrMutationType;

#[juniper::graphql_object]
impl MnstrMutationType {
    async fn collect(ctx: &Ctx, mnstr_qr_code: String) -> Result<Mnstr, FieldError> {
        collect(ctx, mnstr_qr_code).await
    }

    async fn create(
        ctx: &Ctx,
        mnstr_name: Option<String>,
        mnstr_description: Option<String>,
        mnstr_qr_code: Option<String>,
        current_health: Option<i32>,
        max_health: Option<i32>,
        current_attack: Option<i32>,
        max_attack: Option<i32>,
        current_defense: Option<i32>,
        max_defense: Option<i32>,
        current_speed: Option<i32>,
        max_speed: Option<i32>,
        current_intelligence: Option<i32>,
        max_intelligence: Option<i32>,
        current_magic: Option<i32>,
        max_magic: Option<i32>,
    ) -> Result<Mnstr, FieldError> {
        create(
            ctx,
            mnstr_name,
            mnstr_description,
            mnstr_qr_code,
            current_health,
            max_health,
            current_attack,
            max_attack,
            current_defense,
            max_defense,
            current_speed,
            max_speed,
            current_intelligence,
            max_intelligence,
            current_magic,
            max_magic,
        )
        .await
    }

    async fn create_batch(ctx: &Ctx, mnstrs: BatchMnstrInput) -> Result<Vec<Mnstr>, FieldError> {
        create_batch(ctx, mnstrs.mnstrs).await
    }

    async fn update(
        ctx: &Ctx,
        id: String,
        mnstr_name: Option<String>,
        mnstr_description: Option<String>,
        mnstr_qr_code: Option<String>,
        current_health: Option<i32>,
        max_health: Option<i32>,
        current_attack: Option<i32>,
        max_attack: Option<i32>,
        current_defense: Option<i32>,
        max_defense: Option<i32>,
        current_speed: Option<i32>,
        max_speed: Option<i32>,
        current_intelligence: Option<i32>,
        max_intelligence: Option<i32>,
        current_magic: Option<i32>,
        max_magic: Option<i32>,
    ) -> Result<Mnstr, FieldError> {
        update(
            ctx,
            id,
            mnstr_name,
            mnstr_description,
            mnstr_qr_code,
            current_health,
            max_health,
            current_attack,
            max_attack,
            current_defense,
            max_defense,
            current_speed,
            max_speed,
            current_intelligence,
            max_intelligence,
            current_magic,
            max_magic,
        )
        .await
    }

    async fn update_batch(ctx: &Ctx, mnstrs: BatchMnstrInput) -> Result<Vec<Mnstr>, FieldError> {
        update_batch(ctx, mnstrs.mnstrs).await
    }
}

pub async fn collect(ctx: &Ctx, mnstr_qr_code: String) -> Result<Mnstr, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let session = ctx.session.as_ref().unwrap().clone();
    let user = match get_user_from_token::<Session>(session.session_token.clone()).await {
        Ok(user) => user,
        Err(e) => {
            println!("[collect] Failed to get user: {:?}", e);
            return Err(FieldError::from("Failed to get user"));
        }
    };

    let mut mnstr = Mnstr::new(
        user.id.clone(),
        None,
        None,
        mnstr_qr_code,
    );

    if let Some(error) = mnstr.create().await {
        println!("[collect] Failed to create mnstr: {:?}", error);
        return Err(FieldError::from("Failed to create mnstr"));
    }

    Ok(mnstr)
}

pub async fn create(
    ctx: &Ctx,
    mnstr_name: Option<String>,
    mnstr_description: Option<String>,
    mnstr_qr_code: Option<String>,
    current_health: Option<i32>,
    max_health: Option<i32>,
    current_attack: Option<i32>,
    max_attack: Option<i32>,
    current_defense: Option<i32>,
    max_defense: Option<i32>,
    current_speed: Option<i32>,
    max_speed: Option<i32>,
    current_intelligence: Option<i32>,
    max_intelligence: Option<i32>,
    current_magic: Option<i32>,
    max_magic: Option<i32>,
) -> Result<Mnstr, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let session = ctx.session.as_ref().unwrap().clone();
    let user = match get_user_from_token::<Session>(session.session_token.clone()).await {
        Ok(user) => user,
        Err(e) => {
            println!("[create] Failed to get user: {:?}", e);
            return Err(FieldError::from("Failed to get user"));
        }
    };

    let mut mnstr = Mnstr::new(
        user.id.clone(),
        mnstr_name,
        mnstr_description,
        mnstr_qr_code.unwrap_or(String::new()),
    );

    mnstr.current_health = current_health.unwrap_or(DEFAULT_STAT_VALUE);
    mnstr.max_health = max_health.unwrap_or(DEFAULT_STAT_VALUE);
    mnstr.current_attack = current_attack.unwrap_or(DEFAULT_STAT_VALUE);
    mnstr.max_attack = max_attack.unwrap_or(DEFAULT_STAT_VALUE);
    mnstr.current_defense = current_defense.unwrap_or(DEFAULT_STAT_VALUE);
    mnstr.max_defense = max_defense.unwrap_or(DEFAULT_STAT_VALUE);
    mnstr.current_speed = current_speed.unwrap_or(DEFAULT_STAT_VALUE);
    mnstr.max_speed = max_speed.unwrap_or(DEFAULT_STAT_VALUE);
    mnstr.current_intelligence = current_intelligence.unwrap_or(DEFAULT_STAT_VALUE);
    mnstr.max_intelligence = max_intelligence.unwrap_or(DEFAULT_STAT_VALUE);
    mnstr.current_magic = current_magic.unwrap_or(DEFAULT_STAT_VALUE);
    mnstr.max_magic = max_magic.unwrap_or(DEFAULT_STAT_VALUE);

    if let Some(error) = mnstr.create().await {
        println!("[create] Failed to create mnstr: {:?}", error);
        return Err(FieldError::from("Failed to create mnstr"));
    }

    Ok(mnstr)
}

pub async fn create_batch(ctx: &Ctx, mnstrs: Vec<MnstrInput>) -> Result<Vec<Mnstr>, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let session = ctx.session.as_ref().unwrap().clone();
    let user = match get_user_from_token::<Session>(session.session_token.clone()).await {
        Ok(user) => user,
        Err(e) => {
            println!("[create_batch] Failed to get user: {:?}", e);
            return Err(FieldError::from("Failed to get user"));
        }
    };

    let mnstrs = mnstrs
        .iter()
        .filter(|mnstr_input| mnstr_input.mnstr_qr_code.is_some())
        .map(|mnstr_input| {
            let mut mnstr_params: Vec<(&str, Option<DatabaseValue>)> = Vec::new();
            mnstr_params.push(("user_id", Some(user.id.clone().into())));
            mnstr_params.push((
                "mnstr_name",
                mnstr_input.mnstr_name.as_ref().map(|s| s.into()),
            ));
            mnstr_params.push((
                "mnstr_description",
                mnstr_input.mnstr_description.as_ref().map(|s| s.into()),
            ));
            mnstr_params.push((
                "mnstr_qr_code",
                mnstr_input.mnstr_qr_code.as_ref().map(|s| s.into()),
            ));
            mnstr_params.push((
                "current_health",
                mnstr_input.current_health.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
            ));
            mnstr_params.push(("max_health", mnstr_input.max_health.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into()))));
            mnstr_params.push((
                "current_attack",
                mnstr_input.current_attack.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
            ));
            mnstr_params.push(("max_attack", mnstr_input.max_attack.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into()))));
            mnstr_params.push((
                "current_defense",
                mnstr_input.current_defense.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
            ));
            mnstr_params.push(("max_defense", mnstr_input.max_defense.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into()))));
            mnstr_params.push(("current_speed", mnstr_input.current_speed.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into()))));
            mnstr_params.push(("max_speed", mnstr_input.max_speed.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into()))));
            mnstr_params.push((
                "current_intelligence",
                mnstr_input.current_intelligence.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
            ));
            mnstr_params.push((
                "max_intelligence",
                mnstr_input.max_intelligence.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
            ));
            mnstr_params.push(("current_magic", mnstr_input.current_magic.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into()))));
            mnstr_params.push(("max_magic", mnstr_input.max_magic.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into()))));
            mnstr_params
        })
        .collect::<Vec<Vec<(&str, Option<DatabaseValue>)>>>();

    match Mnstr::create_batch(user.id.clone(), mnstrs).await {
        Ok(mnstrs) => Ok(mnstrs),
        Err(e) => {
            println!("[create_batch] Failed to create mnstrs: {:?}", e);
            return Err(FieldError::from(e.to_string()));
        }
    }
}

pub async fn update(
    ctx: &Ctx,
    id: String,
    mnstr_name: Option<String>,
    mnstr_description: Option<String>,
    mnstr_qr_code: Option<String>,
    current_health: Option<i32>,
    max_health: Option<i32>,
    current_attack: Option<i32>,
    max_attack: Option<i32>,
    current_defense: Option<i32>,
    max_defense: Option<i32>,
    current_speed: Option<i32>,
    max_speed: Option<i32>,
    current_intelligence: Option<i32>,
    max_intelligence: Option<i32>,
    current_magic: Option<i32>,
    max_magic: Option<i32>,
) -> Result<Mnstr, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }

    let mut mnstr = match Mnstr::find_one(id, false).await {
        Ok(mnstr) => mnstr,
        Err(e) => {
            println!("[update] Failed to find mnstr: {:?}", e);
            return Err(FieldError::from(e.to_string()));
        }
    };

    mnstr.mnstr_name = mnstr_name.unwrap_or(mnstr.mnstr_name);
    mnstr.mnstr_description = mnstr_description.unwrap_or(mnstr.mnstr_description);
    mnstr.mnstr_qr_code = mnstr_qr_code.unwrap_or(mnstr.mnstr_qr_code);
    mnstr.current_health = current_health.unwrap_or(mnstr.current_health);
    mnstr.max_health = max_health.unwrap_or(mnstr.max_health);
    mnstr.current_attack = current_attack.unwrap_or(mnstr.current_attack);
    mnstr.max_attack = max_attack.unwrap_or(mnstr.max_attack);
    mnstr.current_defense = current_defense.unwrap_or(mnstr.current_defense);
    mnstr.max_defense = max_defense.unwrap_or(mnstr.max_defense);
    mnstr.current_speed = current_speed.unwrap_or(mnstr.current_speed);
    mnstr.max_speed = max_speed.unwrap_or(mnstr.max_speed);
    mnstr.current_intelligence = current_intelligence.unwrap_or(mnstr.current_intelligence);
    mnstr.max_intelligence = max_intelligence.unwrap_or(mnstr.max_intelligence);
    mnstr.current_magic = current_magic.unwrap_or(mnstr.current_magic);
    mnstr.max_magic = max_magic.unwrap_or(mnstr.max_magic);

    if let Some(error) = mnstr.update().await {
        println!("[update] Failed to update mnstr: {:?}", error);
        return Err(FieldError::from("Failed to update mnstr"));
    }

    Ok(mnstr)
}

pub async fn update_batch(
    ctx: &Ctx,
    mnstr_inputs: Vec<MnstrInput>,
) -> Result<Vec<Mnstr>, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let session = ctx.session.as_ref().unwrap().clone();
    let user = match get_user_from_token::<Session>(session.session_token.clone()).await {
        Ok(user) => user,
        Err(e) => {
            println!("[update_batch] Failed to get user: {:?}", e);
            return Err(FieldError::from("Failed to get user"));
        }
    };

    let mnstrs = mnstr_inputs
        .iter()
        .filter(|mnstr_input| mnstr_input.mnstr_qr_code.is_some())
        .map(|mnstr_input| {
            let mut mnstr_params: Vec<(&str, Option<DatabaseValue>)> = Vec::new();
            mnstr_params.push(("id", mnstr_input.id.as_ref().map(|s| s.into())));
            mnstr_params.push(("user_id", Some(user.id.clone().into())));
            mnstr_params.push((
                "mnstr_name",
                mnstr_input.mnstr_name.as_ref().map(|s| s.into()),
            ));
            mnstr_params.push((
                "mnstr_description",
                mnstr_input.mnstr_description.as_ref().map(|s| s.into()),
            ));
            mnstr_params.push((
                "mnstr_qr_code",
                mnstr_input.mnstr_qr_code.as_ref().map(|s| s.into()),
            ));
            mnstr_params.push(("current_level", mnstr_input.current_level.map_or(Some(0.into()), |i| Some(i.into()))));
            mnstr_params.push((
                "current_experience",
                mnstr_input.current_experience.map_or(Some(0.into()), |i| Some(i.into())),
            ));
            mnstr_params.push((
                "current_health",
                mnstr_input.current_health.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
            ));
            mnstr_params.push(("max_health", mnstr_input.max_health.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into()))));
            mnstr_params.push((
                "current_attack",
                mnstr_input.current_attack.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
            ));
            mnstr_params.push(("max_attack", mnstr_input.max_attack.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into()))));
            mnstr_params.push((
                "current_defense",
                mnstr_input.current_defense.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
            ));
            mnstr_params.push(("max_defense", mnstr_input.max_defense.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into()))));
            mnstr_params.push(("current_speed", mnstr_input.current_speed.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into()))));
            mnstr_params.push(("max_speed", mnstr_input.max_speed.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into()))));
            mnstr_params.push((
                "current_intelligence",
                mnstr_input.current_intelligence.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
            ));
            mnstr_params.push((
                "max_intelligence",
                mnstr_input.max_intelligence.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
            ));
            mnstr_params.push(("current_magic", mnstr_input.current_magic.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into()))));
            mnstr_params.push(("max_magic", mnstr_input.max_magic.map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into()))));
            mnstr_params
        })
        .collect::<Vec<Vec<(&str, Option<DatabaseValue>)>>>();

    let mnstrs = match Mnstr::update_batch(user.id.clone(), mnstrs).await {
        Ok(mnstrs) => mnstrs,
        Err(e) => {
            println!("[update_batch] Failed to update mnstrs: {:?}", e);
            return Err(FieldError::from("Failed to update mnstrs"));
        }
    };

    Ok(mnstrs)
}
