// Block will be of size 64 x 64 

use super::cell::Cell;
use std::collections::*;

pub struct Block {
    pub cells : Vec<Vec<Cell>>,
}
