use crate::{
    models::{session::Session, user::User},
    proto::{
        ForgotPasswordRequest, ForgotPasswordResponse, LoginRequest, LoginResponse, LogoutRequest,
        LogoutResponse, RegisterRequest, RegisterResponse, ResetPasswordRequest,
        ResetPasswordResponse, VerifyEmailRequest, VerifyEmailResponse, VerifyPhoneRequest, VerifyPhoneResponse,
        session_service_server::SessionService,
    },
    utils::{
        emails::send_email_verification_code,
        passwords::{generate_verification_code, hash_password},
    },
};

use tonic::{Request, Response, Status};

#[derive(Debug, Default, Clone)]
pub struct SessionServiceImpl;

#[tonic::async_trait]
impl SessionService for SessionServiceImpl {
    async fn register(
        &self,
        _request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let request = _request.into_inner();

        let email = request.email;
        if email.clone().is_empty() {
            return Err(Status::invalid_argument("Email is required"));
        }

        let password = request.password;
        if password.clone().is_empty() {
            return Err(Status::invalid_argument("Password is required"));
        }

        let code = generate_verification_code();

        let mut user = User::new(
            Some(email.clone()),
            request.phone,
            password.clone(),
            request.display_name.clone(),
        );
        user.email_verification_code = Some(code.clone());
        if let Some(error) = user.create().await {
            return Err(Status::internal(error.to_string()));
        }

        if let Err(error) = send_email_verification_code(
            request.display_name.as_str(),
            email.as_str(),
            code.as_str(),
        )
        .await
        {
            println!(
                "[SessionServiceImpl::register] Failed to send email verification code: {:?}",
                error
            );
            return Err(Status::internal(error.to_string()));
        }

        Ok(Response::new(RegisterResponse { success: true }))
    }

    async fn login(
        &self,
        _request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let request = _request.into_inner();
        let email = request.email;
        if email.clone().is_empty() {
            return Err(Status::invalid_argument("Email is required"));
        }

        let password = request.password;
        if password.clone().is_empty() {
            return Err(Status::invalid_argument("Password is required"));
        }

        let user = User::new(Some(email.clone()), None, password.clone(), "".to_string());

        let params = vec![
            ("email", user.email.clone().unwrap().into()),
            ("password_hash", user.password_hash.clone().into()),
        ];
        let user = match User::find_one_by(params, false).await {
            Ok(user) => user,
            Err(e) => {
                println!(
                    "[SessionServiceImpl::login] Failed to get user by email: {:?}",
                    e
                );
                return Err(Status::not_found("Unable to login"));
            }
        };
        let mut session = Session::new(user.id.clone());
        if let Some(error) = session.create().await {
            println!(
                "[SessionServiceImpl::login] Failed to create session: {:?}",
                error
            );
            return Err(Status::internal(error.to_string()));
        }

        Ok(Response::new(LoginResponse {
            session: Some(session.to_grpc()),
        }))
    }

    async fn logout(
        &self,
        _request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        let request = _request.into_inner();
        let token = request.token;
        if token.clone().is_empty() {
            return Err(Status::invalid_argument("Token is required"));
        }

        let mut session = match Session::find_one_by_token(token.clone()).await {
            Ok(session) => session,
            Err(e) => {
                println!(
                    "[SessionServiceImpl::logout] Failed to get session: {:?}",
                    e
                );
                return Err(Status::not_found("Unable to logout"));
            }
        };
        if let Some(error) = session.delete().await {
            println!(
                "[SessionServiceImpl::logout] Failed to delete session: {:?}",
                error
            );
            return Err(Status::internal(error.to_string()));
        }

        Ok(Response::new(LogoutResponse { success: true }))
    }

    async fn forgot_password(
        &self,
        _request: Request<ForgotPasswordRequest>,
    ) -> Result<Response<ForgotPasswordResponse>, Status> {
        let request = _request.into_inner();
        let email = request.email;
        if email.clone().is_empty() {
            return Err(Status::invalid_argument("Email is required"));
        }
        let params = vec![("email", email.clone().into())];
        let mut user = match User::find_one_by(params, false).await {
            Ok(user) => user,
            Err(e) => {
                println!(
                    "[SessionServiceImpl::forgot_password] Failed to get user: {:?}",
                    e
                );
                return Err(Status::not_found("Unable to forgot password"));
            }
        };
        let code = generate_verification_code();
        user.email_verification_code = Some(code.clone());
        if let Some(error) = user.update().await {
            println!(
                "[SessionServiceImpl::forgot_password] Failed to update user: {:?}",
                error
            );
            return Err(Status::internal(error.to_string()));
        }

        if let Err(error) = send_email_verification_code(
            user.display_name.clone().as_str(),
            user.email.clone().unwrap().as_str(),
            code.as_str(),
        )
        .await
        {
            println!(
                "[SessionServiceImpl::forgot_password] Failed to send password reset code: {:?}",
                error
            );
            return Err(Status::internal(error.to_string()));
        }

        Ok(Response::new(ForgotPasswordResponse { success: true }))
    }

    async fn reset_password(
        &self,
        _request: Request<ResetPasswordRequest>,
    ) -> Result<Response<ResetPasswordResponse>, Status> {
        let request = _request.into_inner();
        let code = request.code;
        if code.clone().is_empty() {
            return Err(Status::invalid_argument("Code is required"));
        }
        let password = request.password;
        if password.clone().is_empty() {
            return Err(Status::invalid_argument("Password is required"));
        }

        let mut user = match User::find_one_by(
            vec![("email_verification_code", code.clone().into())],
            false,
        )
        .await
        {
            Ok(user) => user,
            Err(e) => {
                println!(
                    "[SessionServiceImpl::reset_password] Failed to get user: {:?}",
                    e
                );
                return Err(Status::not_found("Unable to reset password"));
            }
        };

        user.password_hash = hash_password(&password.clone());
        user.email_verification_code = None;
        user.email_verified = true;
        if let Some(error) = user.update().await {
            println!(
                "[SessionServiceImpl::reset_password] Failed to update user: {:?}",
                error
            );
            return Err(Status::internal(error.to_string()));
        }
        Ok(Response::new(ResetPasswordResponse { success: true }))
    }

    async fn verify_email(
        &self,
        _request: Request<VerifyEmailRequest>,
    ) -> Result<Response<VerifyEmailResponse>, Status> {
        let request = _request.into_inner();
        let code = request.code;
        if code.clone().is_empty() {
            return Err(Status::invalid_argument("Code is required"));
        }

        let mut user = match User::find_one_by(
            vec![("email_verification_code", code.clone().into())],
            false,
        )
        .await
        {
            Ok(user) => user,
            Err(e) => {
                println!(
                    "[SessionServiceImpl::verify_email] Failed to get user: {:?}",
                    e
                );
                return Err(Status::not_found("Unable to verify email"));
            }
        };

        user.email_verified = true;
        user.email_verification_code = None;
        if let Some(error) = user.update().await {
            println!(
                "[SessionServiceImpl::verify_email] Failed to update user: {:?}",
                error
            );
            return Err(Status::internal(error.to_string()));
        }

        Ok(Response::new(VerifyEmailResponse { success: true }))
    }

    async fn verify_phone(
        &self,
        _request: Request<VerifyPhoneRequest>,
    ) -> Result<Response<VerifyPhoneResponse>, Status> {
        let request = _request.into_inner();
        let code = request.code;
        if code.clone().is_empty() {
            return Err(Status::invalid_argument("Code is required"));
        }

        let mut user = match User::find_one_by(
            vec![("phone_verification_code", code.clone().into())],
            false,
        )
        .await
        {
            Ok(user) => user,
            Err(e) => {
                println!(
                    "[SessionServiceImpl::verify_phone] Failed to get user: {:?}",
                    e
                );
                return Err(Status::not_found("Unable to verify phone"));
            }
        };

        user.phone_verified = true;
        user.phone_verification_code = None;
        if let Some(error) = user.update().await {
            println!(
                "[SessionServiceImpl::verify_phone] Failed to update user: {:?}",
                error
            );
            return Err(Status::internal(error.to_string()));
        }
        Ok(Response::new(VerifyPhoneResponse { success: true }))
    }
}
