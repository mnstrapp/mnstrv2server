use juniper::FieldError;

use crate::{graphql::Ctx, models::mnstr::Mnstr};
pub struct MnstrQueryType;

#[juniper::graphql_object]
impl MnstrQueryType {
    async fn list(ctx: &Ctx) -> Result<Vec<Mnstr>, FieldError> {
        list(ctx).await
    }

    async fn qr_code(ctx: &Ctx, qr_code: String) -> Result<Option<Mnstr>, FieldError> {
        by_qr_code(ctx, qr_code).await
    }
}

async fn list(ctx: &Ctx) -> Result<Vec<Mnstr>, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let session = ctx.session.as_ref().unwrap().clone();

    match Mnstr::find_all_by(vec![("user_id", session.user_id.clone().into())], false).await {
        Ok(mnstrs) => Ok(mnstrs),
        Err(e) => {
            println!("[mnstrs] Failed to get mnstrs: {:?}", e);
            return Ok(vec![]);
        }
    }
}

async fn by_qr_code(ctx: &Ctx, qr_code: String) -> Result<Option<Mnstr>, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let session = ctx.session.as_ref().unwrap().clone();

    let params = vec![
        ("user_id", session.user_id.clone().into()),
        ("mnstr_qr_code", qr_code.clone().into()),
    ];

    match Mnstr::find_one_by(params, false).await {
        Ok(mnstr) => Ok(Some(mnstr)),
        Err(e) => {
            println!("[get_by_qr_code] Failed to get mnstr: {:?}", e);
            return Ok(None);
        }
    }
}
