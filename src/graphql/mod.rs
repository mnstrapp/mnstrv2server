use futures::stream;
use juniper::{Context, FieldError, RootNode, graphql_object, graphql_subscription};
use juniper_rocket::{GraphQLRequest, GraphQLResponse};
use rocket::{Route, get, post, response::content::RawHtml};

use crate::{
    graphql::{
        sessions::{SessionMutationType, SessionQueryType},
        users::{mutations::UserMutationType, queries::UserQueryType},
    },
    models::session::Session,
    utils::{sessions::validate_session, token::RawToken},
};

pub mod mnstrs;
pub mod sessions;
pub mod users;

pub fn routes() -> Vec<Route> {
    routes![graphiql, graphql]
}

pub struct Ctx {
    pub session: Option<Session>,
}

impl Context for Ctx {}

pub struct Query;

#[graphql_object(context = Ctx)]
impl Query {
    async fn session() -> SessionQueryType {
        SessionQueryType
    }

    pub async fn users() -> UserQueryType {
        UserQueryType
    }
}

pub struct Mutation;

#[graphql_object(context = Ctx)]
impl Mutation {
    pub async fn session() -> SessionMutationType {
        SessionMutationType
    }

    pub async fn users() -> UserMutationType {
        UserMutationType
    }
}

pub struct Subscription;

#[graphql_subscription(context = Ctx)]
impl Subscription {
    async fn hello(_ctx: &Ctx) -> std::pin::Pin<Box<dyn futures::Stream<Item = String> + Send>> {
        Box::pin(stream::once(async { "Hello, world!".to_string() }))
    }
}

pub type Schema = RootNode<'static, Query, Mutation, Subscription>;

#[get("/graphiql")]
pub fn graphiql() -> RawHtml<String> {
    juniper_rocket::graphiql_source("/graphql", None)
}

#[post("/", data = "<request>")]
pub async fn graphql(request: GraphQLRequest, token: RawToken) -> GraphQLResponse {
    let mut ctx = Ctx { session: None };
    if !token.value.is_empty() {
        let session = match verify_session_token(token).await {
            Ok(session) => session,
            Err(_) => {
                return GraphQLResponse::error(FieldError::new(
                    "Invalid session",
                    juniper::Value::Null,
                ));
            }
        };
        ctx.session = Some(session);
    }
    let schema = Schema::new(Query, Mutation, Subscription);

    request.execute(&schema, &ctx).await
}

async fn verify_session_token(token: RawToken) -> Result<Session, FieldError> {
    let mut session = match Session::find_one_by_token(token.value).await {
        Ok(session) => session,
        Err(e) => return Err(e.into()),
    };
    if validate_session(&mut session).await.is_some() {
        return Err(FieldError::from("Invalid session"));
    }
    Ok(session)
}
