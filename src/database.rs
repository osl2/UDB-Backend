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

pub type PooledConnection = r2d2::PooledConnection<ConnectionManager<DatabaseConnection>>;

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
    pub fn get_connection(&self) -> Result<PooledConnection, r2d2::Error> {
        self.pool.get()
    }

    fn delete_access(
        conn: &PooledConnection,
        user: Uuid,
        object: Uuid,
    ) -> Result<(), DatabaseError> {
        diesel::delete(
            schema::access::table
                .filter(schema::access::user_id.eq(user.to_string()))
                .filter(schema::access::object_id.eq(object.to_string())),
        )
        .execute(conn)?;
        Ok(())
    }

    fn insert_access(
        conn: &PooledConnection,
        user: Uuid,
        object: Uuid,
    ) -> Result<(), DatabaseError> {
        diesel::insert_into(schema::access::table)
            .values(models::Access {
                user_id: user.to_string(),
                object_id: object.to_string(),
            })
            .execute(conn)?;
        Ok(())
    }

    pub fn delete_stale_objects(&self, user: Uuid) -> Result<usize, DatabaseError> {
        Self::delete_stale_objects_using_conn(&self.pool.get()?, user)
    }

    fn delete_stale_objects_using_conn(
        conn: &PooledConnection,
        user: Uuid,
    ) -> Result<usize, DatabaseError> {
        let mut sum = 0;
        sum += Self::delete_stale_worksheets(conn, user)?;
        sum += Self::delete_stale_tasks(conn, user)?;
        sum += Self::delete_stale_subtasks(conn, user)?;
        log::trace!("Deleted {} stale objects.", sum);
        Ok(sum)
    }

    pub fn get_subtasks(&self, user: Uuid) -> Result<Vec<models::Subtask>, DatabaseError> {
        Self::get_subtasks_using_conn(&self.pool.get()?, user)
    }

    fn get_subtasks_using_conn(
        conn: &PooledConnection,
        user: Uuid,
    ) -> Result<Vec<models::Subtask>, DatabaseError> {
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
            .load::<models::Subtask>(conn)?)
    }

    pub fn create_subtask(
        &self,
        subtask: models::Subtask,
        user: Uuid,
    ) -> Result<Uuid, DatabaseError> {
        Self::create_subtask_using_conn(&self.pool.get()?, subtask, user)
    }

    fn create_subtask_using_conn(
        conn: &PooledConnection,
        mut subtask: models::Subtask,
        user: Uuid,
    ) -> Result<Uuid, DatabaseError> {
        Ok(conn.transaction::<Uuid, DatabaseError, _>(|| {
            // create subtask object
            let id = Uuid::new_v4();
            subtask.id = id.to_string();

            // insert subtask object
            diesel::insert_into(schema::subtasks::table)
                .values(subtask)
                .execute(conn)?;

            Self::insert_access(&conn, user, id)?;

            Ok(id)
        })?)
    }

    pub fn get_subtask(&self, subtask_id: Uuid) -> Result<models::Subtask, DatabaseError> {
        Self::get_subtask_using_conn(&self.pool.get()?, subtask_id)
    }

    fn get_subtask_using_conn(
        conn: &PooledConnection,
        subtask_id: Uuid,
    ) -> Result<models::Subtask, DatabaseError> {
        Ok(schema::subtasks::table
            .find(subtask_id.to_string())
            .get_result::<models::Subtask>(conn)?)
    }

    pub fn update_subtask(
        &self,
        subtask: models::Subtask,
        subtask_id: Uuid,
    ) -> Result<(), DatabaseError> {
        Self::update_subtask_using_conn(&self.pool.get()?, subtask, subtask_id)
    }

    fn update_subtask_using_conn(
        conn: &PooledConnection,
        subtask: models::Subtask,
        subtask_id: Uuid,
    ) -> Result<(), DatabaseError> {
        diesel::update(schema::subtasks::table.find(subtask_id.to_string()))
            .set(subtask)
            .execute(conn)?;
        Ok(())
    }

    pub fn delete_subtask(&self, subtask_id: Uuid, user: Uuid) -> Result<(), DatabaseError> {
        Self::delete_subtask_using_conn(&self.pool.get()?, subtask_id, user)
    }

    fn delete_subtask_using_conn(
        conn: &PooledConnection,
        subtask_id: Uuid,
        user: Uuid,
    ) -> Result<(), DatabaseError> {
        Ok(conn.transaction::<(), DatabaseError, _>(|| {
            Self::delete_access(conn, user, subtask_id)?;

            // delete subtask
            diesel::delete(schema::subtasks::table.find(subtask_id.to_string())).execute(conn)?;
            Ok(())
        })?)
    }

    fn delete_stale_subtasks(conn: &PooledConnection, user: Uuid) -> Result<usize, DatabaseError> {
        Ok(conn.transaction::<usize, DatabaseError, _>(|| {
            let ids = schema::subtasks::table
                .inner_join(
                    schema::access::table.on(schema::access::object_id.eq(schema::subtasks::id)),
                )
                .filter(schema::access::user_id.eq(user.to_string()))
                .left_join(
                    schema::subtasks_in_tasks::table
                        .on(schema::subtasks::id.eq(schema::subtasks_in_tasks::subtask_id)),
                )
                .filter(schema::subtasks_in_tasks::task_id.is_null())
                .select(schema::subtasks::id)
                .load::<String>(conn)?;
            let amount = ids.len();
            for id in ids {
                Self::delete_subtask_using_conn(conn, uuid::Uuid::parse_str(&id).unwrap(), user)?
            }
            log::trace!("Deleted {} stale subtasks.", amount);
            Ok(amount)
        })?)
    }

    pub fn get_tasks(&self, user: Uuid) -> Result<Vec<models::Task>, DatabaseError> {
        Self::get_tasks_using_conn(&self.pool.get()?, user)
    }

    fn get_tasks_using_conn(
        conn: &PooledConnection,
        user: Uuid,
    ) -> Result<Vec<models::Task>, DatabaseError> {
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
            .load::<models::QueryableTask>(conn)?;
        let mut tasks = Vec::new();
        for task in queryable_tasks {
            tasks.push(Self::queryable_task_to_task(conn, task)?);
        }
        Ok(tasks)
    }

    fn queryable_task_to_task(
        conn: &PooledConnection,
        queryable_task: models::QueryableTask,
    ) -> Result<models::Task, DatabaseError> {
        let subtasks = schema::subtasks_in_tasks::table
            .filter(schema::subtasks_in_tasks::columns::task_id.eq(queryable_task.id.clone()))
            .select(schema::subtasks_in_tasks::columns::subtask_id)
            .order(schema::subtasks_in_tasks::position)
            .load::<String>(conn)?;
        Ok(models::Task {
            id: queryable_task.id.clone(),
            name: queryable_task.name,
            database_id: queryable_task.database_id.unwrap_or_else(String::new),
            subtasks,
        })
    }

    pub fn create_task(&self, user: Uuid, task: models::Task) -> Result<Uuid, DatabaseError> {
        Self::create_task_using_conn(&self.pool.get()?, user, task)
    }

    fn create_task_using_conn(
        conn: &PooledConnection,
        user: Uuid,
        mut task: models::Task,
    ) -> Result<Uuid, DatabaseError> {
        Ok(conn.transaction::<Uuid, DatabaseError, _>(|| {
            // create task object
            let task_id = Uuid::new_v4();
            task.id = task_id.to_string();
            let new_task = models::QueryableTask::from_task(task.clone());

            // insert task object
            diesel::insert_into(schema::tasks::table)
                .values(new_task)
                .execute(conn)?;

            Self::insert_access(conn, user, task_id)?;

            // set subtasks belonging to task
            for (position, subtask_id) in task.subtasks.iter().enumerate() {
                diesel::insert_into(schema::subtasks_in_tasks::table)
                    .values(models::SubtasksInTask {
                        subtask_id: subtask_id.to_string(),
                        task_id: task_id.to_string(),
                        position: position as i32,
                    })
                    .execute(conn)?;
            }

            Ok(task_id)
        })?)
    }

    pub fn get_task(&self, task_id: Uuid) -> Result<models::Task, DatabaseError> {
        Self::get_task_using_conn(&self.pool.get()?, task_id)
    }

    fn get_task_using_conn(
        conn: &PooledConnection,
        task_id: Uuid,
    ) -> Result<models::Task, DatabaseError> {
        Ok(Self::queryable_task_to_task(
            &conn,
            schema::tasks::table
                .find(task_id.to_string())
                .get_result::<models::QueryableTask>(conn)?,
        )?)
    }

    pub fn update_task(
        &self,
        task_id: Uuid,
        task: models::Task,
        user: Uuid,
    ) -> Result<(), DatabaseError> {
        Self::update_task_using_conn(&self.pool.get()?, task_id, task, user)
    }

    fn update_task_using_conn(
        conn: &PooledConnection,
        task_id: Uuid,
        mut task: models::Task,
        user: Uuid,
    ) -> Result<(), DatabaseError> {
        Ok(conn.transaction::<(), DatabaseError, _>(|| {
            task.id = task_id.to_string();

            // update tasks
            diesel::update(schema::tasks::table.find(task_id.to_string()))
                .set(models::QueryableTask::from_task(task.clone()))
                .execute(conn)?;

            // update which subtasks belong to this task
            diesel::delete(
                schema::subtasks_in_tasks::table
                    .filter(schema::subtasks_in_tasks::task_id.eq(task.id.clone())),
            )
            .execute(conn)?;

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
                    .execute(conn)?;
            }
            Self::delete_stale_subtasks(&conn, user)?;
            Ok(())
        })?)
    }

    pub fn delete_task(&self, task_id: Uuid, user: Uuid) -> Result<(), DatabaseError> {
        Self::delete_task_using_conn(&self.pool.get()?, task_id, user)
    }

    fn delete_task_using_conn(
        conn: &PooledConnection,
        task_id: Uuid,
        user: Uuid,
    ) -> Result<(), DatabaseError> {
        Ok(conn.transaction::<(), DatabaseError, _>(|| {
            Self::delete_access(conn, user, task_id)?;

            // delete subtask relations
            diesel::delete(
                schema::subtasks_in_tasks::table
                    .filter(schema::subtasks_in_tasks::task_id.eq(task_id.to_string())),
            )
            .execute(conn)?;

            // delete task
            diesel::delete(schema::tasks::table.find(task_id.to_string())).execute(conn)?;

            // delete stale subtasks
            Self::delete_stale_subtasks(conn, user)?;

            Ok(())
        })?)
    }

    fn delete_stale_tasks(conn: &PooledConnection, user: Uuid) -> Result<usize, DatabaseError> {
        Ok(conn.transaction::<usize, DatabaseError, _>(|| {
            let ids = schema::tasks::table
                .inner_join(
                    schema::access::table.on(schema::access::object_id.eq(schema::tasks::id)),
                )
                .filter(schema::access::user_id.eq(user.to_string()))
                .left_join(
                    schema::tasks_in_worksheets::table
                        .on(schema::tasks::id.eq(schema::tasks_in_worksheets::task_id)),
                )
                .filter(schema::tasks_in_worksheets::worksheet_id.is_null())
                .select(schema::tasks::id)
                .load::<String>(conn)?;
            let amount = ids.len();
            for id in ids {
                Self::delete_task_using_conn(conn, uuid::Uuid::parse_str(&id).unwrap(), user)?
            }
            log::trace!("Deleted {} stale tasks.", amount);
            Ok(amount)
        })?)
    }

    pub fn get_worksheets(&self, user: Uuid) -> Result<Vec<models::Worksheet>, DatabaseError> {
        Self::get_worksheets_using_conn(&self.pool.get()?, user)
    }

    fn get_worksheets_using_conn(
        conn: &PooledConnection,
        user: Uuid,
    ) -> Result<Vec<models::Worksheet>, DatabaseError> {
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
            .load::<models::QueryableWorksheet>(conn)?;
        let mut worksheets = Vec::new();
        for worksheet in queryable_worksheets {
            worksheets.push(Self::queryable_worksheet_to_worksheet(conn, worksheet)?);
        }
        Ok(worksheets)
    }

    fn queryable_worksheet_to_worksheet(
        conn: &PooledConnection,
        queryable_worksheet: models::QueryableWorksheet,
    ) -> Result<models::Worksheet, DatabaseError> {
        let tasks = schema::tasks_in_worksheets::table
            .filter(schema::tasks_in_worksheets::columns::worksheet_id.eq(&queryable_worksheet.id))
            .select(schema::tasks_in_worksheets::columns::task_id)
            .order(schema::tasks_in_worksheets::position)
            .load::<String>(conn)?;
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
        Self::create_worksheet_using_conn(&self.pool.get()?, user, worksheet)
    }

    fn create_worksheet_using_conn(
        conn: &PooledConnection,
        user: Uuid,
        worksheet: models::Worksheet,
    ) -> Result<Uuid, DatabaseError> {
        Ok(conn.transaction::<Uuid, DatabaseError, _>(|| {
            // create worksheet object
            let worksheet_id = Uuid::new_v4();
            let new_worksheet = models::QueryableWorksheet {
                id: worksheet_id.to_string(),
                name: worksheet.name,
                is_online: worksheet.is_online,
                is_solution_online: worksheet.is_solution_online,
            };

            // insert worksheet object
            diesel::insert_into(schema::worksheets::table)
                .values(new_worksheet)
                .execute(conn)?;

            Self::insert_access(conn, user, worksheet_id)?;

            // set tasks belonging to worksheet
            for (position, task_id) in worksheet.tasks.iter().enumerate() {
                diesel::insert_into(schema::tasks_in_worksheets::table)
                    .values(models::TasksInWorksheet {
                        task_id: task_id.to_string(),
                        worksheet_id: worksheet_id.to_string(),
                        position: position as i32,
                    })
                    .execute(conn)?;
            }

            Ok(worksheet_id)
        })?)
    }

    pub fn get_worksheet(&self, worksheet_id: Uuid) -> Result<models::Worksheet, DatabaseError> {
        Self::get_worksheet_using_conn(&self.pool.get()?, worksheet_id)
    }

    fn get_worksheet_using_conn(
        conn: &PooledConnection,
        worksheet_id: Uuid,
    ) -> Result<models::Worksheet, DatabaseError> {
        Ok(Self::queryable_worksheet_to_worksheet(
            conn,
            schema::worksheets::table
                .find(worksheet_id.to_string())
                .get_result::<models::QueryableWorksheet>(conn)?,
        )?)
    }

    pub fn update_worksheet(
        &self,
        worksheet_id: Uuid,
        worksheet: models::Worksheet,
        user: Uuid,
    ) -> Result<(), DatabaseError> {
        Self::update_worksheet_using_conn(&self.pool.get()?, worksheet_id, worksheet, user)
    }

    fn update_worksheet_using_conn(
        conn: &PooledConnection,
        worksheet_id: Uuid,
        mut worksheet: models::Worksheet,
        user: Uuid,
    ) -> Result<(), DatabaseError> {
        Ok(conn.transaction::<(), DatabaseError, _>(|| {
            worksheet.id = worksheet_id.to_string();

            // update worksheet
            diesel::update(schema::worksheets::table.find(worksheet_id.to_string()))
                .set(models::QueryableWorksheet::from_worksheet(
                    worksheet.clone(),
                ))
                .execute(conn)?;

            // update which tasks belong to worksheet
            diesel::delete(
                schema::tasks_in_worksheets::table
                    .filter(schema::tasks_in_worksheets::worksheet_id.eq(worksheet.id.clone())),
            )
            .execute(conn)?;
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
                    .execute(conn)?;
            }

            Self::delete_stale_tasks(conn, user)?;

            Ok(())
        })?)
    }

    pub fn delete_worksheet(&self, worksheet_id: Uuid, user: Uuid) -> Result<(), DatabaseError> {
        Self::delete_worksheet_using_conn(&self.pool.get()?, worksheet_id, user)
    }

    fn delete_worksheet_using_conn(
        conn: &PooledConnection,
        worksheet_id: Uuid,
        user: Uuid,
    ) -> Result<(), DatabaseError> {
        Ok(conn.transaction::<(), DatabaseError, _>(|| {
            Self::delete_access(conn, user, worksheet_id)?;

            // update which tasks belong to worksheet
            diesel::delete(
                schema::tasks_in_worksheets::table
                    .filter(schema::tasks_in_worksheets::worksheet_id.eq(worksheet_id.to_string())),
            )
            .execute(conn)?;

            // delete worksheet
            diesel::delete(schema::worksheets::table.find(worksheet_id.to_string()))
                .execute(conn)?;

            // delete stale tasks
            Self::delete_stale_tasks(conn, user)?;

            Ok(())
        })?)
    }

    fn delete_stale_worksheets(
        conn: &PooledConnection,
        user: Uuid,
    ) -> Result<usize, DatabaseError> {
        Ok(conn.transaction::<usize, DatabaseError, _>(|| {
            let ids = schema::worksheets::table
                .inner_join(
                    schema::access::table.on(schema::access::object_id.eq(schema::worksheets::id)),
                )
                .filter(schema::access::user_id.eq(user.to_string()))
                .left_join(
                    schema::worksheets_in_courses::table
                        .on(schema::worksheets::id.eq(schema::worksheets_in_courses::worksheet_id)),
                )
                .filter(schema::worksheets_in_courses::course_id.is_null())
                .select(schema::worksheets::id)
                .load::<String>(conn)?;
            let amount = ids.len();
            for id in ids {
                Self::delete_worksheet_using_conn(conn, uuid::Uuid::parse_str(&id).unwrap(), user)?
            }
            log::trace!("Deleted {} stale worksheets.", amount);
            Ok(amount)
        })?)
    }

    pub fn get_user_by_name(&self, name: String) -> Result<models::User, DatabaseError> {
        Self::get_user_by_name_using_conn(&self.pool.get()?, name)
    }

    fn get_user_by_name_using_conn(
        conn: &PooledConnection,
        name: String,
    ) -> Result<models::User, DatabaseError> {
        Ok(schema::users::table
            .filter(schema::users::name.eq(name))
            .get_result::<models::User>(conn)?)
    }

    pub fn get_user_by_id(&self, user_id: Uuid) -> Result<models::User, DatabaseError> {
        Self::get_user_by_id_using_conn(&self.pool.get()?, user_id)
    }

    fn get_user_by_id_using_conn(
        conn: &PooledConnection,
        user_id: Uuid,
    ) -> Result<models::User, DatabaseError> {
        Ok(schema::users::table
            .find(user_id.to_string())
            .get_result::<models::User>(conn)?)
    }

    pub fn update_account(
        &self,
        user_id: Uuid,
        login: models::Account,
    ) -> Result<(), DatabaseError> {
        Self::update_account_using_conn(&self.pool.get()?, user_id, login)
    }

    fn update_account_using_conn(
        conn: &PooledConnection,
        user_id: Uuid,
        login: models::Account,
    ) -> Result<(), DatabaseError> {
        diesel::update(schema::users::table.find(user_id.to_string()))
            .set(models::User::new(
                login.username,
                login.password,
                Some(user_id),
            ))
            .execute(conn)?;
        Ok(())
    }

    pub fn create_account(&self, login: models::Account) -> Result<(), DatabaseError> {
        Self::create_account_using_conn(&self.pool.get()?, login)
    }

    fn create_account_using_conn(
        conn: &PooledConnection,
        login: models::Account,
    ) -> Result<(), DatabaseError> {
        diesel::insert_into(schema::users::table)
            .values(models::User::new(login.username, login.password, None))
            .execute(conn)?;
        Ok(())
    }

    pub fn delete_account(&self, account: Uuid) -> Result<(), DatabaseError> {
        Self::delete_account_using_conn(&self.pool.get()?, account)
    }

    fn delete_account_using_conn(
        conn: &PooledConnection,
        account: Uuid,
    ) -> Result<(), DatabaseError> {
        //TODO: delete contents here!!
        diesel::delete(schema::users::table.find(account.to_string())).execute(conn)?;
        Ok(())
    }

    pub fn create_alias(&self, alias_req: models::AliasRequest) -> Result<String, DatabaseError> {
        Self::create_alias_using_conn(&self.pool.get()?, alias_req)
    }

    fn create_alias_using_conn(
        conn: &PooledConnection,
        alias_req: models::AliasRequest,
    ) -> Result<String, DatabaseError> {
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
                    .execute(conn)
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
        Self::get_alias_by_uuid_using_conn(&self.pool.get()?, id)
    }

    pub fn get_alias_by_uuid_using_conn(
        conn: &PooledConnection,
        id: Uuid,
    ) -> Result<models::Alias, DatabaseError> {
        Ok(schema::aliases::table
            .filter(schema::aliases::object_id.eq(id.to_string()))
            .get_result::<models::Alias>(conn)?)
    }

    pub fn get_uuid_by_alias(&self, alias: String) -> Result<models::Alias, DatabaseError> {
        Self::get_uuid_by_alias_using_conn(&self.pool.get()?, alias)
    }

    fn get_uuid_by_alias_using_conn(
        conn: &PooledConnection,
        alias: String,
    ) -> Result<models::Alias, DatabaseError> {
        Ok(schema::aliases::table
            .filter(schema::aliases::alias.eq(alias))
            .get_result::<models::Alias>(conn)?)
    }

    pub fn get_courses(&self, user: Uuid) -> Result<Vec<models::Course>, DatabaseError> {
        Self::get_courses_using_conn(&self.pool.get()?, user)
    }

    fn get_courses_using_conn(
        conn: &PooledConnection,
        user: Uuid,
    ) -> Result<Vec<models::Course>, DatabaseError> {
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
            .load::<models::QueryableCourse>(conn)?;
        let mut courses = Vec::new();
        for course in query_courses {
            courses.push(Self::queryable_course_to_course(conn, course)?);
        }
        Ok(courses)
    }

    fn queryable_course_to_course(
        conn: &PooledConnection,
        queryable_course: models::QueryableCourse,
    ) -> Result<models::Course, DatabaseError> {
        let worksheets = schema::worksheets_in_courses::table
            .filter(schema::worksheets_in_courses::columns::course_id.eq(&queryable_course.id))
            .select(schema::worksheets_in_courses::columns::worksheet_id)
            .order(schema::worksheets_in_courses::position)
            .load::<String>(conn)?;
        Ok(models::Course {
            id: queryable_course.id,
            name: queryable_course.name,
            description: queryable_course.description,
            worksheets,
        })
    }

    pub fn create_course(&self, course: models::Course, user: Uuid) -> Result<Uuid, DatabaseError> {
        Self::create_course_using_conn(&self.pool.get()?, course, user)
    }

    fn create_course_using_conn(
        conn: &PooledConnection,
        course: models::Course,
        user: Uuid,
    ) -> Result<Uuid, DatabaseError> {
        Ok(conn.transaction::<Uuid, DatabaseError, _>(|| {
            // create course object
            let course_id = Uuid::new_v4();
            let new_course = models::QueryableCourse {
                id: course_id.to_string(),
                name: course.name,
                description: course.description,
            };

            // insert course object
            diesel::insert_into(schema::courses::table)
                .values(new_course)
                .execute(conn)?;

            Self::insert_access(conn, user, course_id)?;

            // set worksheets belonging to course
            for (position, worksheet) in course.worksheets.iter().enumerate() {
                diesel::insert_into(schema::worksheets_in_courses::table)
                    .values(models::WorksheetsInCourse {
                        worksheet_id: worksheet.to_string(),
                        course_id: course_id.to_string(),
                        position: position as i32,
                    })
                    .execute(conn)?;
            }

            Ok(course_id)
        })?)
    }

    pub fn get_course(&self, course_id: Uuid) -> Result<models::Course, DatabaseError> {
        Self::get_course_using_conn(&self.pool.get()?, course_id)
    }

    fn get_course_using_conn(
        conn: &PooledConnection,
        course_id: Uuid,
    ) -> Result<models::Course, DatabaseError> {
        Ok(Self::queryable_course_to_course(
            conn,
            schema::courses::table
                .find(course_id.to_string())
                .get_result::<models::QueryableCourse>(conn)?,
        )?)
    }

    pub fn update_course(
        &self,
        course_id: Uuid,
        course: models::Course,
        user: Uuid,
    ) -> Result<(), DatabaseError> {
        Self::update_course_using_conn(&self.pool.get()?, course_id, course, user)
    }

    fn update_course_using_conn(
        conn: &PooledConnection,
        course_id: Uuid,
        mut course: models::Course,
        user: Uuid,
    ) -> Result<(), DatabaseError> {
        Ok(conn.transaction::<(), DatabaseError, _>(|| {
            // make sure that the course ID in the object is the course we want to modify
            course.id = course_id.to_string();

            // update course
            diesel::update(
                schema::courses::table.filter(schema::courses::id.eq(course_id.to_string())),
            )
            .set(models::QueryableCourse::from_course(course.clone()))
            .execute(conn)?;

            // update which worksheets belong to course
            // first delete old ones
            diesel::delete(
                schema::worksheets_in_courses::table
                    .filter(schema::worksheets_in_courses::course_id.eq(course.id.clone())),
            )
            .execute(conn)?;

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
                    .execute(conn)?;
            }
            Self::delete_stale_worksheets(conn, user)?;
            Ok(())
        })?)
    }

    pub fn delete_course(&self, course_id: Uuid, user: Uuid) -> Result<(), DatabaseError> {
        Self::delete_course_using_conn(&self.pool.get()?, course_id, user)
    }

    fn delete_course_using_conn(
        conn: &PooledConnection,
        course_id: Uuid,
        user: Uuid,
    ) -> Result<(), DatabaseError> {
        conn.transaction::<(), DatabaseError, _>(|| {
            Self::delete_access(conn, user, course_id)?;

            diesel::delete(
                schema::worksheets_in_courses::table
                    .filter(schema::worksheets_in_courses::course_id.eq(course_id.to_string())),
            )
            .execute(conn)?;

            diesel::delete(schema::courses::table.find(course_id.to_string())).execute(conn)?;
            Self::delete_stale_worksheets(conn, user)?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn get_databases(&self, user: Uuid) -> Result<Vec<models::Database>, DatabaseError> {
        Self::get_databases_using_conn(&self.pool.get()?, user)
    }

    fn get_databases_using_conn(
        conn: &PooledConnection,
        user: Uuid,
    ) -> Result<Vec<models::Database>, DatabaseError> {
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
            .load::<models::Database>(conn)?)
    }

    pub fn create_database(
        &self,
        user: Uuid,
        database: models::Database,
    ) -> Result<Uuid, DatabaseError> {
        Self::create_database_using_conn(&self.pool.get()?, user, database)
    }

    fn create_database_using_conn(
        conn: &PooledConnection,
        user: Uuid,
        mut database: models::Database,
    ) -> Result<Uuid, DatabaseError> {
        Ok(conn.transaction::<Uuid, DatabaseError, _>(|| {
            // create database object
            let id = Uuid::new_v4();
            database.id = id.to_string();

            // insert database object
            diesel::insert_into(schema::databases::table)
                .values(database)
                .execute(conn)?;

            Self::insert_access(conn, user, id)?;

            Ok(id)
        })?)
    }

    pub fn get_database(&self, database_id: Uuid) -> Result<models::Database, DatabaseError> {
        Self::get_database_using_conn(&self.pool.get()?, database_id)
    }

    fn get_database_using_conn(
        conn: &PooledConnection,
        database_id: Uuid,
    ) -> Result<models::Database, DatabaseError> {
        Ok(schema::databases::table
            .find(database_id.to_string())
            .get_result::<models::Database>(conn)?)
    }

    pub fn update_database(
        &self,
        database_id: Uuid,
        database: models::Database,
    ) -> Result<(), DatabaseError> {
        Self::update_database_using_conn(&self.pool.get()?, database_id, database)
    }

    fn update_database_using_conn(
        conn: &PooledConnection,
        database_id: Uuid,
        mut database: models::Database,
    ) -> Result<(), DatabaseError> {
        database.id = database_id.to_string();
        diesel::update(schema::databases::table.find(database_id.to_string()))
            .set(database)
            .execute(conn)?;
        Ok(())
    }

    pub fn delete_database(&self, database_id: Uuid) -> Result<(), DatabaseError> {
        Self::delete_database_using_conn(&self.pool.get()?, database_id)
    }

    fn delete_database_using_conn(
        conn: &PooledConnection,
        database_id: Uuid,
    ) -> Result<(), DatabaseError> {
        diesel::delete(schema::databases::table.find(database_id.to_string())).execute(conn)?;
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
