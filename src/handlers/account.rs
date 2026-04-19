use crate::{models, schema};
use actix_web::{dev::ServiceRequest, web, Error, HttpMessage, HttpRequest, HttpResponse, Responder, Scope};
use actix_web_httpauth::{extractors::basic::BasicAuth, middleware::HttpAuthentication};
use diesel::{
    r2d2::{self, ConnectionManager},
    Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection,
};
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use serde_json::json;
use std::cell::RefCell;
use uuid::Uuid;

pub fn get_scope() -> Scope {
    let validator = |req: ServiceRequest, credentials: BasicAuth| async move {
        let result: Result<Uuid, BasicAuthError> = (|| {
            let extensions = req.extensions();
            let conn_cell = extensions
                .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
                .unwrap();
            let mut conn = conn_cell.borrow_mut();
            let user = schema::users::table
                .filter(schema::users::name.eq(credentials.user_id()))
                .get_result::<models::User>(&mut *conn)?;
            let password = credentials
                .password()
                .ok_or(BasicAuthError::WrongPwError)
                .map(|pw| pw.to_string())?;
            if user.verify_password(password) {
                Ok(Uuid::parse_str(&user.get_raw_id())?)
            } else {
                Err(BasicAuthError::WrongPwError)
            }
        })();
        match result {
            Ok(user_id) => {
                req.extensions_mut().insert(user_id);
                Ok(req)
            }
            Err(e) => {
                match e {
                    BasicAuthError::WrongPwError | BasicAuthError::UserLoadingError(_) => {}
                    BasicAuthError::InvalidUserID => {
                        log::error!("Invalid user id in database, here's request: {:?}", req)
                    }
                };
                Err((actix_web::Error::from(actix_web::error::ErrorUnauthorized("unauthorized")), req))
            }
        }
    };
    let auth = HttpAuthentication::basic(validator);
    web::scope("/account")
        .service(
            web::resource("")
                .wrap(auth.clone())
                .route(web::get().to(get_account))
                .route(web::put().to(update_account))
                .route(web::delete().to(delete_account)),
        )
        .service(web::resource("/register").route(web::post().to(create_account)))
        .service(
            web::resource("/login")
                .wrap(auth)
                .route(web::post().to(login)),
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

impl From<uuid::Error> for BasicAuthError {
    fn from(_: uuid::Error) -> BasicAuthError {
        BasicAuthError::InvalidUserID
    }
}

async fn get_account(req: HttpRequest) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();
    let user = extensions.get::<Uuid>().unwrap();

    match schema::users::table
        .find(format!("{}", user))
        .get_result::<models::User>(&mut *conn)
    {
        Ok(result) => HttpResponse::Ok().json(result.returnable_userdata()),
        Err(e) => match e {
            diesel::result::Error::NotFound => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        },
    }
}

async fn update_account(
    req: HttpRequest,
    json: web::Json<models::Account>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();
    let user = extensions.get::<Uuid>().unwrap();
    let account_template = json.into_inner();
    match conn.transaction::<(), diesel::result::Error, _>(|conn| {
        diesel::update(schema::users::table.find(format!("{}", user)))
            .set(models::User::new(
                account_template.username,
                account_template.password,
                Some(*user),
            ))
            .execute(conn)?;
        Ok(())
    }) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::BadRequest().body("Username already taken"),
    }
}

async fn create_account(
    req: HttpRequest,
    json: web::Json<models::Account>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();
    let account_template = json.into_inner();
    match conn.transaction::<(), diesel::result::Error, _>(|conn| {
        diesel::insert_into(schema::users::table)
            .values(models::User::new(
                account_template.username,
                account_template.password,
                None,
            ))
            .execute(conn)?;
        Ok(())
    }) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::BadRequest().body("Username already taken"),
    }
}

async fn delete_account(req: HttpRequest) -> impl Responder {
    HttpResponse::NotImplemented().body(format!("{:?}", req))
}

async fn login(req: HttpRequest, data: web::Data<crate::AppData>) -> impl Responder {
    let extensions = req.extensions();
    let user = extensions.get::<Uuid>().unwrap();

    let header = Header::new(Algorithm::HS512);
    let claims = json!({ "sub": user });
    match encode(&header, &claims, &EncodingKey::from_secret(data.settings.jwt_key.as_bytes())) {
        Ok(token) => HttpResponse::Ok().json(json!({ "token": token })),
        Err(e) => {
            log::error!("Couldn't encode JWT: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
