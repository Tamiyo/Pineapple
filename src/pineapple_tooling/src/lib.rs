mod cli;

pub fn exec_from_command_line() {
    match cli::parse_cli() {
        Ok(_) => (),
        Err(e) => panic!(format!("{}", e)),
    }
}
