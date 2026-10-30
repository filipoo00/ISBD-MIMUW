use std::io;

use crate::constants::BATCH_SIZE;

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



pub struct Batch {
    columns: Vec<Column>,
    number_of_rows: usize,
}


impl Batch {
    pub fn new(columns: Vec<Column>) -> io::Result<Self> {
        let number_of_rows = 
            columns.first().map_or(0, |column| column.len());

        if number_of_rows > BATCH_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Batch exceeds BATCH_SIZE.",
            ));
        }

        if columns.iter().any(|column| column.len() != number_of_rows) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "All columns must have the same number of rows.",
            ));
        }

        Ok(Self { columns, number_of_rows })
    }

    pub fn number_of_rows(&self) -> usize {
        self.number_of_rows
    }

    pub fn number_of_cols(&self) -> usize {
        self.columns.len()
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }
}
