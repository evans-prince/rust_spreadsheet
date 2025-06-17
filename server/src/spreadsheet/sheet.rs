use super::region::Region;
use std::collections::*;

pub struct Sheet {
   pub regions : HashMap<(usize, usize), Region> ,
}
