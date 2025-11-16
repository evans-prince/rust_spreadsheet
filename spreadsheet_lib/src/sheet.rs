/*Sheet
 └── Regions (HashMap, lazy)
       └── Blocks (4×4 fixed array, lazy)
             └── Cells (64×64 flat vector)
*/
use super::cell::{Cell, CellValue};
use super::region::Region;
use crate::block::Block;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub enum RangeFunction {
    Sum,
    Avg,
    Min,
    Max,
    Stdev,
}

#[derive(Debug)]
pub struct Sheet {
    pub regions: HashMap<(usize, usize), Region>,
    // Top-left position of the 10×10 viewport
    pub cursor_row: usize,
    pub cursor_col: usize,

    pub output_enabled: bool,
    pub last_status: String,
    pub last_time: f64,
}

impl Sheet {
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
            // start viewport at (0, 0)
            cursor_row: 0,
            cursor_col: 0,

            output_enabled: true,
            last_status: "ok".to_string(),
            last_time: 0.0,
        }
    }

    fn get_region_coords(i: usize, j: usize) -> (usize, usize) {
        (i / 256, j / 256)
    }

    fn get_local_coords(i: usize, j: usize) -> (usize, usize) {
        (i % 256, j % 256)
    }

    pub fn set_cell(&mut self, i: usize, j: usize, c: Cell) {
        let (region_r, region_c) = Self::get_region_coords(i, j);
        let (local_r, local_c) = Self::get_local_coords(i, j);

        let region = self
            .regions
            .entry((region_r, region_c))
            .or_insert_with(Region::new);
        region.set_cell(local_r, local_c, c);
    }

    pub fn get_cell(&self, i: usize, j: usize) -> Option<&Cell> {
        let (region_r, region_c) = Self::get_region_coords(i, j);
        let (local_r, local_c) = Self::get_local_coords(i, j);

        self.regions
            .get(&(region_r, region_c))?
            .get_cell(local_r, local_c)
    }

    fn col_to_name(mut col: usize) -> String {
        // convert 0-based column index to A, B, ... Z, AA, AB ...
        let mut chars = Vec::new();

        while {
            let rem = col % 26;
            chars.push((b'A' + rem as u8) as char);
            if col < 26 {
                false
            } else {
                col = col / 26 - 1;
                true
            }
        } {}

        chars.reverse();
        chars.into_iter().collect()
    }

    pub fn display(&self) {
        let start_r = self.cursor_row;
        let start_c = self.cursor_col;

        let end_r = start_r + 10;
        let end_c = start_c + 10;

        // Print column headers (width = 10)
        print!("     "); // space for row numbers
        for c in start_c..end_c {
            let name = Self::col_to_name(c);
            print!("{:<10}", name); // left-aligned width=10
        }
        println!();

        // Print rows
        for r in start_r..end_r {
            // Row number left-aligned
            print!("{:<5}", r + 1);

            for c in start_c..end_c {
                if let Some(cell) = self.get_cell(r, c) {
                    match &cell.value {
                        CellValue::Number(x) => {
                            let s = format!("{}", x);
                            print!("{:<10}", s);
                        }
                        CellValue::Err => print!("{:<10}", "ERR"),
                        CellValue::Div0 => print!("{:<10}", "DIV0"),
                        CellValue::Empty => print!("{:<10}", "0"),
                    }
                } else {
                    print!("{:<10}", "0");
                }
            }
            println!();
        }

        println!();
    }

    pub fn print_status(&self) {
        print!("[{:.3}] ({}) > ", self.last_time, self.last_status);
    }

    pub fn scroll_up(&mut self) {
        self.cursor_row = self.cursor_row.saturating_sub(10);
    }

    pub fn scroll_down(&mut self) {
        self.cursor_row = self.cursor_row.saturating_add(10);
    }

    pub fn scroll_left(&mut self) {
        self.cursor_col = self.cursor_col.saturating_sub(10);
    }

    pub fn scroll_right(&mut self) {
        self.cursor_col = self.cursor_col.saturating_add(10);
    }

    /// Handle assignment like "A1=23" or "A1=B1" or "A1=1+2" or "A1=-3*-2"
    /// `cleaned` should be input with ALL whitespace removed (execute_command already does that).
    pub fn handle_assignment(&mut self, cleaned: &str) -> Result<(), String> {
        // cleaned might be mixed-case; make RHS handling case-insensitive for cell names
        let parts: Vec<&str> = cleaned.split('=').collect();
        if parts.len() != 2 {
            return Err("invalid_command".into());
        }

        let lhs = parts[0];
        let rhs = parts[1];

        // parse left cell (case-insensitive inside parse_cell)
        let (row, col) = match Self::parse_cell(lhs) {
            Some(x) => x,
            None => return Err("invalid_cell".into()),
        };

        // If RHS contains an operator -> evaluate simple expression
        if let Some((_op_pos, _op)) = Self::find_operator(rhs) {
            match self.parse_simple_expression(rhs) {
                Ok(result) => {
                    let cell = Cell {
                        value: CellValue::Number(result),
                        formula: None,
                    };
                    self.set_cell(row, col, cell);
                    return Ok(());
                }
                Err(e) => {
                    let cell = match e.as_str() {
                        "DIV0" => Cell {
                            value: CellValue::Div0,
                            formula: None,
                        },
                        _ => Cell {
                            value: CellValue::Err,
                            formula: None,
                        },
                    };
                    self.set_cell(row, col, cell);
                    return Err(e);
                }
            }
        }

        // CASE: range functions like SUM(A1:A10), MAX(A1:B3), STDEV(A2:A20)
        if let Some(func_result) = self.handle_range_function(rhs) {
            match func_result {
                Ok(value) => {
                    let cell = Cell {
                        value,
                        formula: None,
                    };
                    self.set_cell(row, col, cell);
                    return Ok(());
                }
                Err(e) => {
                    let cell = Cell {
                        value: CellValue::Err,
                        formula: None,
                    };
                    self.set_cell(row, col, cell);
                    return Err(e);
                }
            }
        }

        // If RHS is a numeric literal
        if let Ok(num) = rhs.parse::<f64>() {
            let cell = Cell {
                value: CellValue::Number(num),
                formula: None,
            };
            self.set_cell(row, col, cell);
            return Ok(());
        }

        // If RHS is a cell reference
        if let Some((rr, cc)) = Self::parse_cell(rhs) {
            if let Some(src) = self.get_cell(rr, cc) {
                // copy the CellValue as-is
                let cell = Cell {
                    value: src.value.clone(),
                    formula: None,
                };
                self.set_cell(row, col, cell);
                return Ok(());
            } else {
                return Err("invalid_cell".into());
            }
        }

        // otherwise invalid value for now (we're numeric-only)
        Err("invalid_value".into())
    }

    pub fn execute_command(&mut self, input: &str) -> bool {
        // timing start
        let start = Instant::now();
        self.last_status = "ok".into();

        // 1. remove ALL whitespace
        let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();

        // 2. convert EVERYTHING to uppercase for command keywords (we still use parse_cell which uppercases)
        let upper = cleaned.to_uppercase();

        // --- CONTROL COMMANDS ---
        if upper == "Q" {
            self.last_status = "quit".into();
            self.last_time = start.elapsed().as_secs_f64();
            return false;
        } else if upper == "W" {
            self.scroll_up();
        } else if upper == "S" {
            self.scroll_down();
        } else if upper == "A" {
            self.scroll_left();
        } else if upper == "D" {
            self.scroll_right();
        } else if upper == "U" {
            // TODO: undo
        } else if upper == "R" {
            // TODO: redo
        } else if upper == "DISABLE_OUTPUT" {
            self.output_enabled = false;
        } else if upper == "ENABLE_OUTPUT" {
            self.output_enabled = true;
        } else if upper.starts_with("SCROLL_TO") {
            let cell_str = &upper[9..];
            if let Some((r, c)) = Self::parse_cell(cell_str) {
                self.cursor_row = r;
                self.cursor_col = c;
            } else {
                self.last_status = "invalid_cell".into();
            }
        } else if cleaned.contains('=') {
            // assignment / formula handling
            match self.handle_assignment(&cleaned) {
                Ok(()) => {}
                Err(e) => self.last_status = e,
            }
        } else {
            self.last_status = "invalid_command".into();
        }

        // timing end
        self.last_time = start.elapsed().as_secs_f64();
        true
    }

    pub fn parse_cell(cell: &str) -> Option<(usize, usize)> {
        // 1. Remove whitespace and uppercase everything
        let cleaned: String = cell.chars().filter(|c| !c.is_whitespace()).collect();
        let s = cleaned.to_uppercase();

        if s.is_empty() {
            return None;
        }

        // 2. Split into letters and digits
        let mut col = String::new();
        let mut row = String::new();
        let mut seen_digit = false;

        for ch in s.chars() {
            if ch.is_ascii_alphabetic() {
                if seen_digit {
                    // letters after digits = invalid
                    return None;
                }
                col.push(ch);
            } else if ch.is_ascii_digit() {
                seen_digit = true;
                row.push(ch);
            } else {
                return None; // invalid char
            }
        }

        // 3. Validate
        if col.is_empty() || row.is_empty() {
            return None;
        }

        // 4. Convert row (1-based → 0-based)
        let row_num = row.parse::<usize>().ok()?;
        if row_num == 0 {
            return None;
        }
        let row_idx = row_num - 1;

        // 5. Convert column letters to index (BASE-26)
        let mut col_idx = 0usize;
        for ch in col.chars() {
            col_idx = col_idx * 26 + ((ch as u8 - b'A' + 1) as usize);
        }
        if col_idx == 0 {
            return None;
        }
        col_idx -= 1; // convert to 0-based

        Some((row_idx, col_idx))
    }

    fn find_operator(expr: &str) -> Option<(usize, char)> {
        let chars: Vec<char> = expr.chars().collect();

        for i in 0..chars.len() {
            let ch = chars[i];

            if ch == '+' || ch == '-' || ch == '*' || ch == '/' {
                // RULE 1: operator cannot be first character (unary)
                if i == 0 {
                    continue;
                }

                // RULE 2: if previous char is operator → this '-' is unary
                let prev = chars[i - 1];
                if prev == '+' || prev == '-' || prev == '*' || prev == '/' {
                    continue;
                }

                // This is a real binary operator
                return Some((i, ch));
            }
        }

        None
    }

    fn value_to_number(&self, s: &str) -> Result<f64, String> {
        // try numeric literal (handles negative and decimals)
        if let Ok(num) = s.parse::<f64>() {
            return Ok(num);
        }

        // cell reference
        if let Some((r, c)) = Self::parse_cell(s) {
            if let Some(cell) = self.get_cell(r, c) {
                match cell.value {
                    CellValue::Number(x) => Ok(x),
                    CellValue::Div0 => Err("DIV0".into()),
                    CellValue::Err => Err("ERR".into()),
                    CellValue::Empty => Ok(0.0),
                }
            } else {
                Err("invalid_cell".into())
            }
        } else {
            Err("invalid_value".into())
        }
    }

    fn parse_simple_expression(&self, expr: &str) -> Result<f64, String> {
        // 1. find operator
        let (op_pos, op) = match Self::find_operator(expr) {
            Some(x) => x,
            None => return Err("invalid_expression".to_string()),
        };

        // 2. split into LHS and RHS
        let left = &expr[..op_pos];
        let right = &expr[op_pos + 1..];

        if left.is_empty() || right.is_empty() {
            return Err("invalid_expression".into());
        }

        // 3. convert both to numbers
        let a = self.value_to_number(left)?;
        let b = self.value_to_number(right)?;

        // 4. evaluate
        let result = match op {
            '+' => a + b,
            '-' => a - b,
            '*' => a * b,
            '/' => {
                if b == 0.0 {
                    return Err("DIV0".into());
                }
                a / b
            }
            _ => return Err("invalid_operator".into()),
        };

        Ok(result)
    }

    fn parse_range(arg: &str) -> Result<((usize, usize), (usize, usize)), String> {
        let parts: Vec<&str> = arg.split(':').collect();
        if parts.len() != 2 {
            return Err("invalid_range".into());
        }

        let (start, end) = (parts[0], parts[1]);

        let (r1, c1) = Self::parse_cell(start).ok_or("invalid_range")?;
        let (r2, c2) = Self::parse_cell(end).ok_or("invalid_range")?;

        // ranges must be forward (A1:A10 OK, A10:A1 NOT OK)
        if r2 < r1 || c2 < c1 {
            return Err("invalid_range".into());
        }

        Ok(((r1, c1), (r2, c2)))
    }

    pub fn eval_range(
        &self,
        r1: usize,
        c1: usize,
        r2: usize,
        c2: usize,
        func: RangeFunction,
    ) -> CellValue {
        let mut sum = 0.0;
        let mut total_cells = 0usize; // for AVG
        let mut numeric_min = f64::INFINITY;
        let mut numeric_max = f64::NEG_INFINITY;
        let mut any_numeric = false;

        // STDEV accumulators
        let mut mean = 0.0;
        let mut m2 = 0.0;

        // iterate over regions
        for rr in r1 / 256..=r2 / 256 {
            for rc in c1 / 256..=c2 / 256 {
                match self.regions.get(&(rr, rc)) {
                    None => {
                        // region is empty → treat as all EMPTY (=0 for sum, ignored for min/max)
                        total_cells += 256 * 256;
                        continue;
                    }
                    Some(region) => {
                        // check full-region coverage
                        let region_r_start = rr * 256;
                        let region_r_end = region_r_start + 255;
                        let region_c_start = rc * 256;
                        let region_c_end = region_c_start + 255;

                        let fully_inside = region_r_start >= r1
                            && region_r_end <= r2
                            && region_c_start >= c1
                            && region_c_end <= c2;

                        if fully_inside {
                            // ERR propagation
                            if region.summary.has_error {
                                return CellValue::Err;
                            }

                            // SUM
                            sum += region.summary.sum;

                            // COUNT for AVG
                            total_cells += region.summary.count;

                            // MIN/MAX
                            if region.summary.numeric_count > 0 {
                                any_numeric = true;
                                numeric_min = numeric_min.min(region.summary.min);
                                numeric_max = numeric_max.max(region.summary.max);
                            }

                            // STDEV merging
                            Self::merge_stdev(
                                &mut mean,
                                &mut m2,
                                region.summary.mean,
                                region.summary.m2,
                                region.summary.count as f64,
                                total_cells as f64 - region.summary.count as f64, // previous count before adding this region
                            );

                            continue;
                        }

                        // PARTIAL REGION → go to blocks
                        self.eval_partial_region(
                            region,
                            rr,
                            rc,
                            r1,
                            c1,
                            r2,
                            c2,
                            func,
                            &mut sum,
                            &mut total_cells,
                            &mut numeric_min,
                            &mut numeric_max,
                            &mut any_numeric,
                            &mut mean,
                            &mut m2,
                        );
                    }
                }
            }
        }

        // Now produce final output
        match func {
            RangeFunction::Sum => CellValue::Number(sum),
            RangeFunction::Avg => {
                if total_cells == 0 {
                    return CellValue::Err;
                }
                CellValue::Number(sum / total_cells as f64)
            }
            RangeFunction::Min => {
                if !any_numeric {
                    return CellValue::Err;
                }
                CellValue::Number(numeric_min)
            }
            RangeFunction::Max => {
                if !any_numeric {
                    return CellValue::Err;
                }
                CellValue::Number(numeric_max)
            }
            RangeFunction::Stdev => {
                if total_cells == 0 {
                    return CellValue::Err;
                }
                // population stdev (divide by N), sqrt(m2 / N)
                CellValue::Number((m2 / total_cells as f64).sqrt())
            }
        }
    }

    fn eval_partial_region(
        &self,
        region: &Region,
        rr: usize,
        rc: usize,
        r1: usize,
        c1: usize,
        r2: usize,
        c2: usize,
        func: RangeFunction,
        sum: &mut f64,
        total_cells: &mut usize,
        numeric_min: &mut f64,
        numeric_max: &mut f64,
        any_numeric: &mut bool,
        mean: &mut f64,
        m2: &mut f64,
    ) {
        for br in 0..4 {
            for bc in 0..4 {
                // array region.blocks is fixed 4x4; use as_ref() to borrow
                if let Some(block) = region.blocks[br][bc].as_ref() {
                    // block coordinates in sheet
                    let block_r_start = rr * 256 + br * 64;
                    let block_r_end = block_r_start + 63;
                    let block_c_start = rc * 256 + bc * 64;
                    let block_c_end = block_c_start + 63;

                    let fully_inside = block_r_start >= r1
                        && block_r_end <= r2
                        && block_c_start >= c1
                        && block_c_end <= c2;

                    if fully_inside {
                        // ERR propagation
                        if block.summary.has_error {
                            // poison result
                            *sum = 0.0;
                            return;
                        }

                        // add block-level summary
                        *sum += block.summary.sum;
                        *total_cells += block.summary.count;

                        if block.summary.numeric_count > 0 {
                            *any_numeric = true;
                            *numeric_min = (*numeric_min).min(block.summary.min);
                            *numeric_max = (*numeric_max).max(block.summary.max);
                        }

                        Self::merge_stdev(
                            mean,
                            m2,
                            block.summary.mean,
                            block.summary.m2,
                            block.summary.count as f64,
                            *total_cells as f64 - block.summary.count as f64, // previous count
                        );
                    } else {
                        // partial block: fallback to cell-level
                        self.eval_partial_block(
                            block,
                            block_r_start,
                            block_c_start,
                            r1,
                            c1,
                            r2,
                            c2,
                            func,
                            sum,
                            total_cells,
                            numeric_min,
                            numeric_max,
                            any_numeric,
                            mean,
                            m2,
                        );
                    }
                } else {
                    // block does not exist = all empty cells
                    *total_cells += 64 * 64;
                }
            }
        }
    }

    fn eval_partial_block(
        &self,
        block: &Block,
        br_start: usize,
        bc_start: usize,
        r1: usize,
        c1: usize,
        r2: usize,
        c2: usize,
        _func: RangeFunction,
        sum: &mut f64,
        total_cells: &mut usize,
        numeric_min: &mut f64,
        numeric_max: &mut f64,
        any_numeric: &mut bool,
        mean: &mut f64,
        m2: &mut f64,
    ) {
        for i in 0..64 {
            for j in 0..64 {
                let r = br_start + i;
                let c = bc_start + j;

                if r < r1 || r > r2 || c < c1 || c > c2 {
                    continue;
                }

                *total_cells += 1;
                let idx = Block::idx(i, j);

                match block.cells[idx].value {
                    CellValue::Number(x) => {
                        *sum += x;
                        *any_numeric = true;
                        *numeric_min = (*numeric_min).min(x);
                        *numeric_max = (*numeric_max).max(x);

                        // update stdev
                        Self::welford_add(mean, m2, *total_cells as f64, x);
                    }
                    CellValue::Empty => {
                        // SUM: +0
                        // MIN/MAX: ignored
                        // STDEV: treated as zero contribution
                    }
                    CellValue::Err | CellValue::Div0 => {
                        // any error in the partial block poisons the range
                        return;
                    }
                }
            }
        }
    }

    /// Merge a block/population (b_count, b_mean, b_m2) into current accumulators mean/m2.
    /// `prev_total` is the number of items already represented by `*mean`/`*m2` BEFORE adding the block.
    fn merge_stdev(
        mean: &mut f64,
        m2: &mut f64,
        b_mean: f64,
        b_m2: f64,
        b_count: f64,
        prev_total: f64,
    ) {
        // if no items previously in accumulator, copy block
        if prev_total <= 0.0 {
            if b_count > 0.0 {
                *mean = b_mean;
                *m2 = b_m2;
            }
            return;
        }

        // if block empty, nothing to do
        if b_count <= 0.0 {
            return;
        }

        // combine two populations:
        // population A: n1 = prev_total, mean1 = *mean, m2_1 = *m2
        // population B: n2 = b_count, mean2 = b_mean, m2_2 = b_m2
        let n1 = prev_total;
        let n2 = b_count;
        let mean1 = *mean;
        let mean2 = b_mean;
        let m2_1 = *m2;
        let m2_2 = b_m2;

        let n = n1 + n2;
        let delta = mean2 - mean1;
        let mean_combined = (n1 * mean1 + n2 * mean2) / n;
        let m2_combined = m2_1 + m2_2 + delta * delta * (n1 * n2) / n;

        *mean = mean_combined;
        *m2 = m2_combined;
    }

    fn welford_add(mean: &mut f64, m2: &mut f64, count: f64, x: f64) {
        if count <= 0.0 {
            *mean = x;
            *m2 = 0.0;
            return;
        }
        let delta = x - *mean;
        let new_mean = *mean + delta / count;
        let new_m2 = *m2 + delta * (x - new_mean);

        *mean = new_mean;
        *m2 = new_m2;
    }

    fn handle_range_function(&self, rhs: &str) -> Option<Result<CellValue, String>> {
        // uppercase for uniformity
        let upper = rhs.to_uppercase();

        // check for SLEEP first (special)
        if upper.starts_with("SLEEP(") && upper.ends_with(')') {
            let inner = &upper[6..upper.len() - 1];
            if let Ok(seconds) = inner.parse::<u64>() {
                std::thread::sleep(std::time::Duration::from_secs(seconds));
                return Some(Ok(CellValue::Empty)); // sleep produces no numeric value
            } else {
                return Some(Err("invalid_sleep".into()));
            }
        }

        // find '('
        let open = upper.find('(')?;
        let close = upper.rfind(')')?;
        if close <= open {
            return Some(Err("invalid_function".into()));
        }

        let func_name = &upper[..open];
        let arg = &upper[open + 1..close];

        // parse range A1:B10
        let ((r1, c1), (r2, c2)) = match Self::parse_range(arg) {
            Ok(x) => x,
            Err(e) => return Some(Err(e)),
        };

        // determine which function
        let function = match func_name {
            "SUM" => RangeFunction::Sum,
            "AVG" => RangeFunction::Avg,
            "MIN" => RangeFunction::Min,
            "MAX" => RangeFunction::Max,
            "STDEV" => RangeFunction::Stdev,
            _ => return None, // not a valid function
        };

        // evaluate range
        let result = self.eval_range(r1, c1, r2, c2, function);

        Some(Ok(result))
    }
}
