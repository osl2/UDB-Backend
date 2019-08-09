use crate::alias_generator::AliasGenerator;
use crate::models;
use crate::schema;
use crate::AppData;
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use diesel::prelude::*;
use futures::future::{Future, IntoFuture};
use lazy_static::lazy_static;

pub fn get_scope() -> Scope {
    web::scope("/alias")
        .service(web::resource("").route(web::post().to_async(create_alias)))
        .service(web::resource("/{id}").route(web::get().to_async(get_alias)))
}

fn create_alias(
    req: HttpRequest,
    json: web::Json<models::AliasRequest>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection() {
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    };

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
            Ok(_) => return Box::new(Ok(HttpResponse::Ok().json(alias.alias)).into_future()),
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
                e => {
                    log::error!(
                        "Unexpected Database error while trying to create alias: {}",
                        e
                    );
                    return Box::new(
                        Ok(HttpResponse::InternalServerError().finish()).into_future(),
                    );
                }
            },
        }
    }
    return Box::new(
        Ok(HttpResponse::InternalServerError().body("Couldn't find a free alias.")).into_future(),
    );
}

fn get_alias(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().body(format!("{:?}", req))).into_future())
}
