use juniper::FieldError;

use crate::{graphql::Ctx, models::mnstr::{Mnstr, MnstrOrderBy, MnstrOrderDirection}};

pub type MnstrOrderByInput = MnstrOrderBy;
pub type MnstrOrderDirectionInput = MnstrOrderDirection;

pub struct MnstrQueryType;

#[juniper::graphql_object]
impl MnstrQueryType {
    async fn list(
        ctx: &Ctx,
        order_by: Option<MnstrOrderByInput>,
        order_direction: Option<MnstrOrderDirectionInput>,
    ) -> Result<Vec<Mnstr>, FieldError> {
        list(ctx, order_by, order_direction).await
    }

    async fn qr_code(ctx: &Ctx, mnstr_qr_code: String) -> Result<Option<Mnstr>, FieldError> {
        by_qr_code(ctx, mnstr_qr_code).await
    }
}

async fn list(
    ctx: &Ctx,
    order_by: Option<MnstrOrderByInput>,
    order_direction: Option<MnstrOrderDirectionInput>,
) -> Result<Vec<Mnstr>, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let session = ctx.session.as_ref().unwrap().clone();

    let params = vec![("user_id", session.user_id.clone().into())];

    println!(
        "[mnstrs] Order by: {:?}",
        order_by.clone().unwrap().to_string()
    );
    println!(
        "[mnstrs] Order direction: {:?}",
        order_direction.clone().unwrap().to_string()
    );

    match Mnstr::find_all_by(params, false, order_by, order_direction).await {
        Ok(mnstrs) => Ok(mnstrs),
        Err(e) => {
            println!("[mnstrs] Failed to get mnstrs: {:?}", e);
            return Ok(vec![]);
        }
    }
}

async fn by_qr_code(ctx: &Ctx, mnstr_qr_code: String) -> Result<Option<Mnstr>, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let session = ctx.session.as_ref().unwrap().clone();

    let params = vec![
        ("user_id", session.user_id.clone().into()),
        ("mnstr_qr_code", mnstr_qr_code.clone().into()),
    ];

    match Mnstr::find_one_by(params, false).await {
        Ok(mnstr) => Ok(Some(mnstr)),
        Err(e) => {
            println!("[get_by_qr_code] Failed to get mnstr: {:?}", e);
            return Ok(None);
        }
    }
}
