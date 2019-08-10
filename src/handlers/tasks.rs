use crate::models;
use crate::schema;
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};

use futures::future::{Future, IntoFuture};
use uuid::Uuid;

use diesel::{
    r2d2::{self, ConnectionManager},
    Connection, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SqliteConnection,
};
pub fn get_scope() -> Scope {
    web::scope("/tasks")
        .service(
            web::resource("")
                .route(web::get().to_async(get_tasks))
                .route(web::post().to_async(create_task)),
        )
        .service(
            web::resource("/{id}")
                .route(web::get().to_async(get_task))
                .route(web::put().to_async(update_task))
                .route(web::delete().to_async(delete_task)),
        )
}

fn get_tasks(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>()
        .unwrap();
    let sub = extensions
        .get::<actix_web_jwt_middleware::AuthenticationData>()
        .unwrap()
        .claims
        .sub
        .clone()
        .unwrap();

    match schema::tasks::table
        .inner_join(
            schema::access::table
                .on(schema::tasks::columns::id.eq(schema::access::columns::object_id)),
        )
        .filter(schema::access::columns::user_id.eq(sub))
        .select((
            schema::tasks::columns::id,
            schema::tasks::columns::database_id,
        ))
        .load::<models::QueryableTask>(&*conn)
    {
        Ok(query_tasks) => {
            let mut tasks: Vec<models::Task> = Vec::new();
            for task in query_tasks {
                let subtasks_query = schema::subtasks_in_tasks::table
                    .filter(schema::subtasks_in_tasks::columns::task_id.eq(&task.id))
                    .select(schema::subtasks_in_tasks::columns::subtask_id)
                    .order(schema::subtasks_in_tasks::position)
                    .load::<String>(&*conn);
                tasks.push(models::Task {
                    id: task.id,
                    database_id: task.database_id,
                    subtasks: subtasks_query.ok(),
                });
            }
            Box::new(Ok(HttpResponse::Ok().json(tasks)).into_future())
        }
        Err(e) => {
            log::error!("Couldn't get tasks: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}

fn create_task(
    req: HttpRequest,
    json: web::Json<models::Task>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>()
        .unwrap();
    let sub = extensions
        .get::<actix_web_jwt_middleware::AuthenticationData>()
        .unwrap()
        .claims
        .sub
        .clone()
        .unwrap();

    match conn.transaction::<Uuid, diesel::result::Error, _>(|| {
        // create task object
        let task = json.into_inner();
        let task_id = Uuid::new_v4();
        let new_task = models::QueryableTask {
            id: task_id.to_string(),
            database_id: task.database_id,
        };

        // insert access for user
        diesel::insert_into(schema::access::table)
            .values(models::Access {
                user_id: sub,
                object_id: task_id.to_string(),
            })
            .execute(&*conn)?;

        // set subtasks belonging to task
        for (position, subtask_id) in task.subtasks.unwrap().iter().enumerate() {
            diesel::insert_into(schema::subtasks_in_tasks::table)
                .values(models::SubtasksInTask {
                    subtask_id: subtask_id.to_string(),
                    task_id: task_id.to_string(),
                    position: position as i32,
                })
                .execute(&*conn)?;
        }

        // insert task object
        diesel::insert_into(schema::tasks::table)
            .values(new_task)
            .execute(&*conn)?;

        Ok(task_id)
    }) {
        Ok(id) => Box::new(Ok(HttpResponse::Ok().json(id)).into_future()),
        Err(e) => {
            log::error!("Couldn't create task: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}

fn get_task(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>()
        .unwrap();

    match schema::tasks::table
        .find(format!("{}", id))
        .get_result::<models::QueryableTask>(&*conn)
    {
        Ok(task) => {
            let subtasks_query = schema::subtasks_in_tasks::table
                .filter(schema::subtasks_in_tasks::columns::task_id.eq(format!("{}", id)))
                .select(schema::subtasks_in_tasks::columns::subtask_id)
                .order(schema::subtasks_in_tasks::position)
                .load::<String>(&*conn);

            Box::new(
                Ok(HttpResponse::Ok().json(models::Task {
                    id: task.id,
                    database_id: task.database_id,
                    subtasks: subtasks_query.ok(),
                }))
                .into_future(),
            )
        }
        Err(e) => match e {
            diesel::result::Error::NotFound => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            e => {
                log::error!("Couldn't load task: {}", e);
                Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
            }
        },
    }
}

fn update_task(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Task>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>()
        .unwrap();

    match conn.transaction::<(), diesel::result::Error, _>(|| {
        let task = json.into_inner();

        // update tasks
        diesel::update(schema::tasks::table.find(format!("{}", id)))
            .set(models::QueryableTask::from_task(task.clone()))
            .execute(&*conn)?;

        // update which subtasks belong to this task
        diesel::delete(
            schema::subtasks_in_tasks::table
                .filter(schema::subtasks_in_tasks::task_id.eq(task.id.clone())),
        )
        .execute(&*conn)?;
        let mut pos = -1;
        let task_id = task.id.clone();
        let subtasks_in_task: Vec<models::SubtasksInTask> = task
            .subtasks
            .unwrap()
            .iter()
            .map(|subtask_id| {
                pos += 1;
                models::SubtasksInTask {
                    subtask_id: subtask_id.to_string(),
                    task_id: task_id.clone(),
                    position: pos,
                }
            })
            .collect();
        for subtask in subtasks_in_task {
            diesel::insert_into(schema::subtasks_in_tasks::table)
                .values(subtask)
                .execute(&*conn)?;
        }
        Ok(())
    }) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => {
            log::error!("Couldn't update task: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}

fn delete_task(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>()
        .unwrap();

    let uuid = id.into_inner();

    match conn.transaction::<(), diesel::result::Error, _>(|| {
        diesel::delete(
            schema::subtasks_in_tasks::table
                .filter(schema::subtasks_in_tasks::task_id.eq(uuid.to_string())),
        )
        .execute(&*conn)?;
        diesel::delete(
            schema::tasks_in_worksheets::table
                .filter(schema::tasks_in_worksheets::task_id.eq(uuid.to_string())),
        )
        .execute(&*conn)?;
        diesel::delete(
            schema::access::table.filter(schema::access::object_id.eq(uuid.to_string())),
        )
        .execute(&*conn)?;
        diesel::delete(schema::tasks::table.find(format!("{}", uuid))).execute(&*conn)?;
        Ok(())
    }) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => {
            log::error!("Couldn't delete task: {}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
