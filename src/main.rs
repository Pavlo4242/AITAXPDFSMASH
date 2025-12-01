// crates/gui/src/main.rs - IMPROVED VERSION
#![windows_subsystem = "windows"]
use cli_interface::{arguments::args, entrypoint, Code};
use pretty_env_logger;

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let args = args();
    
    match entrypoint(args)? {
        Code::Success => Ok(()),
        Code::Failure => std::process::exit(1),
    }
}
use native_windows_gui as nwg;
use native_windows_derive as nwd;
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub struct PDFRipGui {
    window: nwg::Window,
    grid: nwg::GridLayout,
    
    // UI Elements
    title_label: nwg::Label,
    subtitle_label: nwg::Label,
    file_label: nwg::Label,
    file_input: nwg::TextInput,
    browse_button: nwg::Button,
    first_initial_label: nwg::Label,
    first_initial_input: nwg::TextInput,
    last_initial_label: nwg::Label,
    last_initial_input: nwg::TextInput,
    ssn_label: nwg::Label,
    ssn_input: nwg::TextInput,
    threads_label: nwg::Label,
    threads_input: nwg::TextInput,
    output_box: nwg::TextBox,
    start_button: nwg::Button,
    stop_button: nwg::Button,
    clear_button: nwg::Button,
    
    // Resources
    file_dialog: nwg::FileDialog,
    title_font: nwg::Font,
    small_font: nwg::Font,
    output_notice: nwg::Notice,
    
    // State
    running: Arc<AtomicBool>,
    output_buffer: Arc<Mutex<Vec<String>>>,
}

impl PDFRipGui {
    fn build_ui(mut data: Self) -> Result<Self, nwg::NwgError> {
        // Initialize fonts
        nwg::Font::builder()
            .family("Segoe UI")
            .size(18)
            .weight(700)
            .build(&mut data.title_font)?;
            
        nwg::Font::builder()
            .family("Segoe UI")
            .size(9)
            .build(&mut data.small_font)?;
        
        // Build window
        nwg::Window::builder()
            .size((600, 500))
            .position((300, 300))
            .title("PDF Password Recovery Tool")
            .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
            .build(&mut data.window)?;
        
        // Build file dialog
        nwg::FileDialog::builder()
            .action(nwg::FileDialogAction::Open)
            .title("Select PDF File")
            .filters("PDF Files(*.pdf)|All Files(*.*)")
            .build(&mut data.file_dialog)?;
        
        // Build notice for thread communication
        nwg::Notice::builder()
            .parent(&data.window)
            .build(&mut data.output_notice)?;
        
        // Build layout
        nwg::GridLayout::builder()
            .parent(&data.window)
            .spacing(1)
            .child_item(nwg::GridLayoutItem::new(&data.title_label, 0, 0, 3, 1))
            .build(&mut data.grid)?;
        
        // Title
        nwg::Label::builder()
            .text("PDF Password Recovery Tool")
            .font(Some(&data.title_font))
            .parent(&data.window)
            .build(&mut data.title_label)?;
        
        // Subtitle
        nwg::Label::builder()
            .text("Powered by PDFRip v2.0.1\nCustomized for AITAX ADVISERS")
            .font(Some(&data.small_font))
            .parent(&data.window)
            .build(&mut data.subtitle_label)?;
        
        // File selection row
        nwg::Label::builder()
            .text("PDF File:")
            .parent(&data.window)
            .build(&mut data.file_label)?;
        
        nwg::TextInput::builder()
            .readonly(true)
            .parent(&data.window)
            .build(&mut data.file_input)?;
        
        nwg::Button::builder()
            .text("Browse...")
            .parent(&data.window)
            .build(&mut data.browse_button)?;
        
        // First initial
        nwg::Label::builder()
            .text("First Initial:")
            .parent(&data.window)
            .build(&mut data.first_initial_label)?;
        
        nwg::TextInput::builder()
            .placeholder_text(Some("e.g., T"))
            .parent(&data.window)
            .build(&mut data.first_initial_input)?;
        
        // Last initial
        nwg::Label::builder()
            .text("Last Initial:")
            .parent(&data.window)
            .build(&mut data.last_initial_label)?;
        
        nwg::TextInput::builder()
            .placeholder_text(Some("e.g., C"))
            .parent(&data.window)
            .build(&mut data.last_initial_input)?;
        
        // SSN
        nwg::Label::builder()
            .text("Last 4-6 of SSN:")
            .parent(&data.window)
            .build(&mut data.ssn_label)?;
        
        nwg::TextInput::builder()
            .placeholder_text(Some("e.g., 1234"))
            .parent(&data.window)
            .build(&mut data.ssn_input)?;
        
        // Threads
        nwg::Label::builder()
            .text("CPU Threads:")
            .parent(&data.window)
            .build(&mut data.threads_label)?;
        
        nwg::TextInput::builder()
            .text("8")
            .parent(&data.window)
            .build(&mut data.threads_input)?;
        
        // Output box
        nwg::TextBox::builder()
            .text("")
            .readonly(true)
            .flags(nwg::TextBoxFlags::VISIBLE | nwg::TextBoxFlags::VSCROLL | nwg::TextBoxFlags::AUTOVSCROLL)
            .parent(&data.window)
            .build(&mut data.output_box)?;
        
        // Buttons
        nwg::Button::builder()
            .text("Start Recovery")
            .parent(&data.window)
            .build(&mut data.start_button)?;
        
        nwg::Button::builder()
            .text("Stop")
            .enabled(false)
            .parent(&data.window)
            .build(&mut data.stop_button)?;
        
        nwg::Button::builder()
            .text("Clear")
            .parent(&data.window)
            .build(&mut data.clear_button)?;
        
        // Bind events
        let window_handles = data.window.handle;
        let browse_handler = nwg::full_bind_event_handler(&data.window.handle, move |evt, _evt_data, handle| {
            use nwg::Event;
            
            if handle == data.browse_button.handle {
                if let Event::OnButtonClick = evt {
                    Self::select_file(&data);
                }
            }
        });
        
        Ok(data)
    }
    
    fn select_file(&self) {
        if self.file_dialog.run(Some(&self.window)) {
            if let Ok(path) = self.file_dialog.get_selected_item() {
                self.file_input.set_text(&path);
            }
        }
    }
    
    fn append_output(&self, text: &str) {
        let current = self.output_box.text();
        self.output_box.set_text(&format!("{}{}\r\n", current, text));
        let len = self.output_box.text().len();
        self.output_box.set_selection(len..len);
    }
    
    fn start_recovery(&self) {
        let pdf_path = self.file_input.text();
        if pdf_path.is_empty() {
            nwg::modal_error_message(&self.window, "Error", "Please select a PDF file");
            return;
        }
        
        let first_initial = self.first_initial_input.text().trim().to_string();
        let last_initial = self.last_initial_input.text().trim().to_string();
        let ssn_text = self.ssn_input.text().trim().to_string();
        
        if first_initial.is_empty() || last_initial.is_empty() {
            nwg::modal_error_message(&self.window, "Error", "Please enter both initials");
            return;
        }
        
        if ssn_text.is_empty() {
            nwg::modal_error_message(&self.window, "Error", "Please enter last 4-6 digits of SSN");
            return;
        }
        
        // Take only last 4 digits
        let ssn_digits: String = ssn_text.chars()
            .filter(|c| c.is_numeric())
            .rev()
            .take(4)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        
        let threads = self.threads_input.text().parse::<usize>().unwrap_or(8);
        
        self.start_button.set_enabled(false);
        self.stop_button.set_enabled(true);
        self.running.store(true, Ordering::SeqCst);
        
        self.append_output("===========================================");
        self.append_output(&format!("Starting password recovery for: {}", pdf_path));
        self.append_output(&format!("Client Initials: {}{}", first_initial, last_initial));
        self.append_output(&format!("SSN Digits: {}", ssn_digits));
        self.append_output(&format!("Threads: {}", threads));
        self.append_output("===========================================");
        
        let exe_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("pdfrip.exe")))
            .unwrap_or_else(|| PathBuf::from("pdfrip.exe"));
        
        let patterns = self.build_patterns(&first_initial, &last_initial, &ssn_digits);
        let output_buffer = self.output_buffer.clone();
        let running = self.running.clone();
        let notice = self.output_notice.sender();
        
        std::thread::spawn(move || {
            for (i, pattern) in patterns.iter().enumerate() {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                
                let msg = format!("Attempt {}/{}: Testing pattern '{}'", i + 1, patterns.len(), pattern);
                output_buffer.lock().unwrap().push(msg);
                notice.notice();
                
                let result = Command::new(&exe_path)
                    .args(&["-n", &threads.to_string(), "-f", &pdf_path, "custom-query", pattern])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output();
                
                match result {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        
                        if stdout.contains("Success!") || stdout.contains("Found password") {
                            output_buffer.lock().unwrap().push(format!("✓ SUCCESS! Password found with pattern: {}", pattern));
                            output_buffer.lock().unwrap().push(stdout.to_string());
                            running.store(false, Ordering::SeqCst);
                            notice.notice();
                            return;
                        }
                    }
                    Err(e) => {
                        output_buffer.lock().unwrap().push(format!("  Error executing pdfrip: {}", e));
                        notice.notice();
                    }
                }
            }
            
            if running.load(Ordering::SeqCst) {
                output_buffer.lock().unwrap().push("❌ All patterns exhausted. Password not found.".to_string());
            } else {
                output_buffer.lock().unwrap().push("⏹ Recovery stopped by user.".to_string());
            }
            
running.store(false, Ordering::SeqCst);
            notice.notice();
        });
    }
    
    fn build_patterns(&self, first: &str, last: &str, ssn_digits: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        
        // Pattern 1: SSN digits alone as a fixed password, treated as a range {SSN-SSN}
        // This is done to use the existing 'custom-query' engine method.
        let ssn_fixed_pattern = format!("{{{}-{}}}", ssn_digits, ssn_digits);
        patterns.push(ssn_fixed_pattern.clone());

        // Pattern 2: Numbers only (original brute-force ranges)
        patterns.push("{0-9999}".to_string());
        patterns.push("{0-99999}".to_string());
        patterns.push("{0-999999}".to_string());
        
        // All case combinations
        let combos = vec![
            format!("{}{}", first.to_lowercase(), last.to_lowercase()),
            format!("{}{}", first.to_uppercase(), last.to_uppercase()),
            format!("{}{}", first.to_uppercase(), last.to_lowercase()),
            format!("{}{}", first.to_lowercase(), last.to_uppercase()),
        ];
        
        for combo in &combos {
            patterns.push(format!("{}{{{}-{}}}", combo, 0, 9999));
            patterns.push(format!("{}{{{}-{}}}", combo, 0, 99999));
            patterns.push(format!("{}{{{}-{}}}", combo, 0, 999999));
        }

        // Pattern 3: Initials + SSN (fixed SSN suffix, as a fixed password pattern)
        for combo in combos {
             patterns.push(format!("{}{}", combo, ssn_fixed_pattern));
        }
        
        patterns
    }

   
    
    fn process_output(&self) {
        let mut buffer = self.output_buffer.lock().unwrap();
        for msg in buffer.drain(..) {
            self.append_output(&msg);
        }
        
        if !self.running.load(Ordering::SeqCst) {
            self.start_button.set_enabled(true);
            self.stop_button.set_enabled(false);
        }
    }
    
    fn stop_recovery(&self) {
        self.running.store(false, Ordering::SeqCst);
        self.append_output("Stopping recovery...");
    }
    
    fn clear_output(&self) {
        self.output_box.set_text("");
    }
    
    fn exit(&self) {
        self.running.store(false, Ordering::SeqCst);
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    
    let app = PDFRipGui::build_ui(Default::default()).expect("Failed to build UI");
    
    nwg::dispatch_thread_events();
}
