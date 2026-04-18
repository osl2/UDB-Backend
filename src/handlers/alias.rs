use crate::alias_generator::AliasGenerator;
use crate::models;
use crate::schema;
use actix_web::{web, HttpRequest, HttpResponse, Responder, Scope};
use actix_web::HttpMessage;
use diesel::{
    r2d2::{self, ConnectionManager},
    sqlite::SqliteConnection,
    Connection, ExpressionMethods, QueryDsl, RunQueryDsl,
};
use lazy_static::lazy_static;
use std::cell::RefCell;
use uuid::Uuid;

pub fn get_scope() -> Scope {
    web::scope("/alias")
        .service(web::resource("").route(web::post().to(create_alias)))
        .service(web::resource("/{id}").route(web::get().to(get_alias)))
        .service(web::resource("/uuid/{alias}").route(web::get().to(get_uuid)))
}

async fn create_alias(
    req: HttpRequest,
    json: web::Json<models::AliasRequest>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();

    match conn.transaction::<String, AliasError, _>(|conn| {
        let alias_req = json.into_inner();
        lazy_static! {
            static ref GENERATOR: AliasGenerator = AliasGenerator::default();
        }
        let mut alias = models::Alias {
            alias: GENERATOR.generate(4),
            object_id: alias_req.object_id,
            object_type: alias_req.object_type,
        };
        for i in 0..20 {
            match diesel::insert_into(schema::aliases::table)
                .values(alias.clone())
                .execute(conn)
            {
                Ok(_) => return Ok(alias.alias),
                Err(e) => match e {
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                        _,
                    )
                    | diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UniqueViolation,
                        _,
                    ) => {
                        alias.alias = GENERATOR.generate(4 + i / 5);
                    }
                    e => return Err(AliasError::from(e)),
                },
            }
        }
        Err(AliasError::NoFreeAliases)
    }) {
        Ok(alias) => HttpResponse::Ok().body(alias),
        Err(e) => match e {
            AliasError::Diesel(e) => {
                log::error!("Couldn't create new alias: {}", e);
                HttpResponse::InternalServerError().finish()
            }
            AliasError::NoFreeAliases => {
                HttpResponse::InternalServerError().body("Couldn't find a free alias.")
            }
        },
    }
}

enum AliasError {
    Diesel(diesel::result::Error),
    NoFreeAliases,
}

impl From<diesel::result::Error> for AliasError {
    fn from(val: diesel::result::Error) -> AliasError {
        AliasError::Diesel(val)
    }
}

async fn get_alias(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();
    let uuid = id.into_inner();

    match schema::aliases::table
        .filter(schema::aliases::object_id.eq(format!("{}", uuid)))
        .get_result::<models::Alias>(&mut *conn)
    {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => {
            log::error!("Couldn't get alias: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn get_uuid(
    req: HttpRequest,
    alias: web::Path<String>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();

    match schema::aliases::table
        .filter(schema::aliases::alias.eq(alias.into_inner()))
        .get_result::<models::Alias>(&mut *conn)
    {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => {
            log::error!("Couldn't get alias: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
