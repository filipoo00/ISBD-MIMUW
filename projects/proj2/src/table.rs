use std::io;

use crate::constants::BATCH_SIZE;

#[derive(Debug, PartialEq, Clone)]
pub struct VarcharColumn {
    data: String,
    offsets: Vec<u32>,
}

impl VarcharColumn {
    pub fn new() -> VarcharColumn {
        VarcharColumn { data: String::new(), offsets: vec![0] }
    }

    pub fn from_parts(data: String, offsets: Vec<u32>) -> io::Result<Self> {
        if offsets.first() != Some(&0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Varchar offsets must start with 0.",
            ))
        }
        if offsets.windows(2).any(|w| w[0] > w[1]) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Varchar offsets must not decrease.",
            ))
        }
        if *offsets.last().unwrap() as usize != data.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Last varchar offset must equal data length.",
            ))
        }

        Ok(Self { data, offsets })
    }

    pub fn push(&mut self, value: &str) {
        self.data.push_str(value);
        let end = u32::try_from(self.data.len()).expect("Data exceeds 4GB");
        self.offsets.push(end);
    }

    pub fn len(&self) -> usize {
        return self.offsets.len() - 1
    }

    pub fn offsets(&self) -> &[u32] {
        return &self.offsets
    }
    
    pub fn data(&self) -> &str {
        return &self.data
    }
}



#[derive(Debug)]
pub enum Column {
    Int64(Vec<i64>),
    Varchar(VarcharColumn),
    Boolean(Vec<bool>),
}


impl Column {
    pub fn len(&self) -> usize {
        match self {
            Column::Int64(values) => values.len(),
            Column::Varchar(varchar_columns) => varchar_columns.len(),
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
