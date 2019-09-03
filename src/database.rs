use crate::{models, schema};
use diesel::{
    r2d2::ConnectionManager, Connection, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl,
};
use serde::Deserialize;
use uuid::Uuid;

use lazy_static::lazy_static;

#[cfg(any(
    all(feature = "sqlite", feature = "postgres"),
    all(not(feature = "sqlite"), not(feature = "postgres"))
))]
compile_error!("features `sqlite` and `postgres` are mutually exclusive");

#[cfg(feature = "sqlite")]
pub type DatabaseConnection = diesel::SqliteConnection;

#[cfg(feature = "postgres")]
pub type DatabaseConnection = diesel::PgConnection;

/// The String specifies a filepath or URI for the DB Connection
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum DatabaseConnectionConfig {
    #[serde(rename = "sqlite")]
    SQLiteFile { file: String },
    #[serde(rename = "memory")]
    SQLiteInMemory,
    #[serde(rename = "postgres")]
    Postgres { uri: String },
}

#[derive(Debug)]
pub enum DatabaseConnectionError {
    IncompatibleBuild,
    Diesel(diesel::ConnectionError),
    R2D2(r2d2::Error),
}

impl From<r2d2::Error> for DatabaseConnectionError {
    fn from(error: r2d2::Error) -> DatabaseConnectionError {
        DatabaseConnectionError::R2D2(error)
    }
}

impl From<diesel::ConnectionError> for DatabaseConnectionError {
    fn from(error: diesel::ConnectionError) -> DatabaseConnectionError {
        DatabaseConnectionError::Diesel(error)
    }
}

impl DatabaseConnectionConfig {
    #[cfg(any(feature = "sqlite", feature = "postgres"))]
    pub fn create_database(&self) -> Result<Database, DatabaseConnectionError> {
        match self {
            DatabaseConnectionConfig::SQLiteFile { file } => {
                if cfg!(feature = "sqlite") {
                    Ok(Database::new(r2d2::Pool::builder().max_size(15).build(
                        ConnectionManager::<DatabaseConnection>::new(&file.clone()),
                    )?))
                } else {
                    Err(DatabaseConnectionError::IncompatibleBuild)
                }
            }
            DatabaseConnectionConfig::SQLiteInMemory => {
                if cfg!(feature = "sqlite") {
                    Ok(Database::new(r2d2::Pool::builder().max_size(15).build(
                        ConnectionManager::<DatabaseConnection>::new(":memory:"),
                    )?))
                } else {
                    Err(DatabaseConnectionError::IncompatibleBuild)
                }
            }
            DatabaseConnectionConfig::Postgres { uri } => {
                if cfg!(feature = "postgres") {
                    Ok(Database::new(r2d2::Pool::builder().max_size(15).build(
                        ConnectionManager::<DatabaseConnection>::new(uri),
                    )?))
                } else {
                    Err(DatabaseConnectionError::IncompatibleBuild)
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Database {
    pool: r2d2::Pool<ConnectionManager<DatabaseConnection>>,
}

impl Database {
    pub fn new(pool: r2d2::Pool<ConnectionManager<DatabaseConnection>>) -> Self {
        Self { pool }
    }
    pub fn get_connection(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>, r2d2::Error> {
        self.pool.get()
    }

    pub fn delete_stale_objects(&self, user: Uuid) -> Result<usize, DatabaseError> {
        let mut sum = 0;
        sum += self.delete_stale_worksheets(user)?;
        sum += self.delete_stale_tasks(user)?;
        sum += self.delete_stale_subtasks(user)?;
        Ok(sum)
    }

    pub fn get_subtasks(&self, user: Uuid) -> Result<Vec<models::Subtask>, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(schema::subtasks::table
            .inner_join(
                schema::access::table
                    .on(schema::subtasks::columns::id.eq(schema::access::columns::object_id)),
            )
            .filter(schema::access::columns::user_id.eq(user.to_string()))
            .select((
                schema::subtasks::columns::id,
                schema::subtasks::columns::instruction,
                schema::subtasks::is_solution_visible,
                schema::subtasks::is_solution_verifiable,
                schema::subtasks::content,
            ))
            .load::<models::Subtask>(&*conn)?)
    }

    pub fn create_subtask(
        &self,
        mut subtask: models::Subtask,
        user: Uuid,
    ) -> Result<Uuid, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(conn.transaction::<Uuid, diesel::result::Error, _>(|| {
            // create subtask object
            let id = Uuid::new_v4();
            subtask.id = id.to_string();

            // insert access for user
            diesel::insert_into(schema::access::table)
                .values(models::Access {
                    user_id: user.to_string(),
                    object_id: id.to_string(),
                })
                .execute(&*conn)?;

            // insert subtask object
            diesel::insert_into(schema::subtasks::table)
                .values(subtask)
                .execute(&*conn)?;

            Ok(id)
        })?)
    }

    pub fn get_subtask(&self, subtask_id: Uuid) -> Result<models::Subtask, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(schema::subtasks::table
            .find(subtask_id.to_string())
            .get_result::<models::Subtask>(&*conn)?)
    }

    pub fn update_subtask(
        &self,
        subtask: models::Subtask,
        subtask_id: Uuid,
    ) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        diesel::update(schema::subtasks::table.find(subtask_id.to_string()))
            .set(subtask)
            .execute(&*conn)?;
        Ok(())
    }

    pub fn delete_subtask(&self, subtask_id: Uuid, user: Uuid) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        Ok(conn.transaction::<(), DatabaseError, _>(|| {
            // delete access
            diesel::delete(
                schema::access::table
                    .filter(schema::access::object_id.eq(subtask_id.to_string()))
                    .filter(schema::access::user_id.eq(user.to_string())),
            )
            .execute(&*conn)?;

            // delete subtask
            diesel::delete(schema::subtasks::table.find(subtask_id.to_string())).execute(&*conn)?;
            Ok(())
        })?)
    }

    pub fn delete_stale_subtasks(&self, user: Uuid) -> Result<usize, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(conn.transaction::<usize, DatabaseError, _>(|| {
            let ids = schema::subtasks::table
                .inner_join(
                    schema::access::table.on(schema::access::user_id.eq(schema::subtasks::id)),
                )
                .filter(schema::subtasks::id.eq(user.to_string()))
                .left_join(
                    schema::subtasks_in_tasks::table
                        .on(schema::subtasks::id.eq(schema::subtasks_in_tasks::subtask_id)),
                )
                .filter(schema::subtasks_in_tasks::task_id.is_null())
                .select(schema::subtasks::id)
                .load::<String>(&conn)?;
            let amount = ids.len();
            for id in ids {
                self.delete_subtask(uuid::Uuid::parse_str(&id).unwrap(), user)?
            }
            Ok(amount)
        })?)
    }

    pub fn get_tasks(&self, user: Uuid) -> Result<Vec<models::Task>, DatabaseError> {
        let conn = self.pool.get()?;
        let queryable_tasks = schema::tasks::table
            .inner_join(
                schema::access::table
                    .on(schema::tasks::columns::id.eq(schema::access::columns::object_id)),
            )
            .filter(schema::access::columns::user_id.eq(user.to_string()))
            .select((
                schema::tasks::columns::id,
                schema::tasks::columns::database_id,
                schema::tasks::columns::name,
            ))
            .load::<models::QueryableTask>(&*conn)?;
        let mut tasks = Vec::new();
        for task in queryable_tasks {
            tasks.push(self.queryable_task_to_task(task)?);
        }
        Ok(tasks)
    }

    pub fn queryable_task_to_task(
        &self,
        queryable_task: models::QueryableTask,
    ) -> Result<models::Task, DatabaseError> {
        let conn = self.pool.get()?;
        let subtasks = schema::subtasks_in_tasks::table
            .filter(schema::subtasks_in_tasks::columns::task_id.eq(queryable_task.id.clone()))
            .select(schema::subtasks_in_tasks::columns::subtask_id)
            .order(schema::subtasks_in_tasks::position)
            .load::<String>(&*conn)?;
        Ok(models::Task {
            id: queryable_task.id.clone(),
            name: queryable_task.name,
            database_id: queryable_task.database_id.unwrap_or_else(|| String::new()),
            subtasks,
        })
    }

    pub fn create_task(&self, user: Uuid, mut task: models::Task) -> Result<Uuid, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(conn.transaction::<Uuid, diesel::result::Error, _>(|| {
            // create task object
            let task_id = Uuid::new_v4();
            task.id = task_id.to_string();
            let new_task = models::QueryableTask::from_task(task.clone());

            // insert access for user
            diesel::insert_into(schema::access::table)
                .values(models::Access {
                    user_id: user.to_string(),
                    object_id: task_id.to_string(),
                })
                .execute(&*conn)?;

            // set subtasks belonging to task
            for (position, subtask_id) in task.subtasks.iter().enumerate() {
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
        })?)
    }

    pub fn get_task(&self, task_id: Uuid) -> Result<models::Task, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(self.queryable_task_to_task(
            schema::tasks::table
                .find(task_id.to_string())
                .get_result::<models::QueryableTask>(&*conn)?,
        )?)
    }

    pub fn update_task(
        &self,
        task_id: Uuid,
        mut task: models::Task,
        user: Uuid,
    ) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        Ok(conn.transaction::<(), DatabaseError, _>(|| {
            task.id = task_id.to_string();

            // update tasks
            diesel::update(schema::tasks::table.find(task_id.to_string()))
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
            self.delete_stale_subtasks(user)?;
            Ok(())
        })?)
    }

    pub fn delete_task(&self, task_id: Uuid, user: Uuid) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        Ok(conn.transaction::<(), DatabaseError, _>(|| {
            // delete access
            diesel::delete(
                schema::access::table
                    .filter(schema::access::object_id.eq(task_id.to_string()))
                    .filter(schema::access::user_id.eq(user.to_string())),
            )
            .execute(&*conn)?;

            // delete task
            diesel::delete(schema::tasks::table.find(task_id.to_string())).execute(&*conn)?;

            // delete stale subtasks
            self.delete_stale_subtasks(user)?;

            Ok(())
        })?)
    }

    pub fn delete_stale_tasks(&self, user: Uuid) -> Result<usize, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(conn.transaction::<usize, DatabaseError, _>(|| {
            let ids = schema::tasks::table
                .inner_join(schema::access::table.on(schema::access::user_id.eq(schema::tasks::id)))
                .filter(schema::tasks::id.eq(user.to_string()))
                .left_join(
                    schema::tasks_in_worksheets::table
                        .on(schema::tasks::id.eq(schema::tasks_in_worksheets::task_id)),
                )
                .filter(schema::tasks_in_worksheets::worksheet_id.is_null())
                .select(schema::tasks::id)
                .load::<String>(&conn)?;
            let amount = ids.len();
            for id in ids {
                self.delete_task(uuid::Uuid::parse_str(&id).unwrap(), user)?
            }
            Ok(amount)
        })?)
    }

    pub fn get_worksheets(&self, user: Uuid) -> Result<Vec<models::Worksheet>, DatabaseError> {
        let conn = self.pool.get()?;
        let queryable_worksheets = schema::worksheets::table
            .inner_join(
                schema::access::table
                    .on(schema::worksheets::columns::id.eq(schema::access::columns::object_id)),
            )
            .filter(schema::access::columns::user_id.eq(user.to_string()))
            .select((
                schema::worksheets::columns::id,
                schema::worksheets::columns::name,
                schema::worksheets::columns::is_online,
                schema::worksheets::columns::is_solution_online,
            ))
            .load::<models::QueryableWorksheet>(&*conn)?;
        let mut worksheets = Vec::new();
        for worksheet in queryable_worksheets {
            worksheets.push(self.queryable_worksheet_to_worksheet(worksheet)?);
        }
        Ok(worksheets)
    }

    pub fn queryable_worksheet_to_worksheet(
        &self,
        queryable_worksheet: models::QueryableWorksheet,
    ) -> Result<models::Worksheet, DatabaseError> {
        let conn = self.pool.get()?;
        let tasks = schema::tasks_in_worksheets::table
            .filter(schema::tasks_in_worksheets::columns::worksheet_id.eq(&queryable_worksheet.id))
            .select(schema::tasks_in_worksheets::columns::task_id)
            .order(schema::tasks_in_worksheets::position)
            .load::<String>(&*conn)?;
        Ok(models::Worksheet {
            id: queryable_worksheet.id,
            name: queryable_worksheet.name,
            is_online: queryable_worksheet.is_online,
            is_solution_online: queryable_worksheet.is_solution_online,
            tasks,
        })
    }

    pub fn create_worksheet(
        &self,
        user: Uuid,
        worksheet: models::Worksheet,
    ) -> Result<Uuid, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(conn.transaction::<Uuid, diesel::result::Error, _>(|| {
            // create worksheet object
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
                    user_id: user.to_string(),
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
        })?)
    }

    pub fn get_worksheet(&self, worksheet_id: Uuid) -> Result<models::Worksheet, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(self.queryable_worksheet_to_worksheet(
            schema::worksheets::table
                .find(worksheet_id.to_string())
                .get_result::<models::QueryableWorksheet>(&*conn)?,
        )?)
    }

    pub fn update_worksheet(
        &self,
        worksheet_id: Uuid,
        mut worksheet: models::Worksheet,
        user: Uuid,
    ) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        Ok(conn.transaction::<(), DatabaseError, _>(|| {
            worksheet.id = worksheet_id.to_string();

            // update worksheet
            diesel::update(schema::worksheets::table.find(worksheet_id.to_string()))
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
            let tasks_in_worksheet: Vec<models::TasksInWorksheet> = worksheet
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

            self.delete_stale_tasks(user)?;

            Ok(())
        })?)
    }

    pub fn delete_worksheet(&self, worksheet_id: Uuid, user: Uuid) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        Ok(conn.transaction::<(), DatabaseError, _>(|| {
            // delete access
            diesel::delete(
                schema::access::table
                    .filter(schema::access::object_id.eq(worksheet_id.to_string()))
                    .filter(schema::access::user_id.eq(user.to_string())),
            )
            .execute(&*conn)?;

            // delete worksheet
            diesel::delete(schema::worksheets::table.find(worksheet_id.to_string()))
                .execute(&*conn)?;

            // delete stale tasks
            self.delete_stale_tasks(user)?;

            Ok(())
        })?)
    }

    pub fn delete_stale_worksheets(&self, user: Uuid) -> Result<usize, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(conn.transaction::<usize, DatabaseError, _>(|| {
            let ids = schema::worksheets::table
                .inner_join(
                    schema::access::table.on(schema::access::user_id.eq(schema::worksheets::id)),
                )
                .filter(schema::worksheets::id.eq(user.to_string()))
                .left_join(
                    schema::worksheets_in_courses::table
                        .on(schema::worksheets::id.eq(schema::worksheets_in_courses::worksheet_id)),
                )
                .filter(schema::worksheets_in_courses::course_id.is_null())
                .select(schema::worksheets::id)
                .load::<String>(&conn)?;
            let amount = ids.len();
            for id in ids {
                self.delete_worksheet(uuid::Uuid::parse_str(&id).unwrap(), user)?
            }
            Ok(amount)
        })?)
    }

    pub fn get_user_by_name(&self, name: String) -> Result<models::User, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(schema::users::table
            .filter(schema::users::name.eq(name))
            .get_result::<models::User>(&*conn)?)
    }

    pub fn get_user_by_id(&self, user_id: Uuid) -> Result<models::User, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(schema::users::table
            .find(user_id.to_string())
            .get_result::<models::User>(&*conn)?)
    }

    pub fn update_account(
        &self,
        user_id: Uuid,
        login: models::Account,
    ) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        diesel::update(schema::users::table.find(user_id.to_string()))
            .set(models::User::new(
                login.username,
                login.password,
                Some(user_id),
            ))
            .execute(&*conn)?;
        Ok(())
    }

    pub fn create_account(&self, login: models::Account) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        diesel::insert_into(schema::users::table)
            .values(models::User::new(login.username, login.password, None))
            .execute(&*conn)?;
        Ok(())
    }

    pub fn delete_account(&self, account: Uuid) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        diesel::delete(schema::users::table.find(account.to_string())).execute(&conn)?;
        Ok(())
    }

    pub fn create_alias(&self, alias_req: models::AliasRequest) -> Result<String, DatabaseError> {
        let conn = self.pool.get()?;
        conn.transaction::<String, DatabaseError, _>(|| {
            lazy_static! {
                static ref GENERATOR: crate::alias_generator::AliasGenerator =
                    crate::alias_generator::AliasGenerator::default();
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
                    Ok(_) => return Ok(alias.alias),
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
                        e => Err(e)?,
                    },
                }
            }
            Err(DatabaseError::Other(OtherErrorKind::NoFreeAliases))
        })
    }

    pub fn get_alias_by_uuid(&self, id: Uuid) -> Result<models::Alias, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(schema::aliases::table
            .filter(schema::aliases::object_id.eq(id.to_string()))
            .get_result::<models::Alias>(&*conn)?)
    }

    pub fn get_uuid_by_alias(&self, alias: String) -> Result<models::Alias, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(schema::aliases::table
            .filter(schema::aliases::alias.eq(alias))
            .get_result::<models::Alias>(&*conn)?)
    }

    pub fn get_courses(&self, user: Uuid) -> Result<Vec<models::Course>, DatabaseError> {
        let conn = self.pool.get()?;
        let query_courses = schema::courses::table
            .inner_join(
                schema::access::table
                    .on(schema::courses::columns::id.eq(schema::access::columns::object_id)),
            )
            .filter(schema::access::columns::user_id.eq(user.to_string()))
            .select((
                schema::courses::columns::id,
                schema::courses::columns::name,
                schema::courses::columns::description,
            ))
            .load::<models::QueryableCourse>(&conn)?;
        let mut courses = Vec::new();
        for course in query_courses {
            courses.push(self.queryable_course_to_course(course)?);
        }
        Ok(courses)
    }

    pub fn queryable_course_to_course(
        &self,
        queryable_course: models::QueryableCourse,
    ) -> Result<models::Course, DatabaseError> {
        let conn = self.pool.get()?;
        let worksheets = schema::worksheets_in_courses::table
            .filter(schema::worksheets_in_courses::columns::course_id.eq(&queryable_course.id))
            .select(schema::worksheets_in_courses::columns::worksheet_id)
            .order(schema::worksheets_in_courses::position)
            .load::<String>(&*conn)?;
        Ok(models::Course {
            id: queryable_course.id,
            name: queryable_course.name,
            description: queryable_course.description,
            worksheets,
        })
    }

    pub fn create_course(&self, course: models::Course, user: Uuid) -> Result<Uuid, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(conn.transaction::<Uuid, diesel::result::Error, _>(|| {
            // create course object
            let course_id = Uuid::new_v4();
            let new_course = models::QueryableCourse {
                id: course_id.to_string(),
                name: course.name,
                description: course.description,
            };

            // insert access for user
            diesel::insert_into(schema::access::table)
                .values(models::Access {
                    user_id: user.to_string(),
                    object_id: course_id.to_string(),
                })
                .execute(&*conn)?;

            // insert course object
            diesel::insert_into(schema::courses::table)
                .values(new_course)
                .execute(&*conn)?;

            // set worksheets belonging to course
            for (position, worksheet) in course.worksheets.iter().enumerate() {
                diesel::insert_into(schema::worksheets_in_courses::table)
                    .values(models::WorksheetsInCourse {
                        worksheet_id: worksheet.to_string(),
                        course_id: course_id.to_string(),
                        position: position as i32,
                    })
                    .execute(&*conn)?;
            }

            Ok(course_id)
        })?)
    }

    pub fn get_course(&self, course_id: Uuid) -> Result<models::Course, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(self.queryable_course_to_course(
            schema::courses::table
                .find(course_id.to_string())
                .get_result::<models::QueryableCourse>(&*conn)?,
        )?)
    }

    pub fn update_course(
        &self,
        course_id: Uuid,
        mut course: models::Course,
        user: Uuid,
    ) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        Ok(conn.transaction::<(), DatabaseError, _>(|| {
            // make sure that the course ID in the object is the course we want to modify
            course.id = course_id.to_string();

            // update course
            diesel::update(
                schema::courses::table.filter(schema::courses::id.eq(course_id.to_string())),
            )
            .set(models::QueryableCourse::from_course(course.clone()))
            .execute(&*conn)?;

            // update which worksheets belong to course
            // first delete old ones
            diesel::delete(
                schema::worksheets_in_courses::table
                    .filter(schema::worksheets_in_courses::course_id.eq(course.id.clone())),
            )
            .execute(&*conn)?;

            // then list new ones
            let mut pos = -1;
            let course_id = course.id.clone();
            let worksheets_in_course: Vec<models::WorksheetsInCourse> = course
                .worksheets
                .iter()
                .map(|worksheet_id| {
                    pos += 1;
                    models::WorksheetsInCourse {
                        worksheet_id: worksheet_id.to_string(),
                        course_id: course_id.clone(),
                        position: pos,
                    }
                })
                .collect();

            // finally insert them into the table
            for sheet in worksheets_in_course {
                diesel::insert_into(schema::worksheets_in_courses::table)
                    .values(sheet)
                    .execute(&*conn)?;
            }
            self.delete_stale_worksheets(user)?;
            Ok(())
        })?)
    }

    pub fn delete_course(&self, course_id: Uuid, user: Uuid) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        conn.transaction::<(), DatabaseError, _>(|| {
            diesel::delete(
                schema::access::table.filter(schema::access::object_id.eq(course_id.to_string())),
            )
            .execute(&*conn)?;

            diesel::delete(
                schema::worksheets_in_courses::table
                    .filter(schema::worksheets_in_courses::course_id.eq(course_id.to_string())),
            )
            .execute(&*conn)?;

            diesel::delete(schema::courses::table.find(course_id.to_string())).execute(&*conn)?;
            self.delete_stale_worksheets(user)?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn get_databases(&self, user: Uuid) -> Result<Vec<models::Database>, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(schema::databases::table
            .inner_join(
                schema::access::table
                    .on(schema::databases::columns::id.eq(schema::access::columns::object_id)),
            )
            .filter(schema::access::columns::user_id.eq(user.to_string()))
            .select((
                schema::databases::columns::id,
                schema::databases::columns::name,
                schema::databases::columns::content,
            ))
            .load::<models::Database>(&*conn)?)
    }

    pub fn create_database(
        &self,
        user: Uuid,
        mut database: models::Database,
    ) -> Result<Uuid, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(conn.transaction::<Uuid, diesel::result::Error, _>(|| {
            // create database object
            let id = Uuid::new_v4();
            database.id = id.to_string();

            // insert access for user
            diesel::insert_into(schema::access::table)
                .values(models::Access {
                    user_id: user.to_string(),
                    object_id: id.to_string(),
                })
                .execute(&*conn)?;

            // insert database object
            diesel::insert_into(schema::databases::table)
                .values(database)
                .execute(&*conn)?;

            Ok(id)
        })?)
    }

    pub fn get_database(&self, database_id: Uuid) -> Result<models::Database, DatabaseError> {
        let conn = self.pool.get()?;
        Ok(schema::databases::table
            .find(database_id.to_string())
            .get_result::<models::Database>(&*conn)?)
    }

    pub fn update_database(
        &self,
        database_id: Uuid,
        mut database: models::Database,
    ) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        database.id = database_id.to_string();
        diesel::update(schema::databases::table.find(database_id.to_string()))
            .set(database)
            .execute(&*conn)?;
        Ok(())
    }

    pub fn delete_database(&self, database_id: Uuid) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        diesel::delete(schema::databases::table.find(database_id.to_string())).execute(&*conn)?;
        Ok(())
    }

    // pub fn get_objects(&self, user: Uuid) -> Result<Vec<Object>, DatabaseError> {
    // let conn = self.pool.get()?;

    // }

    // pub fn create_object(&self, user: Uuid, object: Object) -> Result<Uuid, DatabaseError> {
    // let conn = self.pool.get()?;

    // }

    // pub fn get_object(&self, object_id: Uuid) -> Result<Object, DatabaseError> {
    // let conn = self.pool.get()?;

    // }

    // pub fn update_object(&self, object_id: Uuid, object: Object) -> Result<(), DatabaseError> {
    // let conn = self.pool.get()?;

    // }

    // pub fn delete_object(&self, object_id: Uuid) -> Result<(), DatabaseError> {
    // let conn = self.pool.get()?;

    // }
}

#[derive(Debug)]
pub enum DatabaseError {
    R2D2(r2d2::Error),
    Diesel(diesel::result::Error),
    Other(OtherErrorKind),
}

impl From<r2d2::Error> for DatabaseError {
    fn from(error: r2d2::Error) -> Self {
        Self::R2D2(error)
    }
}

impl From<diesel::result::Error> for DatabaseError {
    fn from(error: diesel::result::Error) -> Self {
        Self::Diesel(error)
    }
}

#[derive(Debug)]
pub enum OtherErrorKind {
    NoFreeAliases,
}
