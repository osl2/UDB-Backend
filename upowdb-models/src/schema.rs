table! {
    access (user_id, object_id) {
        user_id -> Text,
        object_id -> Text,
    }
}

table! {
    aliases (alias) {
        alias -> Text,
        object_id -> Text,
        object_type -> Integer,
    }
}

table! {
    courses (id) {
        id -> Text,
        name -> Text,
        description -> Nullable<Text>,
    }
}

table! {
    databases (id) {
        id -> Text,
        name -> Text,
        content -> Text,
    }
}

table! {
    subtasks (id) {
        id -> Text,
        instruction -> Text,
        is_solution_verifiable -> Bool,
        is_solution_visible -> Bool,
        content -> Text,
    }
}

table! {
    subtasks_in_tasks (subtask_id, task_id) {
        subtask_id -> Text,
        task_id -> Text,
        position -> Integer,
    }
}

table! {
    tasks (id) {
        id -> Text,
        database_id -> Text,
    }
}

table! {
    tasks_in_worksheets (task_id, worksheet_id) {
        task_id -> Text,
        worksheet_id -> Text,
        position -> Nullable<Integer>,
    }
}

table! {
    users (id) {
        id -> Text,
        name -> Text,
        password_hash -> Text,
        salt -> Text,
    }
}

table! {
    worksheets (id) {
        id -> Text,
        name -> Nullable<Text>,
        is_online -> Bool,
        is_solution_online -> Bool,
    }
}

table! {
    worksheets_in_courses (worksheet_id, course_id) {
        worksheet_id -> Text,
        course_id -> Text,
        position -> Nullable<Integer>,
    }
}

joinable!(subtasks_in_tasks -> subtasks (subtask_id));
joinable!(subtasks_in_tasks -> tasks (task_id));
joinable!(tasks -> databases (database_id));
joinable!(tasks_in_worksheets -> tasks (task_id));
joinable!(tasks_in_worksheets -> worksheets (worksheet_id));
joinable!(worksheets_in_courses -> courses (course_id));
joinable!(worksheets_in_courses -> worksheets (worksheet_id));

allow_tables_to_appear_in_same_query!(
    access,
    aliases,
    courses,
    databases,
    subtasks,
    subtasks_in_tasks,
    tasks,
    tasks_in_worksheets,
    users,
    worksheets,
    worksheets_in_courses,
);
