use upowdb_models::models;

fn main() -> Result<(), Box<std::error::Error>> {
    let username = "elite_admin";
    let password = "2342";

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::ORIGIN, reqwest::header::HeaderValue::from_str("https://staging.upowdb.xyz")?);

    let client = reqwest::Client::builder().default_headers(headers.clone()).build()?;

    dbg!(client.post("https://api.staging.upowdb.xyz/api/v1/account/register")
    .json(&models::Account {username: username.to_string(), password: password.to_string()}).send()?);

    let token : serde_json::Value = serde_json::from_str(&dbg!(client.post("https://api.staging.upowdb.xyz/api/v1/account/login")
        .basic_auth(username, Some(password)).send()?.text()?))?;
    let token = token["token"].as_str().unwrap();
    headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(&format!("Bearer {}", dbg!(token)))?);
    let client = reqwest::Client::builder().default_headers(headers).build()?;

    let sql_subtask = models::Subtask {
        id: "".to_string(),
        instruction: "select all genres".to_string(),
        is_solution_verifiable: false,
        is_solution_visible: false,
        content: models::Content::SQL {
            is_point_and_click_allowed: true,
            row_order_matters: false,
            allowed_sql: models::AllowedSQL::ALL,
            solution: Some(models::SQLSolution {
                query: "".to_string(),
                columns: vec!["Name".to_string()],
                rows: vec![
                    vec!["Rock".to_string(),],
                    vec!["Jazz".to_string(),],
                    vec!["Metal".to_string(),],
                    vec!["Alternative & Punk".to_string(),],
                    vec!["Rock And Roll".to_string(),],
                    vec!["Blues".to_string(),],
                    vec!["Latin".to_string(),],
                    vec!["Reggae".to_string(),],
                    vec!["Pop".to_string(),],
                    vec!["Soundtrack".to_string(),],
                    vec!["Bossa Nova".to_string(),],
                    vec!["Easy Listening".to_string(),],
                    vec!["Heavy Metal".to_string(),],
                    vec!["R&B/Soul".to_string(),],
                    vec!["Electronica/Dance".to_string(),],
                    vec!["World".to_string(),],
                    vec!["Hip Hop/Rap".to_string(),],
                    vec!["Science Fiction".to_string(),],
                    vec!["TV Shows".to_string(),],
                    vec!["Sci Fi & Fantasy".to_string(),],
                    vec!["Drama".to_string(),],
                    vec!["Comedy".to_string(),],
                    vec!["Alternative".to_string(),],
                    vec!["Classical".to_string(),],
                    vec!["Opera".to_string(),],
                ]

            })
        }
    };
    let instruction_subtask = models::Subtask {
        id: "".to_string(),
        instruction: "this will instruct you to do something".to_string(),
        is_solution_verifiable: false,
        is_solution_visible: false,
        content: models::Content::Instruction,
    };
    let plain_subtask = models::Subtask {
        id: "".to_string(),
        instruction: "What color is #00FF00".to_string(),
        is_solution_verifiable: false,
        is_solution_visible: false,
        content: models::Content::Plaintext {
            solution: Some(models::PlaintextSolution {
                text: "Green".to_string()
            })
        }
    };
    let multiple_choice_subtask = models::Subtask {
        id: "".to_string(),
        instruction: "Who made these tasks?".to_string(),
        is_solution_verifiable: false,
        is_solution_visible: false,
        content: models::Content::MC {
            answer_options: vec![
                "Lisa".to_string(),
                "David".to_string(),
                "Marcus".to_string(),
                "Grüntier".to_string(),
                "Svenja".to_string(),
            ],
            solution: Some(models::MCSolution {
                correct_positions: vec![3i64]
            })
        }
    };
    let mut subtask_ids : Vec<String> = vec![];

    subtask_ids.push(dbg!(client.post("https://api.staging.upowdb.xyz/api/v1/subtasks").json(&sql_subtask).send()?).text()?);
    subtask_ids.push(dbg!(client.post("https://api.staging.upowdb.xyz/api/v1/subtasks").json(&instruction_subtask).send()?).text()?);
    subtask_ids.push(dbg!(client.post("https://api.staging.upowdb.xyz/api/v1/subtasks").json(&plain_subtask).send()?).text()?);
    subtask_ids.push(dbg!(client.post("https://api.staging.upowdb.xyz/api/v1/subtasks").json(&multiple_choice_subtask).send()?).text()?);
    println!("{:?}", subtask_ids);

    let task = models::Task {
        database_id: "eef5b7b0-7c28-4856-a9ae-3959fcc5b988".to_string(),
        id: "".to_string(),
        subtasks: subtask_ids,
    };

    let task_id = client.post("https://api.staging.upowdb.xyz/api/v1/tasks").json(&task).send()?.text()?;

    let worksheet = models::Worksheet {
        id: "".to_string(),
        is_online: true,
        is_solution_online: true,
        name: Some("testsheet".to_string()),
        tasks: vec![task_id],

    };

    let worksheet_id = client.post("https://api.staging.upowdb.xyz/api/v1/worksheets").json(&worksheet).send()?.text()?;

    let course = models::Course {
        id: "".to_string(),
        name: "testcourse".to_string(),
        description: Some("description of this course".to_string()),
        worksheets: vec![worksheet_id],
    };

    let course_id = client.post("https://api.staging.upowdb.xyz/api/v1/courses").json(&course).send()?.text()?;

    let alias_req = models::AliasRequest {
        object_id: course_id,
        object_type: models::ObjectType::COURSE,
    };

    let alias = client.post("https://api.staging.upowdb.xyz/api/v1/alias").json(&alias_req).send()?.text()?;

    println!("{}", alias);
    Ok(())
}
