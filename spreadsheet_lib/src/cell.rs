#[derive(Debug, Clone)]
pub enum CellValue {
    Number(f64),
    Err,
    Div0,
    Empty, // we will use this for uninitialized cells
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub value: CellValue,
    pub formula: Option<String>,
}

impl Cell {
    pub fn new() -> Cell {
        Self {
            value: CellValue::Empty,
            formula: None,
        }
    }
}
