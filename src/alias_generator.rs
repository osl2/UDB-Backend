use rand;
use self::rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct AliasGenerator {
    words: Vec<String>,
}
#[allow(dead_code)]
impl AliasGenerator {
    pub fn new() -> Self {
        Self { words: Vec::new() }
    }

    pub fn from_file(filename: &str) -> std::io::Result<Self> {
        let mut words = Vec::new();
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            words.push(line?);
        }
        Ok(Self { words })
    }

    pub fn add_words<'a>(&'a mut self, mut words: Vec<String>) -> &'a mut Self {
        self.words.append(&mut words);
        self
    }

    pub fn generate(&self, n_words: usize) -> String {
        let mut chosen = Vec::new();
        let mut rng = thread_rng();
        for _ in 0..n_words {
            if let Some(word) = self.words.as_slice().choose(&mut rng) {
                chosen.push(word.clone());
            }
        }
        chosen.join("-")
    }
}

impl Default for AliasGenerator {
    fn default() -> AliasGenerator {
        AliasGenerator {
            words: vec![
                "database",
                "table",
                "column",
                "row",
                "integer",
                "text",
                "null",
                "real",
                "blob",
                "date",
                "time",
                "sqlite",
                "postgresql",
                "mariadb",
                "mysql",
                "oracle",
                "nosql",
                "sql",
                "select",
                "from",
                "where",
                "join",
                "group",
                "values",
                "order",
                "limit",
                "having",
                "distinct",
                "all",
                "with",
                "recursive",
                "create",
                "table",
                "as",
                "insert",
                "into",
                "replace",
                "values",
                "default",
                "drop",
                "view",
                "update",
                "rollback",
                "commit",
                "set",
                "offset",
                "abort",
                "delete",
                "primary",
                "foreign",
                "key",
                "by",
            ]
            .iter()
            .map(|w| w.to_string())
            .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::alias_generator::AliasGenerator;

    #[test]
    fn generate() {
        let words = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let generated = AliasGenerator::new().add_words(words.clone()).generate(1);
        assert!(words.contains(&generated));

        let words = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let generated = AliasGenerator::new().add_words(words).generate(2);
        let possible = vec![
            "A-A".to_string(),
            "A-B".to_string(),
            "A-C".to_string(),
            "B-A".to_string(),
            "B-B".to_string(),
            "B-C".to_string(),
            "C-A".to_string(),
            "C-B".to_string(),
            "C-C".to_string(),
        ];
        assert!(possible.contains(&generated));
    }
}
