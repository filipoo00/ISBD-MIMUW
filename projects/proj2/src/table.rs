#[derive(Debug)]
pub enum Column {
    Int64(Vec<i64>),
    Varchar(Vec<String>),
    Boolean(Vec<bool>),
}

impl Column {
    pub fn len(&self) -> usize {
        match self {
            Column::Int64(values) => values.len(),
            Column::Varchar(values) => values.len(),
            Column::Boolean(values) => values.len(),
        }
    }

    pub fn extend(&mut self, other: Column) {
        match (self, other) {
            (Column::Int64(a), Column::Int64(b)) => a.extend(b),
            (Column::Varchar(a), Column::Varchar(b)) => a.extend(b),
            (Column::Boolean(a), Column::Boolean(b)) => a.extend(b),
            _ => panic!("Column types do not match"),
        }
    }
}


pub struct Table {
    pub columns: Vec<Column>
}


pub struct Batch {
    pub columns: Vec<Column>
}