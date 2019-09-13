use crate::{
    models,
    util::{database, internal_server_error, user},
};
use actix_web::{dev::ServiceRequest, web, Error, HttpMessage, HttpRequest, HttpResponse, Scope};
use actix_web_httpauth::{extractors::basic::BasicAuth, middleware::HttpAuthentication};
use futures::future::{self, Future, FutureResult, IntoFuture};
use serde_json::json;
use uuid::Uuid;

pub fn get_scope() -> Scope {
    let validator =
        |req: ServiceRequest, credentials: BasicAuth| -> FutureResult<ServiceRequest, Error> {
            let result: Result<Uuid, _> = (|| {
                let user = req
                    .app_data::<crate::AppData>()
                    .unwrap()
                    .database
                    .get_user_by_name(credentials.user_id().to_string())?;
                let password = credentials
                    .password()
                    .ok_or(BasicAuthError::WrongPwError)
                    .map(|pw| pw.clone().into_owned())?;
                if user.verify_password(password) {
                    Ok(Uuid::parse_str(&user.get_raw_id())?)
                } else {
                    Err(BasicAuthError::WrongPwError)
                }
            })();
            match result {
                Ok(user_id) => {
                    req.extensions_mut().insert(user_id);
                    future::ok(req)
                }
                Err(e) => {
                    match e {
                        BasicAuthError::WrongPwError => {}
                        BasicAuthError::UserLoadingError(e) => {
                            log::error!("Couldn't load user: {:?}", e);
                        }
                        BasicAuthError::InvalidUserID => {
                            log::error!("Invalid user id in database, here's request: {:?}", req)
                        }
                    };
                    future::err(actix_web::Error::from(()))
                }
            }
        };
    let auth = HttpAuthentication::basic(validator);
    web::scope("/account")
        .service(
            web::resource("")
                .wrap(auth.clone())
                .route(web::get().to_async(get_account))
                .route(web::put().to_async(update_account))
                .route(web::delete().to_async(delete_account)),
        )
        .service(web::resource("/register").route(web::post().to_async(create_account)))
        .service(
            web::resource("/login")
                .wrap(auth)
                .route(web::post().to_async(login)),
        )
        .service(web::resource("/housekeeping").route(web::post().to_async(clean_up_account)))
}

enum BasicAuthError {
    UserLoadingError(crate::database::DatabaseError),
    InvalidUserID,
    WrongPwError,
}

impl From<crate::database::DatabaseError> for BasicAuthError {
    fn from(err: crate::database::DatabaseError) -> BasicAuthError {
        BasicAuthError::UserLoadingError(err)
    }
}

impl From<uuid::parser::ParseError> for BasicAuthError {
    fn from(_: uuid::parser::ParseError) -> BasicAuthError {
        BasicAuthError::InvalidUserID
    }
}

fn get_account(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).get_user_by_id(*req.extensions().get::<Uuid>().unwrap()) {
        Ok(result) => {
            Box::new(Ok(HttpResponse::Ok().json(result.returnable_userdata())).into_future())
        }
        Err(error) => match error {
            crate::database::DatabaseError::Diesel(diesel::result::Error::NotFound) => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            error => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't load user: {:?}",
                    error
                )))
                .into_future(),
            ),
        },
    }
}
fn update_account(
    req: HttpRequest,
    json: web::Json<models::Account>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).update_account(*req.extensions().get::<Uuid>().unwrap(), json.into_inner())
    {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(error) => match error {
            crate::database::DatabaseError::Diesel(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) => Box::new(
                Ok(HttpResponse::BadRequest().body("Username already taken")).into_future(),
            ),
            error => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't update user: {:?}",
                    error
                )))
                .into_future(),
            ),
        },
    }
}
fn create_account(
    req: HttpRequest,
    json: web::Json<models::Account>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).create_account(json.into_inner()) {
        Ok(_) => Box::new(Ok(HttpResponse::Created().finish()).into_future()),
        Err(error) => match error {
            crate::database::DatabaseError::Diesel(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) => Box::new(
                Ok(HttpResponse::BadRequest().body("Username already taken")).into_future(),
            ),
            error => Box::new(
                Ok(internal_server_error(format!(
                    "Couldn't create user: {:?}",
                    error
                )))
                .into_future(),
            ),
        },
    }
}
fn delete_account(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let user = *req.extensions().get::<Uuid>().unwrap();
    match database(&req).delete_account(user) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(error) => Box::new(
            Ok(internal_server_error(format!(
                "Couldn't delete user: {:?}",
                error
            )))
            .into_future(),
        ),
    }
}
fn login(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &crate::AppData = req.app_data().unwrap();
    let extensions = req.extensions();
    let user = extensions.get::<Uuid>().unwrap();

    match frank_jwt::encode(
        json!({}),
        &appdata.settings.jwt_key,
        &json!({ "sub": user }),
        frank_jwt::Algorithm::HS512,
    ) {
        Ok(token) => Box::new(Ok(HttpResponse::Ok().json(json!({ "token": token }))).into_future()),
        Err(e) => {
            log::error!("Couldn't encode JWT: {:?}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
fn clean_up_account(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    match database(&req).delete_stale_objects(user(&req)) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => {
            log::error!("Couldn't clean up stale objects: {:?}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
