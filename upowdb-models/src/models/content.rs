use crate::models::{MCSolution, PlaintextSolution, SQLSolution, Solution, AllowedSQL};
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression)]
#[sql_type = "Text"]
pub enum Content {
    #[serde(rename = "sql")]
    SQL {
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

//special to and from sql traits because content gets saved as json

impl<DB> FromSql<Text, DB> for Content
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        match String::from_sql(bytes) {
            Ok(json) => match serde_json::from_str(&json) {
                Ok(content) => Ok(content),
                Err(x) => Err(Box::new(x)),
            },
            Err(e) => Err(e),
        }
    }
}

impl<DB> ToSql<Text, DB> for Content
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        match serde_json::to_string(self) {
            Ok(json) => out
                .write_fmt(format_args!("{}", json))
                .map(|_| IsNull::No)
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>),
            Err(e) => Err(Box::new(e)),
        }
    }
}
