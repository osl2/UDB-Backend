use crate::{models, schema, database::DatabaseConnection};
use actix_web::{dev::ServiceRequest, web, Error, HttpMessage, HttpRequest, HttpResponse, Scope};
use actix_web_httpauth::{extractors::basic::BasicAuth, middleware::HttpAuthentication};
use diesel::{
    r2d2::{self, ConnectionManager},
    Connection, ExpressionMethods, QueryDsl, RunQueryDsl,
};
use futures::future::{self, Future, FutureResult, IntoFuture};
use serde_json::json;
use uuid::Uuid;

pub fn get_scope() -> Scope {
    let validator =
        |req: ServiceRequest, credentials: BasicAuth| -> FutureResult<ServiceRequest, Error> {
            let result: Result<Uuid, _> = (|| {
                let extensions = req.extensions();
                let conn = extensions
                    .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
                    .unwrap();
                let user = schema::users::table
                    .filter(schema::users::name.eq(credentials.user_id()))
                    .get_result::<models::User>(&*conn)?;
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
                        BasicAuthError::WrongPwError | BasicAuthError::UserLoadingError(_) => {}
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
}

enum BasicAuthError {
    UserLoadingError(diesel::result::Error),
    InvalidUserID,
    WrongPwError,
}

impl From<diesel::result::Error> for BasicAuthError {
    fn from(err: diesel::result::Error) -> BasicAuthError {
        BasicAuthError::UserLoadingError(err)
    }
}

impl From<uuid::parser::ParseError> for BasicAuthError {
    fn from(_: uuid::parser::ParseError) -> BasicAuthError {
        BasicAuthError::InvalidUserID
    }
}

fn get_account(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();
    let user = extensions.get::<Uuid>().unwrap();

    match schema::users::table
        .find(format!("{}", user))
        .get_result::<models::User>(&*conn)
    {
        Ok(result) => {
            Box::new(Ok(HttpResponse::Ok().json(result.returnable_userdata())).into_future())
        }
        Err(e) => match e {
            diesel::result::Error::NotFound => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            _ => Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future()),
        },
    }
}
fn update_account(
    req: HttpRequest,
    json: web::Json<models::Account>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();
    let user = extensions.get::<Uuid>().unwrap();
    let account_template = json.into_inner();
    match conn.transaction::<(), diesel::result::Error, _>(|| {
        diesel::update(schema::users::table.find(format!("{}", user)))
            .set(models::User::new(
                account_template.username,
                account_template.password,
                Some(*user),
            ))
            .execute(&*conn)?;
        Ok(())
    }) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(_) => {
            Box::new(Ok(HttpResponse::BadRequest().body("Username already taken")).into_future())
        }
    }
}
fn create_account(
    req: HttpRequest,
    json: web::Json<models::Account>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();
    let account_template = json.into_inner();
    match conn.transaction::<(), diesel::result::Error, _>(|| {
        diesel::insert_into(schema::users::table)
            .values(models::User::new(
                account_template.username,
                account_template.password,
                None,
            ))
            .execute(&*conn)?;
        Ok(())
    }) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(_) => {
            Box::new(Ok(HttpResponse::BadRequest().body("Username already taken")).into_future())
        }
    }
}
fn delete_account(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().body(format!("{:?}", req))).into_future())
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
            log::error!("Couldn't encode JWT: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
