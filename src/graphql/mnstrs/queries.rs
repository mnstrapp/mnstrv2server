use juniper::FieldError;

use crate::{graphql::Ctx, models::mnstr::Mnstr};
pub struct MnstrQueryType;

#[juniper::graphql_object]
impl MnstrQueryType {
    async fn mnstrs(ctx: &Ctx) -> Result<Vec<Mnstr>, FieldError> {
        mnstrs(ctx).await
    }
}

async fn mnstrs(ctx: &Ctx) -> Result<Vec<Mnstr>, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let session = ctx.session.as_ref().unwrap().clone();

    match Mnstr::find_all_by(vec![("user_id", session.user_id.clone().into())]).await {
        Ok(mnstrs) => Ok(mnstrs),
        Err(e) => {
            println!("[mnstrs] Failed to get mnstrs: {:?}", e);
            return Ok(vec![]);
        }
    }
}
