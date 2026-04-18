use crate::models;
use crate::models::TasksInWorksheet;
use crate::schema;
use actix_web::{web, HttpRequest, HttpResponse, Responder, Scope};
use actix_web::HttpMessage;
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
    SqliteConnection,
};
use uuid::Uuid;

pub fn get_scope() -> Scope {
    web::scope("/worksheets")
        .service(
            web::resource("")
                .route(web::get().to(get_worksheets))
                .route(web::post().to(create_worksheet)),
        )
        .service(
            web::resource("/{id}")
                .route(web::get().to(get_worksheet))
                .route(web::put().to(update_worksheet))
                .route(web::delete().to(delete_worksheet)),
        )
}

async fn get_worksheets(req: HttpRequest) -> impl Responder {
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

    match conn.transaction::<Vec<models::Worksheet>, diesel::result::Error, _>(|| {
        let mut worksheets: Vec<models::Worksheet> = Vec::new();
        let query_worksheets = schema::worksheets::table
            .inner_join(
                schema::access::table
                    .on(schema::worksheets::columns::id.eq(schema::access::columns::object_id)),
            )
            .filter(schema::access::columns::user_id.eq(sub))
            .select((
                schema::worksheets::columns::id,
                schema::worksheets::columns::name,
                schema::worksheets::columns::is_online,
                schema::worksheets::columns::is_solution_online,
            ))
            .load::<models::QueryableWorksheet>(&*conn)?;
        for worksheet in query_worksheets {
            let tasks_query = schema::tasks_in_worksheets::table
                .filter(schema::tasks_in_worksheets::columns::worksheet_id.eq(&worksheet.id))
                .select(schema::tasks_in_worksheets::columns::task_id)
                .order(schema::tasks_in_worksheets::position)
                .load::<String>(&*conn);
            worksheets.push(models::Worksheet {
                id: worksheet.id,
                name: worksheet.name,
                is_online: worksheet.is_online,
                is_solution_online: worksheet.is_solution_online,
                tasks: tasks_query.unwrap(),
            });
        }
        Ok(worksheets)
    }) {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => {
            log::error!("Couldn't load worksheet: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn create_worksheet(
    req: HttpRequest,
    json: web::Json<models::Worksheet>,
) -> impl Responder {
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
        let worksheet = json.into_inner();
        let worksheet_id = Uuid::new_v4();
        let new_worksheet = models::QueryableWorksheet {
            id: worksheet_id.to_string(),
            name: worksheet.name,
            is_online: worksheet.is_online,
            is_solution_online: worksheet.is_solution_online,
        };

        diesel::insert_into(schema::access::table)
            .values(models::Access {
                user_id: sub,
                object_id: worksheet_id.to_string(),
            })
            .execute(&*conn)?;

        for (position, task_id) in worksheet.tasks.iter().enumerate() {
            diesel::insert_into(schema::tasks_in_worksheets::table)
                .values(models::TasksInWorksheet {
                    task_id: task_id.to_string(),
                    worksheet_id: worksheet_id.to_string(),
                    position: position as i32,
                })
                .execute(&*conn)?;
        }

        diesel::insert_into(schema::worksheets::table)
            .values(new_worksheet)
            .execute(&*conn)?;

        Ok(worksheet_id)
    }) {
        Ok(id) => HttpResponse::Ok().body(id.to_string()),
        Err(e) => {
            log::error!("Couldn't create worksheet: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn get_worksheet(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>()
        .unwrap();
    let uuid = id.into_inner();

    match conn.transaction::<models::Worksheet, diesel::result::Error, _>(|| {
        let worksheet = schema::worksheets::table
            .find(format!("{}", uuid))
            .get_result::<models::QueryableWorksheet>(&*conn)?;

        let tasks_query = schema::tasks_in_worksheets::table
            .filter(schema::tasks_in_worksheets::columns::worksheet_id.eq(format!("{}", uuid)))
            .select(schema::tasks_in_worksheets::columns::task_id)
            .order(schema::tasks_in_worksheets::position)
            .load::<String>(&*conn);

        Ok(models::Worksheet {
            id: worksheet.id,
            name: worksheet.name,
            is_online: worksheet.is_online,
            is_solution_online: worksheet.is_solution_online,
            tasks: tasks_query.unwrap(),
        })
    }) {
        Ok(sheet) => HttpResponse::Ok().json(sheet),
        Err(e) => match e {
            diesel::result::Error::NotFound => HttpResponse::NotFound().finish(),
            e => {
                log::error!("{}", e);
                HttpResponse::InternalServerError().finish()
            }
        },
    }
}

async fn update_worksheet(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Worksheet>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>()
        .unwrap();
    let uuid = id.into_inner();

    match conn.transaction::<(), diesel::result::Error, _>(|| {
        let worksheet = json.into_inner();

        diesel::update(schema::worksheets::table.find(format!("{}", uuid)))
            .set(models::QueryableWorksheet::from_worksheet(worksheet.clone()))
            .execute(&*conn)?;

        diesel::delete(
            schema::tasks_in_worksheets::table
                .filter(schema::tasks_in_worksheets::worksheet_id.eq(worksheet.id.clone())),
        )
        .execute(&*conn)?;
        let mut pos = -1;
        let worksheet_id = worksheet.id.clone();
        let tasks_in_worksheet: Vec<TasksInWorksheet> = worksheet
            .tasks
            .iter()
            .map(|task_id| {
                pos += 1;
                models::TasksInWorksheet {
                    task_id: task_id.to_string(),
                    worksheet_id: worksheet_id.clone(),
                    position: pos,
                }
            })
            .collect();

        for task in tasks_in_worksheet {
            diesel::insert_into(schema::tasks_in_worksheets::table)
                .values(task)
                .execute(&*conn)?;
        }
        Ok(())
    }) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Couldn't update worksheet: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn delete_worksheet(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> impl Responder {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>>()
        .unwrap();
    let uuid = id.into_inner();

    match conn.transaction::<(), diesel::result::Error, _>(|| {
        diesel::delete(
            schema::tasks_in_worksheets::table
                .filter(schema::tasks_in_worksheets::worksheet_id.eq(uuid.to_string())),
        )
        .execute(&*conn)?;
        diesel::delete(schema::worksheets::table.find(format!("{}", uuid))).execute(&*conn)?;
        Ok(())
    }) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Couldn't delete worksheet: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
