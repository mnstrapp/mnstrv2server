use juniper::FieldError;

use crate::{graphql::Ctx, models::mnstr::Mnstr};

pub struct MnstrMutationType;

#[juniper::graphql_object]
impl MnstrMutationType {
    async fn collect(ctx: &Ctx, qr_code: String) -> Result<Mnstr, FieldError> {
        collect(ctx, qr_code).await
    }
    async fn update(
        ctx: &Ctx,
        id: String,
        name: Option<String>,
        description: Option<String>,
        qr_code: Option<String>,
        health: Option<i32>,
        max_health: Option<i32>,
        attack: Option<i32>,
        max_attack: Option<i32>,
        defense: Option<i32>,
        max_defense: Option<i32>,
        speed: Option<i32>,
        max_speed: Option<i32>,
        intelligence: Option<i32>,
        max_intelligence: Option<i32>,
        magic: Option<i32>,
        max_magic: Option<i32>,
    ) -> Result<Mnstr, FieldError> {
        update(
            ctx,
            id,
            name,
            description,
            qr_code,
            health,
            max_health,
            attack,
            max_attack,
            defense,
            max_defense,
            speed,
            max_speed,
            intelligence,
            max_intelligence,
            magic,
            max_magic,
        )
        .await
    }
}

pub async fn collect(ctx: &Ctx, qr_code: String) -> Result<Mnstr, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let session = ctx.session.as_ref().unwrap().clone();

    let mut mnstr = Mnstr::new(
        session.user_id.clone(),
        String::new(),
        String::new(),
        qr_code.clone(),
    );

    if let Some(error) = mnstr.create().await {
        println!("[collect] Failed to create mnstr: {:?}", error);
        return Err(FieldError::from("Failed to create mnstr"));
    }

    Ok(mnstr)
}

pub async fn update(
    ctx: &Ctx,
    id: String,
    name: Option<String>,
    description: Option<String>,
    qr_code: Option<String>,
    health: Option<i32>,
    max_health: Option<i32>,
    attack: Option<i32>,
    max_attack: Option<i32>,
    defense: Option<i32>,
    max_defense: Option<i32>,
    speed: Option<i32>,
    max_speed: Option<i32>,
    intelligence: Option<i32>,
    max_intelligence: Option<i32>,
    magic: Option<i32>,
    max_magic: Option<i32>,
) -> Result<Mnstr, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }

    let mut mnstr = Mnstr::find_one(id, false)
        .await
        .map_err(|e| FieldError::from(e.to_string()))?;

    mnstr.mnstr_name = name.unwrap_or(mnstr.mnstr_name);
    mnstr.mnstr_description = description.unwrap_or(mnstr.mnstr_description);
    mnstr.mnstr_qr_code = qr_code.unwrap_or(mnstr.mnstr_qr_code);
    mnstr.current_health = health.unwrap_or(mnstr.current_health);
    mnstr.max_health = max_health.unwrap_or(mnstr.max_health);
    mnstr.current_attack = attack.unwrap_or(mnstr.current_attack);
    mnstr.max_attack = max_attack.unwrap_or(mnstr.max_attack);
    mnstr.current_defense = defense.unwrap_or(mnstr.current_defense);
    mnstr.max_defense = max_defense.unwrap_or(mnstr.max_defense);
    mnstr.current_speed = speed.unwrap_or(mnstr.current_speed);
    mnstr.max_speed = max_speed.unwrap_or(mnstr.max_speed);
    mnstr.current_intelligence = intelligence.unwrap_or(mnstr.current_intelligence);
    mnstr.max_intelligence = max_intelligence.unwrap_or(mnstr.max_intelligence);
    mnstr.current_magic = magic.unwrap_or(mnstr.current_magic);
    mnstr.max_magic = max_magic.unwrap_or(mnstr.max_magic);

    if let Some(error) = mnstr.update().await {
        println!("[update] Failed to update mnstr: {:?}", error);
        return Err(FieldError::from("Failed to update mnstr"));
    }

    Ok(mnstr)
}
