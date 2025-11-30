use spreadsheet_lib::sheet::Sheet;
use spreadsheet_lib::cell::CellValue;

#[test]
fn test_undo_basic() {
    let mut sheet = Sheet::new();
    
    // A1 = 10
    sheet.execute_command("A1=10");
    assert_eq!(get_number(&sheet, 0, 0), 10.0);
    
    // A1 = 20
    sheet.execute_command("A1=20");
    assert_eq!(get_number(&sheet, 0, 0), 20.0);
    
    // UNDO -> A1 should be 10
    sheet.execute_command("UNDO");
    assert_eq!(get_number(&sheet, 0, 0), 10.0);
    
    // UNDO -> A1 should be empty (or 0 if empty treated as 0, but here we check cell existence)
    sheet.execute_command("UNDO");
    assert!(sheet.get_cell(0, 0).is_none() || matches!(sheet.get_cell(0, 0).unwrap().value, CellValue::Empty));
}

#[test]
fn test_redo_basic() {
    let mut sheet = Sheet::new();
    
    sheet.execute_command("A1=10");
    sheet.execute_command("UNDO");
    assert!(sheet.get_cell(0, 0).is_none() || matches!(sheet.get_cell(0, 0).unwrap().value, CellValue::Empty));
    
    sheet.execute_command("REDO");
    assert_eq!(get_number(&sheet, 0, 0), 10.0);
}

#[test]
fn test_undo_dependency() {
    let mut sheet = Sheet::new();
    
    // A1 = 10
    sheet.execute_command("A1=10");
    // B1 = A1 * 2
    sheet.execute_command("B1=A1*2");
    
    assert_eq!(get_number(&sheet, 0, 1), 20.0); // B1
    
    // Change A1 -> 5. B1 should be 10.
    sheet.execute_command("A1=5");
    assert_eq!(get_number(&sheet, 0, 1), 10.0);
    
    // UNDO. A1 -> 10. B1 should update to 20.
    sheet.execute_command("UNDO");
    assert_eq!(get_number(&sheet, 0, 0), 10.0);
    assert_eq!(get_number(&sheet, 0, 1), 20.0);
}

#[test]
fn test_undo_circular() {
    let mut sheet = Sheet::new();
    
    sheet.execute_command("A1=10");
    sheet.execute_command("B1=A1");
    
    // Try circular: A1 = B1
    // This should fail and set A1 to Err
    sheet.execute_command("A1=B1");
    
    if let Some(cell) = sheet.get_cell(0, 0) {
        assert!(matches!(cell.value, CellValue::Err));
    }
    
    // UNDO. A1 should be 10. B1 should be 10.
    sheet.execute_command("UNDO");
    assert_eq!(get_number(&sheet, 0, 0), 10.0);
    assert_eq!(get_number(&sheet, 0, 1), 10.0);
}

fn get_number(sheet: &Sheet, r: usize, c: usize) -> f64 {
    if let Some(cell) = sheet.get_cell(r, c) {
        if let CellValue::Number(n) = cell.value {
            return n;
        }
    }
    panic!("Expected number at {},{}", r, c);
}
