use spreadsheet_lib::cell::Cell; // (not used yet but okay to keep)
use spreadsheet_lib::sheet::Sheet;
use std::io::{self, Write};

fn main() {
    let mut sheet = Sheet::new();

    // Initial display
    sheet.display();
    sheet.print_status();

    loop {
        // print prompt
        // print!("> ");
        io::stdout().flush().unwrap();

        // read user input
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read input.");
            continue;
        }

        // process command
        let continue_running = sheet.execute_command(&input);

        // display sheet (if output enabled)
        if sheet.output_enabled {
            sheet.display();
        }

        // always show status line
        sheet.print_status();

        // quit if needed
        if !continue_running {
            break;
        }
    }
}
