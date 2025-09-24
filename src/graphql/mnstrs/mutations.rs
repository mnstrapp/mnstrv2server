use juniper::FieldError;

use crate::{graphql::Ctx, models::mnstr::Mnstr};

pub struct MnstrMutationType;

#[juniper::graphql_object]
impl MnstrMutationType {
    async fn collect(ctx: &Ctx, qr_code: String) -> Result<Mnstr, FieldError> {
        collect(ctx, qr_code).await
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
