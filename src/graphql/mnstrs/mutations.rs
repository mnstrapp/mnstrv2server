use juniper::FieldError;

use crate::{graphql::Ctx, models::mnstr::Mnstr};

pub struct MnstrMutationType;

#[juniper::graphql_object]
impl MnstrMutationType {
    async fn collect(
        ctx: &Ctx,
        qr_code: String,
        name: String,
        current_health: i32,
        max_health: i32,
        current_attack: i32,
        max_attack: i32,
        current_defense: i32,
        max_defense: i32,
        current_speed: i32,
        max_speed: i32,
        current_intelligence: i32,
        max_intelligence: i32,
        current_magic: i32,
        max_magic: i32,
    ) -> Result<Mnstr, FieldError> {
        collect(
            ctx,
            qr_code,
            name,
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

pub async fn collect(
    ctx: &Ctx,
    qr_code: String,
    name: String,
    current_health: i32,
    max_health: i32,
    current_attack: i32,
    max_attack: i32,
    current_defense: i32,
    max_defense: i32,
    current_speed: i32,
    max_speed: i32,
    current_intelligence: i32,
    max_intelligence: i32,
    current_magic: i32,
    max_magic: i32,
) -> Result<Mnstr, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let session = ctx.session.as_ref().unwrap().clone();
    println!("[collect] qr code: {:?}", qr_code);
    println!("[collect] name: {:?}", name);
    println!("[collect] current health: {:?}", current_health);
    println!("[collect] max health: {:?}", max_health);
    println!("[collect] current attack: {:?}", current_attack);
    println!("[collect] max attack: {:?}", max_attack);
    println!("[collect] current defense: {:?}", current_defense);
    println!("[collect] max defense: {:?}", max_defense);
    println!("[collect] current speed: {:?}", current_speed);
    println!("[collect] max speed: {:?}", max_speed);
    println!("[collect] current intelligence: {:?}", current_intelligence);
    println!("[collect] max intelligence: {:?}", max_intelligence);
    println!("[collect] current magic: {:?}", current_magic);
    println!("[collect] max magic: {:?}", max_magic);

    let mut mnstr = Mnstr::new(
        session.user_id.clone(),
        name.clone(),
        String::new(),
        qr_code.clone(),
    )
    .copy_with(
        Some(name),
        None,
        Some(qr_code),
        None,
        None,
        None,
        None,
        None,
        Some(current_health),
        Some(max_health),
        Some(current_attack),
        Some(max_attack),
        Some(current_defense),
        Some(max_defense),
        Some(current_speed),
        Some(max_speed),
        Some(current_intelligence),
        Some(max_intelligence),
        Some(current_magic),
        Some(max_magic),
    );

    if let Some(error) = mnstr.create().await {
        println!("[collect] Failed to create mnstr: {:?}", error);
        return Err(FieldError::from("Failed to create mnstr"));
    }

    Ok(mnstr)
}
