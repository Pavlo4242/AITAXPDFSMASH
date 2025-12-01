use colored::*;
#[cfg(windows)]
pub fn banner() {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();

    println!(
    use colored::*;
#[cfg(windows)]
pub fn banner() {
    println!(
        "{}",
        format!(include_str!("banner.txt"), env!("CARGO_PKG_VERSION"))
            .bold()
            .red()
    );
