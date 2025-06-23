
#[derive(Debug, Clone)]
pub struct Cell {
    pub value : Option<String>,
    pub formula : Option<String>,
}

impl Cell {
    pub fn new() -> Cell{
        Self{
            value : None,
            formula : None,
        }
    }
}
