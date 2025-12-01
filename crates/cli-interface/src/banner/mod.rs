use colored::*;
#[cfg(windows)]
use colored::control; // Added missing import for `control::set_virtual_terminal`

pub fn banner() {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();

    println!(
        "{}",
        format!(include_str!("banner.txt"), env!("CARGO_PKG_VERSION"))
            .bold()
            .red()
    );
}
