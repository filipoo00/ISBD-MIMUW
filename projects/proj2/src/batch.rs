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
}