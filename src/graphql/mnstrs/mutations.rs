use juniper::FieldError;

use crate::{graphql::Ctx, models::mnstr::Mnstr};

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
}

pub async fn collect(ctx: &Ctx, mnstr_qr_code: String) -> Result<Mnstr, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let session = ctx.session.as_ref().unwrap().clone();

    let mut mnstr = Mnstr::new(
        session.user_id.clone(),
        String::new(),
        String::new(),
        mnstr_qr_code.clone(),
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

    let mut mnstr = Mnstr::new(
        session.user_id.clone(),
        mnstr_name.unwrap_or(String::new()),
        mnstr_description.unwrap_or(String::new()),
        mnstr_qr_code.unwrap_or(String::new()),
    );

    mnstr.current_health = current_health.unwrap_or(10) as i32;
    mnstr.max_health = max_health.unwrap_or(10) as i32;
    mnstr.current_attack = current_attack.unwrap_or(10) as i32;
    mnstr.max_attack = max_attack.unwrap_or(10) as i32;
    mnstr.current_defense = current_defense.unwrap_or(10) as i32;
    mnstr.max_defense = max_defense.unwrap_or(10) as i32;
    mnstr.current_speed = current_speed.unwrap_or(10) as i32;
    mnstr.max_speed = max_speed.unwrap_or(10) as i32;
    mnstr.current_intelligence = current_intelligence.unwrap_or(10) as i32;
    mnstr.max_intelligence = max_intelligence.unwrap_or(10) as i32;
    mnstr.current_magic = current_magic.unwrap_or(10) as i32;
    mnstr.max_magic = max_magic.unwrap_or(10) as i32;

    if let Some(error) = mnstr.create().await {
        println!("[create] Failed to create mnstr: {:?}", error);
        return Err(FieldError::from("Failed to create mnstr"));
    }

    Ok(mnstr)
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

    let mut mnstr = Mnstr::find_one(id, false)
        .await
        .map_err(|e| FieldError::from(e.to_string()))?;

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
