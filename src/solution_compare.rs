use crate::models::{
    MCSolutionResult, PlaintextSolutionResult, SQLSolutionResult, Solution, SolutionResult,
};

pub fn rows_equal(row1: &Vec<String>, row2: &Vec<String>) -> bool {
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

pub fn compare_solutions(student_solution: Solution, teacher_solution: Solution) -> SolutionResult {
    match (student_solution, teacher_solution) {
        (Solution::SQL(student_solution), Solution::SQL(teacher_solution)) => {
            // indices of rows in teacher solution that have been found in student solution:
            let mut visited_teacher_rows: Vec<usize> = Vec::new();
            // rows in student solution that are not present in teacher solution:
            let mut wrong_rows: Vec<Vec<String>> = Vec::new();
            // rows in teacher solution that are not present in student solution:
            let mut missed_rows: Vec<Vec<String>> = Vec::new();

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

            // find rows in teacher solution that student missed
            let mut i = 0;
            for (index, teacher_row) in teacher_solution.rows.iter().enumerate() {
                if visited_teacher_rows.len() <= i {
                    missed_rows.push(teacher_row.clone());
                } else {
                    if index == visited_teacher_rows[i] {
                        // skip row if already found pair in previous loop.
                        // we know that indices in visited_teacher_rows are in order,
                        // so the next one is the next index coming up
                        i += 1;
                    } else {
                        missed_rows.push(teacher_row.clone());
                    }
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

    #[test]
    fn rows_equal() {
        let row1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let row2 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let row3 = vec!["x".to_string(), "y".to_string(), "z".to_string()];
        assert_eq!(solution_compare::rows_equal(&row1, &row2), true);
        assert_eq!(solution_compare::rows_equal(&row2, &row3), false);
    }

    #[test]
    fn comparing() {
        let solution1 = Solution::SQL(SQLSolution {
            query: "SELECT * FROM users;".to_string(),
            columns: vec!["id".to_string(), "name".to_string(), "age".to_string()],
            rows: vec![
                vec!["1".to_string(), "Alice".to_string(), "21".to_string()],
                vec!["2".to_string(), "Bob".to_string(), "32".to_string()],
                vec!["6".to_string(), "Bill".to_string(), "33".to_string()],
                vec!["3".to_string(), "Charlie".to_string(), "5".to_string()],
            ],
        });
        let solution2 = Solution::SQL(SQLSolution {
            query: "SELECT * FROM users;".to_string(),
            columns: vec!["id".to_string(), "name".to_string(), "age".to_string()],
            rows: vec![
                vec!["5".to_string(), "Ball".to_string(), "31".to_string()],
                vec!["2".to_string(), "Bob".to_string(), "32".to_string()],
                vec!["3".to_string(), "Charlie".to_string(), "5".to_string()],
                vec!["4".to_string(), "Dennis".to_string(), "17".to_string()],
            ],
        });

        solution_compare::compare_solutions(solution1, solution2);
    }
}
