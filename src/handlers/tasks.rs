use futures::future::{Future, IntoFuture};
use uuid::Uuid;
use actix_web::{
    get, put, post, delete, web, Error, HttpRequest, HttpResponse, Scope
};
use diesel::prelude::*;
use crate::schema;
use crate::models;
use crate::AppData;
use crate::handlers::subtasks;
use std::io::SeekFrom::Start;
use crate::models::Task;

pub fn get_scope() -> Scope {
    web::scope("/tasks")
    .service(get_tasks)
    .service(create_task)
    .service(get_task)
    .service(update_task)
    .service(delete_task)
    .service(subtasks::get_scope())
}

#[get("")]
fn get_tasks(req: HttpRequest) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = schema::tasks::table.inner_join(schema::access::table.on(schema::tasks::columns::id.eq(schema::access::columns::object_id)))
        .filter(schema::access::columns::user_id.eq(appdata.get_user().to_string()))
        .select((schema::tasks::columns::id, schema::tasks::columns::database_id))
        .load::<models::QueryableTask>(&*conn);

    match query {
        Ok(query_tasks) => {
            let mut tasks: Vec<models::Task> = Vec::new();
            for task in query_tasks {
                let subtasks_query = schema::subtasks_in_tasks::table
                    .filter(schema::subtasks_in_tasks::columns::task_id.eq(&task.id))
                    .select((schema::subtasks_in_tasks::columns::subtask_id))
                    .load::<String>(&*conn);
                tasks.push(models::Task {
                    id: task.id,
                    database_id: task.database_id,
                    subtasks: subtasks_query.ok(),
                });
            }
            Box::new(Ok(HttpResponse::Ok().json(tasks)).into_future())
        },
        Err(e) => {
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
#[post("")]
fn create_task(req: HttpRequest, json: web::Json<models::Task>) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    // create task object
    let task = json.into_inner();
    let task_id = Uuid::new_v4();
    let new_task = models::QueryableTask {
        id: task_id.to_string(),
        database_id: task.database_id,
    };

    // insert access for user
    match diesel::insert_into(schema::access::table)
        .values(models::Access{ user_id: appdata.current_user.to_string(), object_id: task_id.to_string() })
        .execute(&*conn) {
        Ok(result) => {},
        Err(e) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    }

    // set subtasks belonging to task
    for (position, subtask_id) in task.subtasks.unwrap().iter().enumerate() {
        match diesel::insert_into(schema::subtasks_in_tasks::table)
            .values(models::SubtasksInTask {
                subtask_id: subtask_id.to_string(),
                task_id: task_id.to_string(),
                position: position as i32,
            })
            .execute(&*conn) {
            Ok(result) => {},
            Err(e) => {
                return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
            }
        }
    }

    // insert task object
    match diesel::insert_into(schema::tasks::table).values(new_task).execute(&*conn) {
        Ok(result) => {
            Box::new(Ok(HttpResponse::Ok().json(task_id)).into_future())
        }
        Err(e) => {
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
#[get("/{id}")]
fn get_task(req: HttpRequest, id: web::Path<Uuid>) -> Box<dyn Future<Item=HttpResponse, Error=Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = schema::tasks::table.find(format!("{}", id)).get_result::<models::QueryableTask>(&*conn);

    match query {
        Ok(task) => {
            let subtasks_query = schema::subtasks_in_tasks::table
                .filter(schema::subtasks_in_tasks::columns::task_id.eq(format!("{}", id)))
                .select((schema::subtasks_in_tasks::columns::subtask_id))
                .order(schema::subtasks_in_tasks::position)
                .load::<String>(&*conn);

            Box::new(Ok(HttpResponse::Ok().json(models::Task {
                id: task.id,
                database_id: task.database_id,
                subtasks: subtasks_query.ok(),
            })).into_future())
        },
        Err(e) => {
            match e {
                diesel::result::Error::NotFound => Box::new(Ok(HttpResponse::NotFound().finish()).into_future()),
                _ => Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future()),
            }
        }
    }
}
#[put("/{id}")]
fn update_task(req: HttpRequest, id: web::Path<Uuid>, json: web::Json<models::Task>) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let task = json.into_inner();

    // update tasks
    let query = diesel::update(schema::tasks::table.find(format!("{}", id)))
        .set(models::QueryableTask::from_task(task.clone()))
        .execute(&*conn);
    match query {
        Ok(result) => {},
        Err(e) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    }

    // update which subtasks belong to this task
    match diesel::delete(schema::subtasks_in_tasks::table.filter(schema::subtasks_in_tasks::task_id.eq(task.id.clone())))
        .execute(&*conn) {
        Ok(result) => {
            let mut pos = -1;
            let task_id = task.id.clone();
            let subtasks_in_task: Vec<models::SubtasksInTask> = task.subtasks.unwrap().iter()
                .map(|subtask_id| {
                    pos += 1;
                    models::SubtasksInTask {
                        subtask_id: subtask_id.to_string(),
                        task_id: task_id.clone(),
                        position: pos,
                    }
                })
                .collect();
            match diesel::insert_into(schema::subtasks_in_tasks::table)
                .values(subtasks_in_task)
                .execute(&*conn) {
                Ok(result) => {
                    return Box::new(Ok(HttpResponse::Ok().finish()).into_future());
                }
                Err(e) => {
                    return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
                }
            }
        },
        Err(e) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    }
}
#[delete("/{id}")]
fn delete_task(req: HttpRequest, id: web::Path<Uuid>) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let uuid = id.into_inner();

    match diesel::delete(schema::subtasks_in_tasks::table
        .filter(schema::subtasks_in_tasks::task_id.eq(uuid.to_string())))
        .execute(&*conn) {
        Ok(result) => {},
        Err(e) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    }


    let query = diesel::delete(schema::tasks::table.find(format!("{}", uuid)))
        .execute(&*conn);
    match query {
        Ok(result) => {
            Box::new(Ok(HttpResponse::Ok().finish()).into_future())
        },
        Err(e) => {
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
