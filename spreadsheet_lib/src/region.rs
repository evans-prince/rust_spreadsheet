use super::cell::{Cell, CellValue};
use crate::block::{Block, BlockSummary};

#[derive(Clone, Debug)]
pub struct RegionSummary {
    pub sum: f64,
    pub count: usize,
    pub numeric_count: usize,

    pub min: f64,
    pub max: f64,

    pub has_error: bool,

    pub mean: f64,
    pub m2: f64,
}

impl RegionSummary {
    pub fn new() -> Self {
        Self {
            sum: 0.0,
            count: 256 * 256,
            numeric_count: 0,

            min: f64::INFINITY,
            max: f64::NEG_INFINITY,

            has_error: false,

            mean: 0.0,
            m2: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Region {
    pub blocks: [[Option<Block>; 4]; 4], // <-- FIXED ARRAY
    pub summary: RegionSummary,
}

impl Region {
    pub fn new() -> Self {
        Self {
            blocks: core::array::from_fn(|_| core::array::from_fn(|_| None)),
            summary: RegionSummary::new(),
        }
    }

    #[inline(always)]
    fn get_block_coords(i: usize, j: usize) -> (usize, usize) {
        (i >> 6, j >> 6) // 0..3
    }

    #[inline(always)]
    fn get_local_coords(i: usize, j: usize) -> (usize, usize) {
        (i & 63, j & 63) // 0..63
    }

    pub fn set_cell(&mut self, i: usize, j: usize, cell: Cell) {
        let (br, bc) = Self::get_block_coords(i, j);
        let (lr, lc) = Self::get_local_coords(i, j);

        // lazy init
        if self.blocks[br][bc].is_none() {
            self.blocks[br][bc] = Some(Block::new());
        }

        let block = self.blocks[br][bc].as_mut().unwrap();

        let old_summary = block.summary.clone();
        block.set_cell(lr, lc, cell);
        let new_summary = block.summary.clone();

        self.update_region_summary(&old_summary, &new_summary);
    }

    pub fn get_cell(&self, i: usize, j: usize) -> Option<&Cell> {
        let (br, bc) = Self::get_block_coords(i, j);
        let (lr, lc) = Self::get_local_coords(i, j);

        self.blocks[br][bc].as_ref()?.get_cell(lr, lc)
    }

    fn update_region_summary(&mut self, old: &BlockSummary, new: &BlockSummary) {
        // SUM
        self.summary.sum -= old.sum;
        self.summary.sum += new.sum;

        // numeric count
        self.summary.numeric_count -= old.numeric_count;
        self.summary.numeric_count += new.numeric_count;

        // ERR
        if new.has_error {
            self.summary.has_error = true;
        } else if old.has_error {
            self.recompute_has_error();
        }

        // MIN / MAX
        let need_rescan_min = old.min == self.summary.min;
        let need_rescan_max = old.max == self.summary.max;

        if need_rescan_min || need_rescan_max {
            self.recompute_min_max();
        } else {
            if new.numeric_count > 0 {
                if new.min < self.summary.min {
                    self.summary.min = new.min;
                }
                if new.max > self.summary.max {
                    self.summary.max = new.max;
                }
            }
        }

        // STDEV
        self.merge_stdev(old, new);
    }

    fn recompute_has_error(&mut self) {
        self.summary.has_error = false;

        for row in 0..4 {
            for col in 0..4 {
                if let Some(block) = &self.blocks[row][col] {
                    if block.summary.has_error {
                        self.summary.has_error = true;
                        return;
                    }
                }
            }
        }
    }

    fn recompute_min_max(&mut self) {
        self.summary.min = f64::INFINITY;
        self.summary.max = f64::NEG_INFINITY;

        for row in 0..4 {
            for col in 0..4 {
                if let Some(block) = &self.blocks[row][col] {
                    if block.summary.numeric_count > 0 {
                        if block.summary.min < self.summary.min {
                            self.summary.min = block.summary.min;
                        }
                        if block.summary.max > self.summary.max {
                            self.summary.max = block.summary.max;
                        }
                    }
                }
            }
        }

        if self.summary.min == f64::INFINITY {
            self.summary.min = 0.0;
            self.summary.max = 0.0;
        }
    }

    fn merge_stdev(&mut self, old: &BlockSummary, new: &BlockSummary) {
        // remove old
        {
            let r = &mut self.summary;
            let b = old;

            let total = r.count as f64;
            let delta = b.mean - r.mean;

            let new_mean = (r.mean * total - b.mean * b.count as f64) / (total - b.count as f64);

            let new_m2 =
                r.m2 - b.m2 - delta * delta * (b.count as f64) * ((total - b.count as f64) / total);

            r.mean = new_mean;
            r.m2 = new_m2.max(0.0);
        }

        // add new
        {
            let r = &mut self.summary;
            let b = new;

            let total = r.count as f64;
            let delta = b.mean - r.mean;

            let new_mean = r.mean + delta * (b.count as f64) / total;
            let new_m2 = r.m2 + b.m2 + delta * delta * (r.count as f64) * (b.count as f64) / total;

            r.mean = new_mean;
            r.m2 = new_m2;
        }
    }
}
