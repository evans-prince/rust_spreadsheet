// Block will be of size 64 x 64

use super::cell::CellValue::{Div0, Empty, Err, Number};
use super::cell::{Cell, CellValue};

#[derive(Clone, Debug)]
pub struct BlockSummary {
    pub sum: f64,
    pub count: usize,         // total cells = 4096
    pub numeric_count: usize, // only numeric cells

    pub min: f64,
    pub max: f64,

    pub has_error: bool,

    pub mean: f64, // for STDEV
    pub m2: f64,   // variance accumulator
}

impl BlockSummary {
    pub fn new() -> Self {
        Self {
            sum: 0.0,
            count: 64 * 64,
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
pub struct Block {
    pub cells: Vec<Cell>, // FLAT VECTOR of 4096 cells
    pub summary: BlockSummary,
}

impl Block {
    pub fn new() -> Self {
        Self {
            cells: vec![Cell::new(); 64 * 64],
            summary: BlockSummary::new(),
        }
    }

    #[inline(always)]
    pub fn idx(row: usize, col: usize) -> usize {
        row * 64 + col
    }

    pub fn set_cell(&mut self, i: usize, j: usize, new_cell: Cell) {
        if i < 64 && j < 64 {
            let index = Self::idx(i, j);

            let old_val = self.cells[index].value.clone();
            let new_val = new_cell.value.clone();

            self.cells[index] = new_cell;
            self.update_summary(old_val, new_val);
        }
    }

    pub fn get_cell(&self, r: usize, c: usize) -> Option<&Cell> {
        if r < 64 && c < 64 {
            self.cells.get(Self::idx(r, c))
        } else {
            None
        }
    }

    fn update_summary(&mut self, old: CellValue, new: CellValue) {
        // ---- REMOVE OLD VALUE CONTRIBUTIONS ----
        match old {
            Number(x_old) => {
                self.summary.sum -= x_old;
                self.summary.numeric_count -= 1;

                // Update stdev (remove contribution)
                let count = self.summary.count as f64;
                if count > 1.0 {
                    let mean = self.summary.mean;
                    let m2 = self.summary.m2;

                    let delta = x_old - mean;
                    let mean_new = (mean * count - x_old) / (count - 1.0);

                    let m2_new = m2 - delta * (x_old - mean_new);

                    self.summary.mean = mean_new;
                    self.summary.m2 = m2_new.max(0.0);
                }
            }
            Err | Div0 => {
                // maybe remove error flag later
            }
            Empty => {}
        }

        // ---- ADD NEW VALUE CONTRIBUTIONS ----
        match new {
            Number(x_new) => {
                self.summary.sum += x_new;
                self.summary.numeric_count += 1;

                if x_new < self.summary.min {
                    self.summary.min = x_new;
                }
                if x_new > self.summary.max {
                    self.summary.max = x_new;
                }

                let count = self.summary.count as f64;
                let mean = self.summary.mean;
                let delta = x_new - mean;
                let mean_new = mean + delta / count;
                let m2_new = self.summary.m2 + delta * (x_new - mean_new);

                self.summary.mean = mean_new;
                self.summary.m2 = m2_new;
            }

            Err | Div0 => {
                self.summary.has_error = true;
            }

            Empty => {}
        }

        // ---- FIX MIN/MAX if old was boundary ----
        if let Number(x_old) = old {
            if x_old == self.summary.min || x_old == self.summary.max {
                self.recompute_min_max();
            }
        }

        // ---- FIX ERROR FLAG ----
        if matches!(old, Err | Div0) && matches!(new, Number(_) | Empty) {
            self.recompute_has_error();
        }
    }

    fn recompute_min_max(&mut self) {
        self.summary.min = f64::INFINITY;
        self.summary.max = f64::NEG_INFINITY;

        for cell in &self.cells {
            if let CellValue::Number(x) = cell.value {
                if x < self.summary.min {
                    self.summary.min = x;
                }
                if x > self.summary.max {
                    self.summary.max = x;
                }
            }
        }

        if self.summary.min == f64::INFINITY {
            self.summary.min = 0.0;
            self.summary.max = 0.0;
        }
    }

    fn recompute_has_error(&mut self) {
        self.summary.has_error = false;

        for cell in &self.cells {
            if let CellValue::Err | CellValue::Div0 = cell.value {
                self.summary.has_error = true;
                return;
            }
        }
    }
}
