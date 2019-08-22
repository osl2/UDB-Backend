use crate::{models, schema, solution_compare::compare_solutions, database::DatabaseConnection};
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};

use futures::future::{Future, IntoFuture};
use uuid::Uuid;

use diesel::{
    r2d2::{self, ConnectionManager},
    Connection, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl,
};

pub fn get_scope() -> Scope {
    web::scope("/subtasks")
        .service(
            web::resource("")
                .route(web::get().to_async(get_subtasks))
                .route(web::post().to_async(create_subtask)),
        )
        .service(
            web::resource("/{id}")
                .route(web::get().to_async(get_subtask))
                .route(web::put().to_async(update_subtask))
                .route(web::delete().to_async(delete_subtask)),
        )
        .service(web::resource("/{id}/verify").route(web::post().to_async(verify_subtask_solution)))
}

fn get_subtasks(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
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

    match schema::subtasks::table
        .inner_join(
            schema::access::table
                .on(schema::subtasks::columns::id.eq(schema::access::columns::object_id)),
        )
        .filter(schema::access::columns::user_id.eq(sub))
        .select((
            schema::subtasks::columns::id,
            schema::subtasks::columns::instruction,
            schema::subtasks::is_solution_visible,
            schema::subtasks::is_solution_verifiable,
            schema::subtasks::content,
        ))
        .load::<models::Subtask>(&*conn)
    {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(e) => {
            log::error!("Couldn't get subtasks: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
fn create_subtask(
    req: HttpRequest,
    json: web::Json<models::Subtask>,
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
        // create subtask object
        let mut new_subtask = json.into_inner();
        let id = Uuid::new_v4();
        new_subtask.id = id.to_string();

        // insert access for user
        diesel::insert_into(schema::access::table)
            .values(models::Access {
                user_id: sub,
                object_id: id.to_string(),
            })
            .execute(&*conn)?;

        // insert subtask object
        diesel::insert_into(schema::subtasks::table)
            .values(new_subtask)
            .execute(&*conn)?;

        Ok(id)
    }) {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().body(result.to_string())).into_future()),
        Err(e) => {
            log::error!("Couldn't create subtask: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
fn get_subtask(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    match schema::subtasks::table
        .find(format!("{}", id))
        .get_result::<models::Subtask>(&*conn)
    {
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(e) => match e {
            diesel::result::Error::NotFound => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            e => {
                log::error!("Couldn't get subtask: {}", e);
                Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
            }
        },
    }
}
fn update_subtask(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Subtask>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    match diesel::update(schema::subtasks::table.find(id.into_inner().to_string()))
        .set(json.into_inner())
        .execute(&*conn)
    {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => {
            log::error!("Couldn't update subtask: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
fn delete_subtask(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    let subtask_id = id.into_inner();

    match conn.transaction::<(), diesel::result::Error, _>(|| {
        // delete subtask
        diesel::delete(schema::subtasks::table.find(subtask_id.to_string())).execute(&*conn)?;

        // delete task associations
        diesel::delete(
            schema::subtasks_in_tasks::table
                .filter(schema::subtasks_in_tasks::subtask_id.eq(subtask_id.to_string())),
        )
        .execute(&*conn)?;

        // delete access
        diesel::delete(
            schema::access::table.filter(schema::access::object_id.eq(subtask_id.to_string())),
        )
        .execute(&*conn)?;
        Ok(())
    }) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => {
            log::error!("Couldn't delete subtask: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
fn verify_subtask_solution(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Solution>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    let subtask_id = id.into_inner();

    // get student solution
    let student_solution = json.into_inner();

    // get teacher solution
    match schema::subtasks::table
        .find(format!("{}", subtask_id))
        .get_result::<models::Subtask>(&*conn)
    {
        Ok(subtask) => {
            if !subtask.is_solution_verifiable || !subtask.is_solution_visible {
                // this subtask does not have a public solution
                return Box::new(Ok(HttpResponse::NotFound().finish()).into_future());
            }

            let teacher_solution = subtask.content.get_solution().unwrap();

            let result = compare_solutions(student_solution, teacher_solution);

            Box::new(Ok(HttpResponse::Ok().json(result)).into_future())
        }
        Err(e) => {
            log::error!("Couldn't compare solution: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
