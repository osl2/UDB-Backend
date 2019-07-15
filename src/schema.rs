table! {
    access (user_id, object_id) {
        user_id -> Text,
        object_id -> Text,
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
        instruction -> Nullable<Text>,
        is_solution_verifiable -> Nullable<Bool>,
        content -> Nullable<Binary>,
        task_id -> Nullable<Text>,
    }
}

table! {
    tasks (id) {
        id -> Text,
        worksheet_id -> Nullable<Text>,
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
        course_id -> Nullable<Text>,
    }
}

joinable!(subtasks -> tasks (task_id));
joinable!(tasks -> worksheets (worksheet_id));
joinable!(worksheets -> courses (course_id));

allow_tables_to_appear_in_same_query!(
    access,
    courses,
    databases,
    subtasks,
    tasks,
    users,
    worksheets,
);
