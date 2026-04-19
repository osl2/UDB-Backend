use crate::models::{MCSolution, PlaintextSolution, SQLSolution, Solution, AllowedSQL};
use diesel::deserialize::FromSql;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use diesel::{deserialize, serialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub enum Content {
    #[serde(rename = "sql")]
    SQL {
        is_point_and_click_allowed: bool,
        row_order_matters: bool,
        #[serde(rename = "allowed_sql")]
        allowed_sql: AllowedSQL,
        solution: Option<SQLSolution>,
    },
    #[serde(rename = "multiple_choice")]
    MC {
        answer_options: Vec<String>,
        solution: Option<MCSolution>,
    },
    #[serde(rename = "plaintext")]
    Plaintext {
        solution: Option<PlaintextSolution>,
    },
    #[serde(rename = "instruction")]
    Instruction,
    Error(String),
}

impl Content {
    pub fn get_solution(&self) -> Option<Solution> {
        match self {
            Content::SQL { solution, .. } => match solution {
                    Some(solution) => Some(Solution::SQL(solution.clone())),
                    None => None,
                },
            Content::MC { solution, .. } => match solution {
                Some(solution) => Some(Solution::MultipleChoice(solution.clone())),
                None => None,
            },
            Content::Plaintext { solution, .. } => match solution {
                Some(solution) => Some(Solution::Text(solution.clone())),
                None => None,
            },
            _ => None,
        }
    }
}

impl FromSql<Text, Sqlite> for Content {
    fn from_sql(bytes: <Sqlite as diesel::backend::Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        match <String as FromSql<Text, Sqlite>>::from_sql(bytes) {
            Ok(json) => match serde_json::from_str(&json) {
                Ok(content) => Ok(content),
                Err(x) => Err(Box::new(x)),
            },
            Err(e) => Err(e),
        }
    }
}

impl ToSql<Text, Sqlite> for Content {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        match serde_json::to_string(self) {
            Ok(json) => {
                out.set_value(diesel::sqlite::SqliteBindValue::from(json));
                Ok(IsNull::No)
            }
            Err(e) => Err(Box::new(e)),
        }
    }
}
