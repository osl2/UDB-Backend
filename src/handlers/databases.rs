use crate::{models, schema, database::DatabaseConnection};
use actix_web::{web, Error, FromRequest, HttpRequest, HttpResponse, Scope};

use futures::future::{Future, IntoFuture};
use uuid::Uuid;

use diesel::{
    r2d2::{self, ConnectionManager},
    Connection, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl,
};

pub fn get_scope() -> Scope {
    let json_config = web::Json::<models::Database>::configure(|cfg| {
        cfg.limit(4194304) //4MB limit
    });
    web::scope("/databases")
        .service(
            web::resource("")
                .data(json_config.clone())
                .route(web::get().to_async(get_databases))
                .route(web::post().to_async(create_database)),
        )
        .service(
            web::resource("/{id}")
                .data(json_config.clone())
                .route(web::get().to_async(get_database))
                .route(web::put().to_async(update_database))
                .route(web::delete().to_async(delete_database)),
        )
}

pub fn get_databases(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();
    let sub = extensions
        .get::<actix_web_jwt_middleware::AuthenticationData>()
        .unwrap()
        .claims
        .sub
        .clone()
        .unwrap();

    match schema::databases::table
        .inner_join(
            schema::access::table
                .on(schema::databases::columns::id.eq(schema::access::columns::object_id)),
        )
        .filter(schema::access::columns::user_id.eq(sub))
        .select((
            schema::databases::columns::id,
            schema::databases::columns::name,
            schema::databases::columns::content,
        ))
        .load::<models::Database>(&*conn)
    {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(e) => {
            log::error!("Couldn't load database: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}

pub fn create_database(
    req: HttpRequest,
    json: web::Json<models::Database>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();
    let sub = extensions
        .get::<actix_web_jwt_middleware::AuthenticationData>()
        .unwrap()
        .claims
        .sub
        .clone()
        .unwrap();

    match conn.transaction::<Uuid, diesel::result::Error, _>(|| {
        // create database object
        let mut new_database = json.into_inner();
        let id = Uuid::new_v4();
        new_database.id = id.to_string();

        // insert access for user
        diesel::insert_into(schema::access::table)
            .values(models::Access {
                user_id: sub,
                object_id: id.to_string(),
            })
            .execute(&*conn)?;

        // insert database object
        diesel::insert_into(schema::databases::table)
            .values(new_database)
            .execute(&*conn)?;

        Ok(id)
    }) {
        Ok(id) => Box::new(Ok(HttpResponse::Ok().body(id.to_string())).into_future()),
        Err(e) => {
            log::error!("Couldn't create database: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}

pub fn get_database(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    match schema::databases::table
        .find(format!("{}", id))
        .get_result::<models::Database>(&*conn)
    {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(e) => match e {
            diesel::result::Error::NotFound => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            e => {
                log::error!("Couldn't load database: {}", e);
                Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
            }
        },
    }
}

pub fn update_database(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Database>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    match diesel::update(schema::databases::table.find(format!("{}", id)))
        .set(json.into_inner())
        .execute(&*conn)
    {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => {
            log::error!("Couldn't update database: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}

pub fn delete_database(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    match diesel::delete(schema::databases::table.find(format!("{}", id.into_inner())))
        .execute(&*conn)
    {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => {
            log::error!("Couldn't delete database: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
