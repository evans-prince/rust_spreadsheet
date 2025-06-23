// Block will be of size 64 x 64 

use super::cell::Cell;
use std::collections::*;

#[derive(Debug)]
pub struct Block {
    pub cells : Vec<Vec<Cell>>,
}

impl Block {

    pub fn new() -> Self{

        let rows = vec![Cell::new() ; 64 ] ;
        let cells = vec![rows ; 64 ];
        Self{cells}

    }

    pub fn set_cell(&mut self , i : usize , j : usize , c : Cell){

        if i<64 && j < 64  {
            self.cells[i][j]=c;
        }

    }

    pub fn get_cell(&self , i : usize , j : usize ) -> Option<&Cell>{

        if i<64 && j < 64  {
            Some(&self.cells[i][j])
        }else{
            None
        }

    }
}
