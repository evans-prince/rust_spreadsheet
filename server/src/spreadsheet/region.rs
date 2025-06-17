// region will be 4 x 4 i.e, 16 blocks each covering 256 x 256 cells

use super::block::Block;
use std::collections::*;

pub struct Region {
   pub blocks : HashMap<(usize, usize ), Block>,
}
