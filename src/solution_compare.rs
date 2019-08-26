use crate::models::{
    MCSolutionResult, PlaintextSolutionResult, SQLSolutionResult, Solution, SolutionResult, Subtask,
    Content
};

pub fn rows_equal(row1: &[String], row2: &[String]) -> bool {
    if row1.len() != row2.len() {
        return false;
    }
    for (item1, item2) in row1.iter().zip(row2.iter()) {
        if item1 != item2 {
            return false;
        }
    }
    true
}

pub fn compare_solutions(student_solution: Solution, subtask: Subtask) -> SolutionResult {

    let teacher_solution = subtask.content.get_solution().unwrap();

    match (student_solution, teacher_solution) {
        (Solution::SQL(student_solution), Solution::SQL(teacher_solution)) => {
            // indices of rows in teacher solution that have been found in student solution:
            let mut visited_teacher_rows: Vec<usize> = Vec::new();
            // rows in student solution that are not present in teacher solution:
            let mut wrong_rows: Vec<Vec<String>> = Vec::new();
            // rows in teacher solution that are not present in student solution:
            let mut missed_rows: Vec<Vec<String>> = Vec::new();

            // if row order matters, comparison is straightforward
            match subtask.content {
                Content::SQL { row_order_matters: true, .. } => {
                    for (student_row, teacher_row) in student_solution.rows.iter().zip(teacher_solution.rows.iter()) {
                        if !rows_equal(student_row, teacher_row) {
                            wrong_rows.push(student_row.clone());
                        }
                    }
                    return SolutionResult::SQL(SQLSolutionResult {
                        correct: wrong_rows.is_empty(),
                        wrong_rows,
                        missed_rows,
                    })
                },
                _ => {}
            }
            // row order does not matter:

            // find wrong rows in student solution
            for student_row in student_solution.rows.iter() {
                let mut found_pair = false;
                for (index, teacher_row) in teacher_solution.rows.iter().enumerate() {
                    if rows_equal(student_row, teacher_row) {
                        visited_teacher_rows.push(index);
                        found_pair = true;
                        break;
                    }
                }
                if !found_pair {
                    wrong_rows.push(student_row.clone());
                }
            }

            // find rows in teacher solution that the student missed
            let mut i = 0;
            for (index, teacher_row) in teacher_solution.rows.iter().enumerate() {
                if visited_teacher_rows.len() <= i {
                    missed_rows.push(teacher_row.clone());
                } else if visited_teacher_rows.contains(&index) {
                    // skip row if already found pair in previous loop.
                    i += 1;
                } else {
                    missed_rows.push(teacher_row.clone());
                }
            }

            SolutionResult::SQL(SQLSolutionResult {
                correct: missed_rows.is_empty() && wrong_rows.is_empty(),
                missed_rows,
                wrong_rows,
            })
        }
        (
            Solution::MultipleChoice(student_solution),
            Solution::MultipleChoice(teacher_solution),
        ) => {
            let mut correct = true;
            let mut wrong_choices: Vec<i64> = Vec::new();
            let mut missed_choices: Vec<i64> = Vec::new();

            // because there are less elements, this is the naive approach of
            // the algorithm finding wrong and missed rows

            for student_choice in student_solution.correct_positions.iter() {
                if !teacher_solution.correct_positions.contains(student_choice) {
                    correct = false;
                    wrong_choices.push(student_choice.clone());
                }
            }

            for teacher_choice in teacher_solution.correct_positions.iter() {
                if !student_solution.correct_positions.contains(teacher_choice) {
                    correct = false;
                    missed_choices.push(teacher_choice.clone());
                }
            }

            SolutionResult::MultipleChoice(MCSolutionResult {
                correct,
                wrong_choices,
                missed_choices,
            })
        }
        (Solution::Text(student_solution), Solution::Text(teacher_solution)) => {
            SolutionResult::Text(PlaintextSolutionResult {
                correct: student_solution.text.eq(&teacher_solution.text),
                correct_answer: teacher_solution.text.clone(),
            })
        }
        _ => SolutionResult::Error("Solution types dont match".to_string()), // solution types not the same
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{SQLSolution, Solution};
    use crate::solution_compare;
    use upowdb_models::models::{Subtask, Content, SolutionResult, AllowedSQL};

    /// Creates a mock sqltask with given options
    fn new_sqltask(row_order_matters: bool, solution: SQLSolution) -> Subtask {
        Subtask {
            id: "".to_string(),
            instruction: "".to_string(),
            is_solution_verifiable: false,
            is_solution_visible: false,
            content: Content::SQL {
                is_point_and_click_allowed: false,
                row_order_matters,
                allowed_sql: AllowedSQL::ALL,
                solution: Some(solution),
            },
        }
    }

    /// helper method to convert Vec<&str> into Vec<String>
    fn as_strings(v: Vec<&str>) -> Vec<String> {
        v.into_iter().map(String::from).collect()
    }

    #[test]
    fn rows_equal() {
        let row1 = as_strings(vec!["a", "b", "c"]);
        let row2 = as_strings(vec!["a", "b", "c"]);
        let row3 = as_strings(vec!["x", "y", "z"]);
        assert_eq!(solution_compare::rows_equal(&row1, &row2), true);
        assert_eq!(solution_compare::rows_equal(&row2, &row3), false);
    }

    #[test]
    fn comparing_sql_solution_correct() {
        let solution = Solution::SQL(SQLSolution {
            query: "SELECT * FROM users;".to_string(),
            columns: as_strings(vec!["id", "name", "age"]),
            rows: vec![
                as_strings(vec!["1", "Alice", "21"]),
                as_strings(vec!["2", "Bob", "32"]),
                as_strings(vec!["6", "Bill", "33"]),
                as_strings(vec!["3", "Charlie", "5"]),
            ],
        });

        let subtask = new_sqltask(
            false,
            SQLSolution {
                query: "SELECT * FROM users;".to_string(),
                columns: as_strings(vec!["id", "name", "age"]),
                rows: vec![
                    as_strings(vec!["3", "Charlie", "5"]),
                    as_strings(vec!["1", "Alice", "21"]),
                    as_strings(vec!["6", "Bill", "33"]),
                    as_strings(vec!["2", "Bob", "32"]),
                ],
        });

        match solution_compare::compare_solutions(solution, subtask) {
            SolutionResult::SQL(result) => {
                assert_eq!(result.correct, true);
            },
            _ => panic!("Wrong solution type"),
        }
    }

    #[test]
    fn comparing_sql_solution_wrong() {
        let solution = Solution::SQL(SQLSolution {
            query: "SELECT * FROM users;".to_string(),
            columns: as_strings(vec!["id", "name", "age"]),
            rows: vec![
                as_strings(vec!["2", "Bob", "32"]),
                as_strings(vec!["3", "Charlie", "5"]),
                as_strings(vec!["4", "David", "21"]),
                as_strings(vec!["1", "Alice", "21"]),
            ],
        });

        let subtask = new_sqltask(
            false,
            SQLSolution {
                query: "SELECT * FROM users;".to_string(),
                columns: as_strings(vec!["id", "name", "age"]),
                rows: vec![
                    as_strings(vec!["1", "Alice", "21"]),
                    as_strings(vec!["2", "Bob", "32"]),
                    as_strings(vec!["6", "Bill", "33"]),
                    as_strings(vec!["3", "Charlie", "5"]),
                ],
            });

        match solution_compare::compare_solutions(solution, subtask) {
            SolutionResult::SQL(result) => {
                assert_eq!(result.correct, false);
                assert_eq!(result.missed_rows, vec![as_strings(vec!["6", "Bill", "33"])]);
                assert_eq!(result.wrong_rows, vec![as_strings(vec!["4", "David", "21"])]);
            },
            _ => panic!("Wrong solution type"),
        }
    }

    #[test]
    fn comparing_sql_solution_order_correct() {
        let solution = Solution::SQL(SQLSolution {
            query: "SELECT * FROM users;".to_string(),
            columns: as_strings(vec!["id", "name", "age"]),
            rows: vec![
                as_strings(vec!["1", "Alice", "21"]),
                as_strings(vec!["2", "Bob", "32"]),
                as_strings(vec!["6", "Bill", "33"]),
                as_strings(vec!["3", "Charlie", "5"]),
            ],
        });

        let subtask = new_sqltask(
            true,
            SQLSolution {
                query: "SELECT * FROM users;".to_string(),
                columns: as_strings(vec!["id", "name", "age"]),
                rows: vec![
                    as_strings(vec!["1", "Alice", "21"]),
                    as_strings(vec!["2", "Bob", "32"]),
                    as_strings(vec!["6", "Bill", "33"]),
                    as_strings(vec!["3", "Charlie", "5"]),
                ],
            });

        match solution_compare::compare_solutions(solution, subtask) {
            SolutionResult::SQL(result) => {
                assert_eq!(result.correct, true);
            },
            _ => panic!("Wrong solution type"),
        }
    }

    #[test]
    fn comparing_sql_solution_order_wrong() {
        let solution = Solution::SQL(SQLSolution {
            query: "SELECT * FROM users;".to_string(),
            columns: as_strings(vec!["id", "name", "age"]),
            rows: vec![
                as_strings(vec!["1", "Alice", "21"]),
                as_strings(vec!["2", "Bob", "32"]),
                as_strings(vec!["3", "Charlie", "5"]),
                as_strings(vec!["4", "David", "21"]),
            ],
        });

        let subtask = new_sqltask(
            true,
            SQLSolution {
                query: "SELECT * FROM users;".to_string(),
                columns: as_strings(vec!["id", "name", "age"]),
                rows: vec![
                    as_strings(vec!["1", "Alice", "21"]),
                    as_strings(vec!["2", "Bob", "32"]),
                    as_strings(vec!["6", "Bill", "33"]),
                    as_strings(vec!["3", "Charlie", "5"]),
                ],
            });

        match solution_compare::compare_solutions(solution, subtask) {
            SolutionResult::SQL(result) => {
                assert_eq!(result.correct, false);
            },
            _ => panic!("Wrong solution type"),
        }
    }
}
