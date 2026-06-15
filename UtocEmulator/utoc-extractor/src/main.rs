pub(crate) mod cli;
pub(crate) mod gui;

use std::error::Error;

pub(crate) type GenericResult<T> = Result<T, Box<dyn Error>>;

fn main() {
    let argc = std::env::args().count();
    if let Err(e) = match argc {
        1 => gui::execute(),
        _ => cli::execute()
    } {
        println!("{}: {}", console::style("ERROR").red(), e);
        if argc == 1 {
            console::Term::stdout().read_key().unwrap();
        }
    }
}
