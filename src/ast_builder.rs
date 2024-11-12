use std::collections::HashMap;

#[derive(Clone)]
enum StatementType {
    Select,
    Update,
    Delete,
    Insert,
}

struct QueryBlock {
    pub query_part: String,
    pub secondary_part: Option<Box<QueryBlock>>,
    pub statement_type: StatementType,
}

struct Model {
    pub name: String,
    pub fields: Option<Vec<String>>,
}

trait SecondaryPart {
    fn values(self, values: &Vec<String>) -> Self;
    fn where_clause(self, where_clause: &String) -> Self;
    fn set(self, arguments: &HashMap<String, String>) -> Self;
}

impl SecondaryPart for QueryBlock {
    fn values(mut self, values: &Vec<String>) -> Self {
        let values_str = format!("VALUES ({})", values.join(", "));
        let query_block = QueryBlock {
            query_part: values_str,
            secondary_part: None,
            statement_type: self.statement_type.clone(),
        };
        let latest_node = traverse_to_the_latest_node(&mut self);
        latest_node.secondary_part = Some(Box::new(query_block));
        self
    }

    fn set(mut self, arguments: &HashMap<String, String>) -> Self {
        let set_clause = arguments
            .iter()
            .map(|(key, value)| format!("{} = {}", key, value))
            .collect::<Vec<String>>()
            .join(", ");
        let set_clause_str = format!("SET {}", set_clause);
        let query_block = QueryBlock {
            query_part: set_clause_str,
            secondary_part: None,
            statement_type: self.statement_type.clone(),
        };
        let latest_node = traverse_to_the_latest_node(&mut self);
        latest_node.secondary_part = Some(Box::new(query_block));
        self
    }

    fn where_clause(mut self, where_clause: &String) -> Self {
        let where_clause_str = format!("WHERE {}", where_clause);

        let query_block = QueryBlock {
            query_part: where_clause_str,
            secondary_part: None,
            statement_type: self.statement_type.clone(),
        };
        let latest_node = traverse_to_the_latest_node(&mut self);
        latest_node.secondary_part = Some(Box::new(query_block));
        self
    }
}

fn select(model: &Model) -> QueryBlock {
    match &model.fields {
        Some(fields) => QueryBlock {
            query_part: format!("SELECT {} FROM {}", fields.join(", "), model.name),
            statement_type: StatementType::Select,
            secondary_part: None,
        },
        None => QueryBlock {
            query_part: format!("SELECT * FROM {}", model.name),
            statement_type: StatementType::Select,
            secondary_part: None,
        },
    }
}

fn update(model: &Model) -> QueryBlock {
    match &model.fields {
        Some(fields) => QueryBlock {
            query_part: format!("UPDATE {}", model.name),
            statement_type: StatementType::Update,
            secondary_part: None,
        },
        None => panic!("Update query must have fields"),
    }
}

fn delete(model: &Model) -> QueryBlock {
    QueryBlock {
        query_part: format!("DELETE FROM {}", model.name),
        statement_type: StatementType::Delete,
        secondary_part: None,
    }
}

fn insert(model: &Model) -> QueryBlock {
    let model = match &model.fields {
        Some(fields) => QueryBlock {
            query_part: format!("INSERT INTO {} ({})", model.name, fields.join(", ")),
            statement_type: StatementType::Insert,
            secondary_part: None,
        },
        None => QueryBlock {
            query_part: format!("INSERT INTO {}", model.name),
            statement_type: StatementType::Insert,
            secondary_part: None,
        },
    };
    model
}

fn traverse_to_the_latest_node(statement: &mut QueryBlock) -> &mut QueryBlock {
    let mut ret = statement;
    while ret.secondary_part.is_some() {
        ret = ret.secondary_part.as_mut().unwrap();
    }
    ret
}

fn compile_statement(statement: &QueryBlock) -> String {
    fn helper(statement: &QueryBlock, acc: String) -> String {
        match &statement.secondary_part {
            Some(next_node) => helper(next_node, acc + " " + &statement.query_part),
            None => acc + " " + &statement.query_part,
        }
    }
    helper(statement, "".to_string())[1..].to_string()
}

mod tests {
    use super::*;

    #[test]
    fn test_select() {
        let model = Model {
            name: "users".to_string(),
            fields: None,
        };
        let query = select(&model);
        assert_eq!(query.query_part, "SELECT * FROM users");
    }

    #[test]
    fn test_select_with_fields() {
        let model = Model {
            name: "users".to_string(),
            fields: Some(vec!["id".to_string(), "name".to_string()]),
        };
        let query = select(&model);
        assert_eq!(query.query_part, "SELECT id, name FROM users");
    }

    #[test]
    fn test_insert_values() {
        let model = Model {
            name: "users".to_string(),
            fields: None,
        };
        let mut query = insert(&model).values(&vec!["1".to_string(), "John".to_string()]);
        let compiled_query = compile_statement(&query);
        assert_eq!(compiled_query, "INSERT INTO users VALUES (1, John)");
    }

    #[test]
    fn test_select_where_clause() {
        let model = Model {
            name: "users".to_string(),
            fields: None,
        };
        let mut binding = select(&model);
        let query = select(&model).where_clause(&"id = 1".to_string());
        let compiled_query = compile_statement(&query);
        assert_eq!(compiled_query, "SELECT * FROM users WHERE id = 1");
    }

    #[test]
    fn test_update_values() {
        let model = Model {
            name: "users".to_string(),
            fields: Some(vec!["name".to_string()]),
        };
        let mut arguments = HashMap::new();
        arguments.insert("name".to_string(), "John".to_string());
        let query = update(&model).set(&arguments);
        let compiled_query = compile_statement(&query);
        assert_eq!(compiled_query, "UPDATE users SET name = John");
    }

    #[test]
    fn test_update_where_clause() {
        let mut arguments = HashMap::new();
        arguments.insert("id".to_string(), "2".to_string());
        let mut where_string = "id = 1".to_string();
        let fields = Some(vec!["id".to_string()]);
        let model = Model {
            name: "users".to_string(),
            fields: fields,
        };
        let mut query = update(&model).set(&arguments).where_clause(&where_string);
        let compiled_query = compile_statement(&query);
        assert_eq!(compiled_query, "UPDATE users SET id = 2 WHERE id = 1");
    }
}
