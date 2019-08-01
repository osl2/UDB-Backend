use futures::future::{Future, IntoFuture};
use uuid::Uuid;
use actix_web::{
    get, put, post, delete, web, Error, HttpRequest, HttpResponse, Scope
};
use diesel::prelude::*;
use crate::AppData;
use crate::schema;
use crate::models;
use crate::models::SubtasksInTask;

pub fn get_scope() -> Scope {
    web::scope("/{task_id}/subtasks")
    .service(get_subtasks)
    .service(create_subtask)
    .service(get_subtask)
    .service(update_subtask)
    .service(delete_subtask)
}

#[get("")]
fn get_subtasks(req: HttpRequest, task_id: web::Path<Uuid>) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = schema::subtasks::table.inner_join(schema::access::table.on(schema::subtasks::columns::id.eq(schema::access::columns::object_id)))
        .filter(schema::access::columns::user_id.eq(appdata.current_user.to_string()))
        .select((schema::subtasks::columns::id, schema::subtasks::columns::instruction, schema::subtasks::is_solution_visible, schema::subtasks::is_solution_verifiable, schema::subtasks::allowed_sql, schema::subtasks::content))
        .load::<models::Subtask>(&*conn);

    match query {
        Ok(result) => {
            Box::new(Ok(HttpResponse::Ok().json(result)).into_future())
        },
        Err(e) => {
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
#[post("")]
fn create_subtask(req: HttpRequest, task_id: web::Path<Uuid>, json: web::Json<models::Subtask>) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    // create subtask object
    let mut new_subtask = json.into_inner();
    let id = Uuid::new_v4();
    new_subtask.id = id.to_string();

    // get task id
    let task_id = task_id.into_inner();

    // insert access for user
    match diesel::insert_into(schema::access::table)
        .values(models::Access{ user_id: appdata.current_user.to_string(), object_id: id.to_string() })
        .execute(&*conn) {
        Ok(result) => {}
        Err(e) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    }

    // insert subtask object
    match diesel::insert_into(schema::subtasks::table).values(new_subtask).execute(&*conn) {
        Ok(result) => {}
        Err(e) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    }

    // get max position
    let mut max_position = 0;
    match schema::subtasks_in_tasks::table.filter(schema::subtasks_in_tasks::task_id.eq(task_id.to_string()))
        .order(schema::subtasks_in_tasks::position.desc())
        .get_result::<models::SubtasksInTask>(&*conn) {
        Ok(association) => {
            max_position = association.position;
        }
        Err(e) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    }

    // update task association
    match diesel::insert_into(schema::subtasks_in_tasks::table).values(SubtasksInTask {
        task_id: task_id.to_string(),
        subtask_id: id.to_string(),
        position: max_position + 1,
    }).execute(&*conn) {
        Ok(result) => {
            Box::new(Ok(HttpResponse::Ok().json(id)).into_future())
        }
        Err(e) => {
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
#[get("/{subtask_id}")]
fn get_subtask(req: HttpRequest, ids: web::Path<(Uuid, Uuid)>) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = schema::subtasks::table.find(format!("{}", ids.1)).get_result::<models::Subtask>(&*conn);

    match query {
        Ok(result) => {
            Box::new(Ok(HttpResponse::Ok().json(result)).into_future())
        },
        Err(e) => {
            match e {
                diesel::result::Error::NotFound => Box::new(Ok(HttpResponse::NotFound().finish()).into_future()),
                _ => Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future()),
            }
        }
    }
}
#[put("/{subtask_id}")]
fn update_subtask(req: HttpRequest, ids: web::Path<(Uuid, Uuid)>) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
#[delete("/{subtask_id}")]
fn delete_subtask(req: HttpRequest, ids: web::Path<(Uuid, Uuid)>) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let task_id = ids.0;
    let subtask_id = ids.1;

    // delete subtask
    let query = diesel::delete(schema::subtasks::table.find(subtask_id.to_string()))
        .execute(&*conn);
    match query {
        Ok(result) => {},
        Err(e) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    }

    // delete task associations
    match diesel::delete(schema::subtasks_in_tasks::table.filter(schema::subtasks_in_tasks::subtask_id.eq(subtask_id.to_string())))
        .execute(&*conn) {
        Ok(result) => {
            Box::new(Ok(HttpResponse::Ok().finish()).into_future())
        },
        Err(e) => {
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
#[post("/{subtask_id}")]
fn verify_subtask_solution(req: HttpRequest, ids: web::Path<(Uuid, Uuid)>, json: web::Json<models::Solution>) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let task_id = ids.0;
    let subtask_id = ids.1;

    // get student solution
    let student_solution = json.into_inner();

    // get teacher solution
    match schema::subtasks::table.find(format!("{}", subtask_id)).get_result::<models::Subtask>(&*conn) {
        Ok(subtask) => {
            if !subtask.is_solution_verifiable || !subtask.is_solution_visible {
                // this subtask does not have a public solution
                return Box::new(Ok(HttpResponse::NotFound().finish()).into_future());
            }
            // TODO: compare solutions
            Box::new(Ok(HttpResponse::Ok().json(false)).into_future())
        }
        Err(e) => {
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
