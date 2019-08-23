use crate::{models::{self, TasksInWorksheet}, schema, database::DatabaseConnection};
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
};
use futures::future::{Future, IntoFuture};
use uuid::Uuid;

pub fn get_scope() -> Scope {
    web::scope("/worksheets")
        .service(
            web::resource("")
                .route(web::get().to_async(get_worksheets))
                .route(web::post().to_async(create_worksheet)),
        )
        .service(
            web::resource("/{id}")
                .route(web::get().to_async(get_worksheet))
                .route(web::put().to_async(update_worksheet))
                .route(web::delete().to_async(delete_worksheet)),
        )
}

fn get_worksheets(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
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
        Ok(result) => Box::new(Ok(HttpResponse::Ok().json(result)).into_future()),
        Err(e) => {
            log::error!("Couldn't load worksheet: {:?}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}

fn create_worksheet(
    req: HttpRequest,
    json: web::Json<models::Worksheet>,
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
        // create worksheet object
        let worksheet = json.into_inner();
        let worksheet_id = Uuid::new_v4();
        let new_worksheet = models::QueryableWorksheet {
            id: worksheet_id.to_string(),
            name: worksheet.name,
            is_online: worksheet.is_online,
            is_solution_online: worksheet.is_solution_online,
        };

        // insert access for user
        diesel::insert_into(schema::access::table)
            .values(models::Access {
                user_id: sub,
                object_id: worksheet_id.to_string(),
            })
            .execute(&*conn)?;

        // set tasks belonging to worksheet
        for (position, task_id) in worksheet.tasks.iter().enumerate() {
            diesel::insert_into(schema::tasks_in_worksheets::table)
                .values(models::TasksInWorksheet {
                    task_id: task_id.to_string(),
                    worksheet_id: worksheet_id.to_string(),
                    position: position as i32,
                })
                .execute(&*conn)?;
        }

        // insert worksheet object
        diesel::insert_into(schema::worksheets::table)
            .values(new_worksheet)
            .execute(&*conn)?;

        Ok(worksheet_id)
    }) {
        Ok(id) => Box::new(Ok(HttpResponse::Ok().body(id.to_string())).into_future()),
        Err(e) => {
            log::error!("Couldn't create worksheet: {:?}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}

fn get_worksheet(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    match conn.transaction::<models::Worksheet, diesel::result::Error, _>(|| {
        let worksheet = schema::worksheets::table
            .find(format!("{}", id))
            .get_result::<models::QueryableWorksheet>(&*conn)?;

        let tasks_query = schema::tasks_in_worksheets::table
            .filter(schema::tasks_in_worksheets::columns::worksheet_id.eq(format!("{}", id)))
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
        Ok(sheet) => Box::new(Ok(HttpResponse::Ok().json(sheet)).into_future()),
        Err(e) => match e {
            diesel::result::Error::NotFound => {
                Box::new(Ok(HttpResponse::NotFound().finish()).into_future())
            }
            e => {
                log::error!("Error looking for worksheet: {:?}", e);
                Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
            }
        },
    }
}

fn update_worksheet(
    req: HttpRequest,
    id: web::Path<Uuid>,
    json: web::Json<models::Worksheet>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    match conn.transaction::<(), diesel::result::Error, _>(|| {
        let worksheet = json.into_inner();

        // update worksheet
        diesel::update(schema::worksheets::table.find(format!("{}", id)))
            .set(models::QueryableWorksheet::from_worksheet(
                worksheet.clone(),
            ))
            .execute(&*conn)?;

        // update which tasks belong to worksheet
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
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => {
            log::error!("Couldn't update worksheet: {:?}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}

fn delete_worksheet(
    req: HttpRequest,
    id: web::Path<Uuid>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let extensions = req.extensions();
    let conn = extensions
        .get::<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>>()
        .unwrap();

    match conn.transaction::<(), diesel::result::Error, _>(|| {
        let uuid = id.into_inner();
        diesel::delete(
            schema::tasks_in_worksheets::table
                .filter(schema::tasks_in_worksheets::worksheet_id.eq(uuid.to_string())),
        )
        .execute(&*conn)?;
        diesel::delete(schema::worksheets::table.find(format!("{}", uuid))).execute(&*conn)?;
        Ok(())
    }) {
        Ok(_) => Box::new(Ok(HttpResponse::Ok().finish()).into_future()),
        Err(e) => {
            log::error!("Couldn't delete worksheet: {:?}", e);
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
