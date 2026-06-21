use crate::{
    models::user::User,
    proto::{MyUserRequest, MyUserResponse, user_service_server::UserService},
    services::helpers::get_user_from_token,
};

use tonic::{Request, Response, Status};

#[derive(Debug, Default, Clone)]
pub struct UserServiceImpl;

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    async fn my_user(
        &self,
        request: Request<MyUserRequest>,
    ) -> Result<Response<MyUserResponse>, Status> {
        let request = request.into_inner();
        let user = match get_user_from_token(request.token.clone()).await {
            Ok(user) => user,
            Err(e) => {
                println!("[UserServiceImpl::my_user] Failed to get user: {:?}", e);
                return Err(Status::not_found("User not found"));
            }
        };
        let user = match User::find_one(user.id.clone(), true).await {
            Ok(user) => user,
            Err(e) => {
                println!("[UserServiceImpl::my_user] Failed to get user: {:?}", e);
                return Err(Status::not_found("User not found"));
            }
        };
        Ok(Response::new(MyUserResponse {
            user: Some(user.to_grpc()),
        }))
    }
}
