use tonic::{Request, Response, Status};

use crate::{
    database::values::DatabaseValue,
    models::mnstr::{DEFAULT_STAT_VALUE, Mnstr, MnstrOrderBy, MnstrOrderDirection},
    proto::{
        CollectMnstrRequest, CollectMnstrResponse, CreateMnstrBatchRequest,
        CreateMnstrBatchResponse, CreateMnstrRequest, CreateMnstrResponse, GetMnstrByQrCodeRequest,
        GetMnstrByQrCodeResponse, ListMnstrsRequest, ListMnstrsResponse, Mnstr as GrpcMnstr, MnstrInput,
        MnstrOrderBy as GrpcMnstrOrderBy, MnstrOrderDirection as GrpcMnstrOrderDirection, UpdateMnstrBatchRequest, UpdateMnstrBatchResponse,
        UpdateMnstrRequest, UpdateMnstrResponse, mnstr_service_server::MnstrService,
    },
    services::helpers::get_user_from_token,
};

#[derive(Debug, Default, Clone)]
pub struct MnstrServiceImpl;

#[tonic::async_trait]
impl MnstrService for MnstrServiceImpl {
    async fn get_by_qr_code(
        &self,
        request: Request<GetMnstrByQrCodeRequest>,
    ) -> Result<Response<GetMnstrByQrCodeResponse>, Status> {
        let request = request.into_inner();

        let user = match get_user_from_token(request.token.clone()).await {
            Ok(user) => user,
            Err(e) => {
                println!(
                    "[MnstrServiceImpl::GetByQrCode] Failed to get user: {:?}",
                    e
                );
                return Err(Status::from_error(e.into()));
            }
        };

        let mnstr = match Mnstr::find_one_by(
            vec![
                ("user_id", user.id.clone().into()),
                ("mnstr_qr_code", request.mnstr_qr_code.into()),
            ],
            false,
        )
        .await
        {
            Ok(mnstr) => mnstr,
            Err(e) => {
                println!(
                    "[MnstrServiceImpl::GetByQrCode] Failed to get mnstr: {:?}",
                    e
                );
                return Err(Status::from_error(e.into()));
            }
        };

        Ok(Response::new(GetMnstrByQrCodeResponse {
            mnstr: Some(mnstr.to_grpc()),
        }))
    }

    async fn collect(
        &self,
        request: Request<CollectMnstrRequest>,
    ) -> Result<Response<CollectMnstrResponse>, Status> {
        let request = request.into_inner();

        let user = match get_user_from_token(request.token.clone()).await {
            Ok(user) => user,
            Err(e) => {
                println!("[MnstrServiceImpl::Collect] Failed to get user: {:?}", e);
                return Err(Status::from_error(e.into()));
            }
        };

        let mnstr = match Mnstr::find_one_by(
            vec![
                ("user_id", user.id.clone().into()),
                ("mnstr_qr_code", request.mnstr_qr_code.into()),
            ],
            false,
        )
        .await
        {
            Ok(mnstr) => mnstr,
            Err(e) => {
                println!("[MnstrServiceImpl::Collect] Failed to get mnstr: {:?}", e);
                return Err(Status::from_error(e.into()));
            }
        };

        Ok(Response::new(CollectMnstrResponse {
            mnstr: Some(mnstr.to_grpc()),
        }))
    }

    async fn create(
        &self,
        request: Request<CreateMnstrRequest>,
    ) -> Result<Response<CreateMnstrResponse>, Status> {
        let request = request.into_inner();

        let user = match get_user_from_token(request.token).await {
            Ok(user) => user,
            Err(e) => {
                println!("[MnstrServiceImpl::Create] Failed to get user: {:?}", e);
                return Err(Status::from_error(e.into()));
            }
        };

        let mut mnstr = Mnstr::new(
            user.id,
            request.mnstr_name,
            request.mnstr_description,
            request.mnstr_qr_code,
        );

        mnstr.current_health = request.current_health.unwrap_or(DEFAULT_STAT_VALUE);
        mnstr.max_health = request.max_health.unwrap_or(DEFAULT_STAT_VALUE);
        mnstr.current_attack = request.current_attack.unwrap_or(DEFAULT_STAT_VALUE);
        mnstr.max_attack = request.max_attack.unwrap_or(DEFAULT_STAT_VALUE);
        mnstr.current_defense = request.current_defense.unwrap_or(DEFAULT_STAT_VALUE);
        mnstr.max_defense = request.max_defense.unwrap_or(DEFAULT_STAT_VALUE);
        mnstr.current_speed = request.current_speed.unwrap_or(DEFAULT_STAT_VALUE);
        mnstr.max_speed = request.max_speed.unwrap_or(DEFAULT_STAT_VALUE);
        mnstr.current_intelligence = request.current_intelligence.unwrap_or(DEFAULT_STAT_VALUE);
        mnstr.max_intelligence = request.max_intelligence.unwrap_or(DEFAULT_STAT_VALUE);
        mnstr.current_magic = request.current_magic.unwrap_or(DEFAULT_STAT_VALUE);
        mnstr.max_magic = request.max_magic.unwrap_or(DEFAULT_STAT_VALUE);

        let mnstr = match mnstr.create().await {
            Some(error) => {
                println!(
                    "[MnstrServiceImpl::Create] Failed to create mnstr: {:?}",
                    error
                );
                return Err(Status::from_error(error.into()));
            }
            None => mnstr,
        };

        Ok(Response::new(CreateMnstrResponse {
            mnstr: Some(mnstr.to_grpc()),
        }))
    }

    async fn create_batch(
        &self,
        request: Request<CreateMnstrBatchRequest>,
    ) -> Result<Response<CreateMnstrBatchResponse>, Status> {
        let request = request.into_inner();

        let user = match get_user_from_token(request.token).await {
            Ok(user) => user,
            Err(e) => {
                println!("[MnstrServiceImpl::Create] Failed to get user: {:?}", e);
                return Err(Status::from_error(e.into()));
            }
        };

        let mnstrs = request
            .mnstrs
            .map_or(vec![], |batch_mnstr_input| {
                batch_mnstr_input
                    .mnstrs
                    .into_iter()
                    .collect::<Vec<MnstrInput>>()
            })
            .iter()
            .filter(|mnstr| mnstr.mnstr_qr_code.is_some())
            .map(|mnstr| {
                let mut mnstr_params: Vec<(&str, Option<DatabaseValue>)> = Vec::new();
                mnstr_params.push(("user_id", Some(user.id.clone().into())));
                mnstr_params.push(("mnstr_name", mnstr.mnstr_name.as_ref().map(|s| s.into())));
                mnstr_params.push((
                    "mnstr_description",
                    mnstr.mnstr_description.as_ref().map(|s| s.into()),
                ));
                mnstr_params.push((
                    "mnstr_qr_code",
                    mnstr.mnstr_qr_code.as_ref().map(|s| s.into()),
                ));
                mnstr_params.push((
                    "current_health",
                    mnstr
                        .current_health
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "max_health",
                    mnstr
                        .max_health
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "current_attack",
                    mnstr
                        .current_attack
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "max_attack",
                    mnstr
                        .max_attack
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "current_defense",
                    mnstr
                        .current_defense
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "max_defense",
                    mnstr
                        .max_defense
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "current_speed",
                    mnstr
                        .current_speed
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "max_speed",
                    mnstr
                        .max_speed
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "current_intelligence",
                    mnstr
                        .current_intelligence
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "max_intelligence",
                    mnstr
                        .max_intelligence
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "current_magic",
                    mnstr
                        .current_magic
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "max_magic",
                    mnstr
                        .max_magic
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params
            })
            .collect::<Vec<Vec<(&str, Option<DatabaseValue>)>>>();

        let mnstrs = match Mnstr::create_batch(user.id.clone(), mnstrs).await {
            Ok(mnstrs) => mnstrs,
            Err(e) => {
                println!(
                    "[MnstrServiceImpl::CreateBatch] Failed to create mnstrs: {:?}",
                    e
                );
                return Err(Status::from_error(e.into()));
            }
        };

        Ok(Response::new(CreateMnstrBatchResponse {
            mnstrs: mnstrs.into_iter().map(|mnstr| mnstr.to_grpc()).collect(),
        }))
    }

    async fn update(
        &self,
        request: Request<UpdateMnstrRequest>,
    ) -> Result<Response<UpdateMnstrResponse>, Status> {
        let request = request.into_inner();

        if let Err(e) = get_user_from_token(request.token).await {
            println!("[MnstrServiceImpl::Update] Failed to get user: {:?}", e);
            return Err(Status::from_error(e.into()));
        };

        let mut mnstr = match Mnstr::find_one(request.id, false).await {
            Ok(mnstr) => mnstr,
            Err(e) => {
                println!("[MnstrServiceImpl::Update] Failed to find mnstr: {:?}", e);
                return Err(Status::from_error(e.into()));
            }
        };

        mnstr.mnstr_name = request.mnstr_name.unwrap_or(mnstr.mnstr_name);
        mnstr.mnstr_description = request.mnstr_description.unwrap_or(mnstr.mnstr_description);
        mnstr.current_health = request.current_health.unwrap_or(mnstr.current_health);
        mnstr.max_health = request.max_health.unwrap_or(mnstr.max_health);
        mnstr.current_attack = request.current_attack.unwrap_or(mnstr.current_attack);
        mnstr.max_attack = request.max_attack.unwrap_or(mnstr.max_attack);
        mnstr.current_defense = request.current_defense.unwrap_or(mnstr.current_defense);
        mnstr.max_defense = request.max_defense.unwrap_or(mnstr.max_defense);
        mnstr.current_speed = request.current_speed.unwrap_or(mnstr.current_speed);
        mnstr.max_speed = request.max_speed.unwrap_or(mnstr.max_speed);
        mnstr.current_intelligence = request
            .current_intelligence
            .unwrap_or(mnstr.current_intelligence);
        mnstr.max_intelligence = request.max_intelligence.unwrap_or(mnstr.max_intelligence);
        mnstr.current_magic = request.current_magic.unwrap_or(mnstr.current_magic);
        mnstr.max_magic = request.max_magic.unwrap_or(mnstr.max_magic);

        let mnstr = match mnstr.update().await {
            Some(error) => {
                println!(
                    "[MnstrServiceImpl::Update] Failed to update mnstr: {:?}",
                    error
                );
                return Err(Status::from_error(error.into()));
            }
            None => mnstr,
        };

        Ok(Response::new(UpdateMnstrResponse {
            mnstr: Some(mnstr.to_grpc()),
        }))
    }

    async fn update_batch(
        &self,
        request: Request<UpdateMnstrBatchRequest>,
    ) -> Result<Response<UpdateMnstrBatchResponse>, Status> {
        let request = request.into_inner();

        let user = match get_user_from_token(request.token).await {
            Ok(user) => user,
            Err(e) => {
                println!(
                    "[MnstrServiceImpl::UpdateBatch] Failed to get user: {:?}",
                    e
                );
                return Err(Status::from_error(e.into()));
            }
        };

        let mnstrs = request
            .mnstrs
            .map_or(vec![], |batch_mnstr_input| {
                batch_mnstr_input
                    .mnstrs
                    .into_iter()
                    .collect::<Vec<MnstrInput>>()
            })
            .iter()
            .filter(|mnstr| mnstr.mnstr_qr_code.is_some())
            .map(|mnstr| {
                let mut mnstr_params: Vec<(&str, Option<DatabaseValue>)> = Vec::new();
                mnstr_params.push(("user_id", Some(user.id.clone().into())));
                mnstr_params.push(("id", mnstr.id.as_ref().map(|s| s.into())));
                mnstr_params.push(("mnstr_name", mnstr.mnstr_name.as_ref().map(|s| s.into())));
                mnstr_params.push((
                    "mnstr_description",
                    mnstr.mnstr_description.as_ref().map(|s| s.into()),
                ));
                mnstr_params.push((
                    "current_health",
                    mnstr
                        .current_health
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "max_health",
                    mnstr
                        .max_health
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "current_attack",
                    mnstr
                        .current_attack
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "max_attack",
                    mnstr
                        .max_attack
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "current_defense",
                    mnstr
                        .current_defense
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "max_defense",
                    mnstr
                        .max_defense
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "current_speed",
                    mnstr
                        .current_speed
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "max_speed",
                    mnstr
                        .max_speed
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "current_intelligence",
                    mnstr
                        .current_intelligence
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "max_intelligence",
                    mnstr
                        .max_intelligence
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "current_magic",
                    mnstr
                        .current_magic
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params.push((
                    "max_magic",
                    mnstr
                        .max_magic
                        .map_or(Some(DEFAULT_STAT_VALUE.into()), |i| Some(i.into())),
                ));
                mnstr_params
            })
            .collect::<Vec<Vec<(&str, Option<DatabaseValue>)>>>();

        let mnstrs = match Mnstr::update_batch(user.id.clone(), mnstrs).await {
            Ok(mnstrs) => mnstrs,
            Err(e) => {
                println!(
                    "[MnstrServiceImpl::UpdateBatch] Failed to update mnstrs: {:?}",
                    e
                );
                return Err(Status::from_error(e.into()));
            }
        };

        Ok(Response::new(UpdateMnstrBatchResponse {
            mnstrs: mnstrs.into_iter().map(|mnstr| mnstr.to_grpc()).collect(),
        }))
    }

    async fn list(
        &self,
        request: Request<ListMnstrsRequest>,
    ) -> Result<Response<ListMnstrsResponse>, Status> {
        let request = request.into_inner();

        let user = match get_user_from_token(request.token).await {
            Ok(user) => user,
            Err(e) => {
                println!("[MnstrServiceImpl::List] Failed to get user: {:?}", e);
                return Err(Status::from_error(e.into()));
            }
        };

        let order_by = request.order_by.unwrap_or(GrpcMnstrOrderBy::default().into());
        let order_direction = request
            .order_direction
            .unwrap_or(GrpcMnstrOrderDirection::default().into());

        let mnstrs = match Mnstr::find_all_by(
            vec![("user_id", user.id.clone().into())],
            false,
            Some(MnstrOrderBy::from_grpc(order_by)),
            Some(MnstrOrderDirection::from_grpc(order_direction)),
        )
        .await
        {
            Ok(mnstrs) => mnstrs,
            Err(e) => {
                println!("[MnstrServiceImpl::List] Failed to find mnstrs: {:?}", e);
                return Err(Status::from_error(e.into()));
            }
        };

        Ok(Response::new(ListMnstrsResponse {
            mnstrs: mnstrs
                .into_iter()
                .map(|mnstr| mnstr.to_grpc())
                .collect::<Vec<GrpcMnstr>>(),
        }))
    }
}
