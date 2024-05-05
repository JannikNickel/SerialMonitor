use std::fmt::Display;

#[derive(Debug)]
pub enum ParseError {
    ColumnMismatch(usize, usize)
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default)]
pub struct SerialParser {
    columns: usize
}

impl SerialParser {
    pub fn new() -> Self {
        Self {
            columns: 0
        }
    }

    pub fn reset(&mut self) {
        self.columns = 0;
    }

    pub fn columns(&self) -> usize {
        self.columns
    }

    pub fn parse_values(&mut self, line: &str) -> Result<Vec<f64>, ParseError> {
        let mut columns = 0;
        let mut res: Vec<f64> = Vec::new();
        for col in line.split(',') {
            if let Ok(v) = col.trim().parse::<f64>() {
                columns += 1;
                res.push(v);
            }
        }

        if self.columns != 0 && self.columns != columns {
            return Err(ParseError::ColumnMismatch(self.columns, columns));
        }
        self.columns = columns;

        Ok(res)
    }
}
