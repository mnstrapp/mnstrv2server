use juniper::{FieldError, GraphQLEnum};

use crate::{graphql::Ctx, models::mnstr::Mnstr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GraphQLEnum)]
pub enum MnstrOrderByInput {
    CreatedAt,
    UpdatedAt,
    Name,
    Level,
    Experience,
    Health,
    Attack,
    Defense,
    Speed,
    Intelligence,
    Magic,
}

impl MnstrOrderByInput {
    pub fn to_string(&self) -> String {
        match self {
            MnstrOrderByInput::CreatedAt => "created_at".to_string(),
            MnstrOrderByInput::UpdatedAt => "updated_at".to_string(),
            MnstrOrderByInput::Name => "mnstr_name".to_string(),
            MnstrOrderByInput::Level => "current_level".to_string(),
            MnstrOrderByInput::Experience => "current_experience".to_string(),
            MnstrOrderByInput::Health => "max_health".to_string(),
            MnstrOrderByInput::Attack => "max_attack".to_string(),
            MnstrOrderByInput::Defense => "max_defense".to_string(),
            MnstrOrderByInput::Speed => "max_speed".to_string(),
            MnstrOrderByInput::Intelligence => "max_intelligence".to_string(),
            MnstrOrderByInput::Magic => "max_magic".to_string(),
        }
    }
}

impl Default for MnstrOrderByInput {
    fn default() -> Self {
        MnstrOrderByInput::CreatedAt
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GraphQLEnum)]
pub enum MnstrOrderDirectionInput {
    Asc,
    Desc,
}

impl MnstrOrderDirectionInput {
    pub fn to_string(&self) -> String {
        match self {
            MnstrOrderDirectionInput::Asc => "asc".to_string(),
            MnstrOrderDirectionInput::Desc => "desc".to_string(),
        }
    }
}

impl Default for MnstrOrderDirectionInput {
    fn default() -> Self {
        MnstrOrderDirectionInput::Asc
    }
}
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
