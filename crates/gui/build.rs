#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set("CompanyName", "YOUR COMPANY NAME HERE")
        .set("FileDescription", "PDF Password Recovery Tool")
        .set("ProductName", "PDF Password Recovery")
        .set("LegalCopyright", "Based on PDFRip by Mufeed VH & Pommaq");
    
    if let Err(e) = res.compile() {
        eprintln!("Warning: {}", e);
    }
}

#[cfg(not(windows))]
fn main() {}
