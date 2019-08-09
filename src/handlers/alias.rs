use crate::models;
use crate::schema;
use crate::AppData;
use crate::alias_generator::AliasGenerator;
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use diesel::prelude::*;
use futures::future::{Future, IntoFuture};
use uuid::Uuid;

pub fn get_scope() -> Scope {
    web::scope("/alias")
        .service(web::resource("").route(web::post().to_async(create_alias)))
        .service(web::resource("/{id}").route(web::get().to_async(get_alias)))
}


fn create_alias(req: HttpRequest, json: web::Json<models::AliasRequest>) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection() {
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    };

    let alias_req = json.into_inner();
    let alias_generator = AliasGenerator::from_file("words.txt");
    let mut alias = models::Alias {
        alias: alias_generator.generate(4),
        object_id: alias_req.object_id,
        object_type: alias_req.object_type,
    };
    loop {
        match diesel::insert_into(schema::aliases::table)
            .values(alias.clone())
            .execute(&*conn) {
            Ok(_) => {
                return Box::new(Ok(HttpResponse::Ok().json(alias.alias)).into_future())
            }
            Err(e) => {
                match e {
                    diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _)
                    | diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _) => {
                        alias.alias = alias_generator.generate(4);
                    }
                    _ => {
                        return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
                    }
                }
            }
        }
    }
    // cannot be reached
    return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
}

fn get_alias(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    return Box::new(Ok(HttpResponse::Ok().finish()).into_future())
}