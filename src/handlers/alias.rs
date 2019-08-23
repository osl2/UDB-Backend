use crate::alias_generator::AliasGenerator;
use crate::{models, schema, database::DatabaseConnection};
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use diesel::{
    r2d2::{self, ConnectionManager},
    Connection, ExpressionMethods, QueryDsl, RunQueryDsl,
};
use futures::future::{Future, IntoFuture};
use lazy_static::lazy_static;
use uuid::Uuid;

pub fn get_scope() -> Scope {
    web::scope("/alias")
        .service(web::resource("").route(web::post().to_async(create_alias)))
        .service(web::resource("/{id}").route(web::get().to_async(get_alias)))
        .service(web::resource("/uuid/{alias}").route(web::get().to_async(get_uuid)))
}

fn create_alias(
    req: HttpRequest,
    json: web::Json<models::AliasRequest>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    match conn.transaction::<String, AliasError, _>(|| {
        let alias_req = json.into_inner();
        lazy_static! {
            static ref GENERATOR: AliasGenerator = AliasGenerator::default();
        }
        let mut alias = models::Alias {
            alias: GENERATOR.generate(4),
            object_id: alias_req.object_id,
            object_type: alias_req.object_type,
        };
        // Try to find a free alias 20 times
        for i in 0..20 {
            match diesel::insert_into(schema::aliases::table)
                .values(alias.clone())
                .execute(&*conn)
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
                        // Try to find a four word alias for five times, then five words for five times,
                        // then six for five times and then seven for five times.
                        alias.alias = GENERATOR.generate(4 + i / 5);
                    }
                    e => return Err(AliasError::from(e)),
                },
            }
        }
        Err(AliasError::NoFreeAliases)
    }) {
        Ok(alias) => Box::new(Ok(HttpResponse::Ok().body(alias)).into_future()),
        Err(e) => match e {
            AliasError::Diesel(e) => {
                log::error!("Couldn't create new alias: {}", e);
                Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
            }
            AliasError::NoFreeAliases => Box::new(
                Ok(HttpResponse::InternalServerError().body("Couldn't find a free alias."))
                    .into_future(),
            ),
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

fn get_alias(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    match schema::aliases::table
        .filter(schema::aliases::object_id.eq(format!("{}", id)))
        .get_result::<models::Alias>(&*conn)
    {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(e) => {
            log::error!("Couldn't get alias: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}

fn get_uuid(
    req: HttpRequest,
    alias: web::Path<String>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    match schema::aliases::table
        .filter(schema::aliases::alias.eq(format!("{}", alias)))
        .get_result::<models::Alias>(&*conn)
    {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(e) => {
            log::error!("Couldn't get alias: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
