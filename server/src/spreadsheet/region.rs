// region will be 4 x 4 i.e, 16 blocks each covering 256 x 256 cells

use super::block::Block;
use super::cell::Cell;
use std::collections::*;

#[derive(Debug)]
pub struct Region {
   pub blocks : HashMap<(usize, usize ), Block>,
}

impl Region{

    pub fn new() -> Self{
        Self{
            blocks : HashMap::new(),
        }
    }

    fn get_block_coords( i : usize , j : usize ) -> (usize , usize) {
        (i / 64 , j / 64 )
    }

    fn get_local_coords( i : usize , j : usize ) -> (usize , usize) {
        (i % 64 , j % 64 )
    }

    pub fn set_cell(&mut self , i : usize , j : usize , c : Cell){

        let (block_r, block_c) = Self::get_block_coords(i,j);
        let (local_r, local_c) = Self::get_local_coords(i,j);

        let block = self.blocks.entry((block_r, block_c)).or_insert_with(Block::new);
        block.set_cell(local_r, local_c, c);

    }

    pub fn get_cell(&self, i: usize, j: usize) -> Option<&Cell> {

        let (block_r, block_c) = Self::get_block_coords(i,j);
        let (local_r, local_c) = Self::get_local_coords(i,j);

        self.blocks.get(&(block_r, block_c))?.get_cell(local_r, local_c)

    }

}
