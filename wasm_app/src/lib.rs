//! Thin wasm-bindgen wrapper around `spreadsheet_lib::sheet::Sheet`.
//!
//! This crate exists purely to expose the existing spreadsheet engine to
//! JavaScript in the browser. It doesn't reimplement any logic -- every
//! method here just forwards to the real `Sheet` API used by the native
//! `cli` binary, and converts the return types to something wasm-bindgen
//! can pass across the JS boundary (numbers, bools, and strings only).

use spreadsheet_lib::cell::CellValue;
use spreadsheet_lib::sheet::Sheet;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SpreadsheetApp {
    sheet: Sheet,
}

#[wasm_bindgen]
impl SpreadsheetApp {
    /// Create a fresh, empty spreadsheet.
    #[wasm_bindgen(constructor)]
    pub fn new() -> SpreadsheetApp {
        SpreadsheetApp { sheet: Sheet::new() }
    }

    /// Run one command, e.g. "A1=23", "B2=A1+10", "C1=SUM(A1:A10)", "U" (undo), "R" (redo).
    /// Returns false only for the quit command ("Q"); true otherwise.
    pub fn execute(&mut self, input: &str) -> bool {
        self.sheet.execute_command(input)
    }

    /// Displayed value of a cell (0-indexed), formatted as a string.
    /// Empty/unset cells return "".
    pub fn get_cell(&self, row: u32, col: u32) -> String {
        match self.sheet.get_cell(row as usize, col as usize) {
            Some(cell) => match &cell.value {
                CellValue::Number(x) => format_number(*x),
                CellValue::Err => "ERR".to_string(),
                CellValue::Div0 => "DIV0".to_string(),
                CellValue::Empty => String::new(),
            },
            None => String::new(),
        }
    }

    /// The raw formula/input that produced a cell's value (for editing it again).
    pub fn get_formula(&self, row: u32, col: u32) -> String {
        match self.sheet.get_cell(row as usize, col as usize) {
            Some(cell) => cell.formula.clone().unwrap_or_default(),
            None => String::new(),
        }
    }

    /// Status of the last executed command (e.g. "ok", "circular_reference", "DIV0").
    pub fn status(&self) -> String {
        self.sheet.last_status.clone()
    }

    /// How long the last command took to execute, in milliseconds.
    pub fn last_time_ms(&self) -> f64 {
        self.sheet.last_time * 1000.0
    }

    pub fn undo(&mut self) {
        self.sheet.undo();
    }

    pub fn redo(&mut self) {
        self.sheet.redo();
    }
}

impl Default for SpreadsheetApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a number the way a spreadsheet cell should: integers with no
/// trailing ".0", everything else trimmed of insignificant trailing zeros.
fn format_number(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1e15 {
        format!("{}", x as i64)
    } else {
        let s = format!("{:.6}", x);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        s.to_string()
    }
}
