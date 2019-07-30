use futures::future::{Future, IntoFuture};
use uuid::Uuid;
use actix_web::{
    get, put, post, delete, web, Error, HttpRequest, HttpResponse, Scope
};
use diesel::prelude::*;
use crate::schema;
use crate::models;
use crate::AppData;

pub fn get_scope() -> Scope {
    web::scope("/worksheets")
    .service(get_worksheets)
    .service(create_worksheet)
    .service(get_worksheet)
    .service(update_worksheet)
    .service(delete_worksheet)
}

#[get("")]
fn get_worksheets(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = schema::worksheets::table.inner_join(schema::access::table.on(schema::worksheets::columns::id.eq(schema::access::columns::object_id)))
        .filter(schema::access::columns::user_id.eq(appdata.get_user().to_string()))
        .select((schema::worksheets::columns::id, schema::worksheets::columns::name, schema::worksheets::columns::is_online, schema::worksheets::columns::is_solution_online))
        .load::<models::QueryableWorksheet>(&*conn);

    match query {
        Ok(query_worksheets) => {
            let mut worksheets: Vec<models::Worksheet> = Vec::new();
            for worksheet in query_worksheets {
                let tasks_query = schema::tasks_in_worksheets::table
                    .filter(schema::tasks_in_worksheets::columns::worksheet_id.eq(&worksheet.id))
                    .select((schema::tasks_in_worksheets::columns::task_id))
                    .load::<String>(&*conn);
                worksheets.push(models::Worksheet {
                    id: worksheet.id,
                    name: worksheet.name,
                    is_online: worksheet.is_online,
                    is_solution_online: worksheet.is_solution_online,
                    tasks: tasks_query.ok(),
                });
            }
            Box::new(Ok(HttpResponse::Ok().json(worksheets)).into_future())
        },
        Err(e) => {
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
#[post("")]
fn create_worksheet(req: HttpRequest, json: web::Json<models::Worksheet>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

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
    match diesel::insert_into(schema::access::table)
        .values(models::Access{ user_id: appdata.current_user.to_string(), object_id: worksheet_id.to_string() })
        .execute(&*conn) {
        Ok(result) => {},
        Err(e) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        }
    }

    // set tasks belonging to worksheet
    for task in worksheet.tasks.unwrap() {
        match diesel::insert_into(schema::tasks_in_worksheets::table)
            .values(models::TasksInWorksheet {task_id: task, worksheet_id: worksheet_id.to_string()})
            .execute(&*conn) {
            Ok(result) => {},
            Err(e) => {
                return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
            }
        }
    }

    // insert worksheet object
    match diesel::insert_into(schema::worksheets::table).values(new_worksheet).execute(&*conn) {
        Ok(result) => {
            Box::new(Ok(HttpResponse::Ok().json(worksheet_id)).into_future())
        }
        Err(e) => {
            Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future())
        }
    }
}
#[get("/{id}")]
fn get_worksheet(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = schema::worksheets::table.find(format!("{}", id)).get_result::<models::QueryableWorksheet>(&*conn);

    match query {
        Ok(worksheet) => {
            let tasks_query = schema::tasks_in_worksheets::table
                .filter(schema::tasks_in_worksheets::columns::worksheet_id.eq(format!("{}", id)))
                .select((schema::tasks_in_worksheets::columns::task_id))
                .load::<String>(&*conn);

            Box::new(Ok(HttpResponse::Ok().json(models::Worksheet {
                id: worksheet.id,
                name: worksheet.name,
                is_online: worksheet.is_online,
                is_solution_online: worksheet.is_solution_online,
                tasks: tasks_query.ok(),
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
fn update_worksheet(req: HttpRequest, id: web::Path<Uuid>, json: web::Json<models::Worksheet>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let appdata: &AppData = req.app_data().unwrap();

    let conn = match appdata.get_db_connection(){
        Ok(connection) => connection,
        Err(_) => {
            return Box::new(Ok(HttpResponse::InternalServerError().finish()).into_future());
        },
    };

    let query = diesel::update(schema::worksheets::table.find(format!("{}", id)))
        .set(models::QueryableWorksheet::from_worksheet(json.into_inner()))
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
#[delete("/{id}")]
fn delete_worksheet(req: HttpRequest, id: web::Path<Uuid>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(Ok(HttpResponse::NotImplemented().finish()).into_future())
}
