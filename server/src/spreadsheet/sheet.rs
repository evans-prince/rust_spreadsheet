use super::region::Region;
use super::cell::Cell;
use std::collections::*;

#[derive(Debug)]
pub struct Sheet {
   pub regions : HashMap<(usize, usize), Region> ,
}

impl Sheet {

    pub fn new() -> Self {
        Self {
            regions : HashMap::new(),
        }
    }

    fn get_region_coords(i : usize , j: usize ) -> (usize ,usize){
        (i/256 , j/256 )
    }

    fn get_local_coords(i : usize , j: usize ) -> (usize ,usize){
        (i%256 , j%256 )
    }

    pub fn set_cell(&mut self , i : usize , j: usize , c : Cell ) {

        let (region_r , region_c ) = Self::get_region_coords(i,j);
        let (local_r , local_c ) = Self::get_local_coords(i,j);

        let region = self.regions.entry((region_r , region_c)).or_insert_with(Region::new);
        region.set_cell(local_r,local_c,c);

    }

    pub fn get_cell(&self, i : usize , j:usize) -> Option<&Cell>{
        
        let (region_r , region_c ) = Self::get_region_coords(i,j);
        let (local_r , local_c ) = Self::get_local_coords(i,j);

        self.regions.get(&(region_r , region_c ))?.get_cell(local_r, local_c )

    }

}   
