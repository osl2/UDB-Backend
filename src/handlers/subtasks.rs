use crate::models;
use crate::schema;
use crate::solution_compare::compare_solutions;
use actix_web::{web, HttpRequest, HttpResponse, Responder, Scope};
use actix_web::HttpMessage;
use std::cell::RefCell;
use uuid::Uuid;

use diesel::{
    r2d2::{self, ConnectionManager},
    Connection, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SqliteConnection,
};

pub fn get_scope() -> Scope {
    web::scope("/subtasks")
        .service(
            web::resource("")
                .route(web::get().to(get_subtasks))
                .route(web::post().to(create_subtask)),
        )
        .service(
            web::resource("/{id}")
                .route(web::get().to(get_subtask))
                .route(web::put().to(update_subtask))
                .route(web::delete().to(delete_subtask)),
        )
        .service(web::resource("/{id}/verify").route(web::post().to(verify_subtask_solution)))
}

async fn get_subtasks(req: HttpRequest) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();
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
        .load::<models::Subtask>(&mut *conn)
    {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => {
            log::error!("Couldn't get subtasks: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn create_subtask(
    req: HttpRequest,
    json: web::Json<models::Subtask>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();
    let sub = extensions
        .get::<actix_web_jwt_middleware::AuthenticationData>()
        .unwrap()
        .claims
        .sub
        .clone()
        .unwrap();

    match conn.transaction::<Uuid, diesel::result::Error, _>(|conn| {
        let mut new_subtask = json.into_inner();
        let id = Uuid::new_v4();
        new_subtask.id = id.to_string();

        diesel::insert_into(schema::access::table)
            .values(models::Access {
                user_id: sub,
                object_id: id.to_string(),
            })
            .execute(conn)?;

        diesel::insert_into(schema::subtasks::table)
            .values(new_subtask)
            .execute(conn)?;

        Ok(id)
    }) {
        Ok(result) => HttpResponse::Ok().body(result.to_string()),
        Err(e) => {
            log::error!("Couldn't create subtask: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn get_subtask(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();
    let uuid = id.into_inner();

    match schema::subtasks::table
        .find(format!("{}", uuid))
        .get_result::<models::Subtask>(&mut *conn)
    {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => match e {
            diesel::result::Error::NotFound => HttpResponse::NotFound().finish(),
            e => {
                log::error!("Couldn't get subtask: {}", e);
                HttpResponse::InternalServerError().finish()
            }
        },
    }
}

async fn update_subtask(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Subtask>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();
    let uuid = id.into_inner();

    match diesel::update(schema::subtasks::table.find(uuid.to_string()))
        .set(json.into_inner())
        .execute(&mut *conn)
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Couldn't update subtask: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn delete_subtask(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();
    let subtask_id = id.into_inner();

    match conn.transaction::<(), diesel::result::Error, _>(|conn| {
        diesel::delete(schema::subtasks::table.find(subtask_id.to_string())).execute(conn)?;

        diesel::delete(
            schema::subtasks_in_tasks::table
                .filter(schema::subtasks_in_tasks::subtask_id.eq(subtask_id.to_string())),
        )
        .execute(conn)?;

        diesel::delete(
            schema::access::table.filter(schema::access::object_id.eq(subtask_id.to_string())),
        )
        .execute(conn)?;
        Ok(())
    }) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Couldn't delete subtask: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn verify_subtask_solution(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Solution>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn_cell = extensions
        .get::<RefCell<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>>()
        .unwrap();
    let mut conn = conn_cell.borrow_mut();
    let subtask_id = id.into_inner();
    let student_solution = json.into_inner();

    match schema::subtasks::table
        .find(format!("{}", subtask_id))
        .get_result::<models::Subtask>(&mut *conn)
    {
        Ok(subtask) => {
            if !subtask.is_solution_verifiable || !subtask.is_solution_visible {
                return HttpResponse::NotFound().finish();
            }

            let teacher_solution = subtask.content.get_solution().unwrap();
            let result = compare_solutions(student_solution, teacher_solution);
            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            log::error!("Couldn't compare solution: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
