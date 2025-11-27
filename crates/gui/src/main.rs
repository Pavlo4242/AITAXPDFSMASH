use eframe::egui;
use rfd::FileDialog;
use std::sync::{Arc, Mutex};
use std::thread;

// Your producer trait and implementation
trait Producer {
    fn next(&mut self) -> Result<Option<Vec<u8>>, String>;
    fn size(&self) -> usize;
}

pub struct DefaultQuery {
    min_length: u32,
    max_length: u32,
    current: Vec<u8>,
    char_set: Vec<u8>,
    rolled: bool,
}

impl DefaultQuery {
    pub fn new(max_length: u32, min_length: u32) -> Self {
        let mut char_set: Vec<u8> = (b'0'..=b'9')
            .chain(b'A'..=b'Z')
            .chain(b'a'..=b'z')
            .chain(b'!'..=b'/')
            .chain(b':'..=b'@')
            .chain(b'['..=b'`')  // Fixed the invalid character
            .chain(b'{'..=b'~')
            .collect();

        char_set.sort();
        
        Self {
            max_length,
            min_length,
            current: vec![char_set[0]; min_length as usize],
            char_set,
            rolled: false,
        }
    }
}

impl Producer for DefaultQuery {
    fn next(&mut self) -> Result<Option<Vec<u8>>, String> {
        let mut stopped = false;
        for i in 0..self.current.len() {
            let spot = match self.char_set.binary_search(&self.current[i]) {
                Ok(spot) => spot,
                Err(_) => return Err("Couldn't find character in character set".to_string()),
            };
            if spot >= self.char_set.len() - 1 {
                self.current[i] = self.char_set[0];
            } else {
                self.current[i] = self.char_set[spot + 1];
                stopped = true;
                break;
            }
        }
        if !stopped {
            self.current.insert(0, self.char_set[0]);
            if self.current.len() > self.max_length as usize {
                if self.rolled {
                    return Err("Out of elements".to_string());
                } else {
                    self.rolled = true;
                    return Ok(Some(self.current.clone()));
                }
            }
        }
        let return_value = self.current.clone();
        Ok(Some(return_value))
    }

    fn size(&self) -> usize {
        let mut ret = 0usize;
        for len in self.min_length..=self.max_length {
            ret += self.char_set.len().pow(len);
        }
        ret
    }
}

#[derive(Default)]
struct PdfCrackerApp {
    input_file: String,
    min_length: String,
    max_length: String,
    
    // Cracking state
    is_running: bool,
    progress: f32,
    status: String,
    result: String,
    passwords_tried: usize,
    
    // Thread handle
    thread_handle: Option<thread::JoinHandle<()>>,
    shared_state: Arc<Mutex<SharedState>>,
}

struct SharedState {
    progress: f32,
    status: String,
    result: String,
    passwords_tried: usize,
    should_stop: bool,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            status: "Ready".to_string(),
            result: String::new(),
            passwords_tried: 0,
            should_stop: false,
        }
    }
}

impl PdfCrackerApp {
    fn start_cracking(&mut self) {
        if self.is_running {
            return;
        }

        if !self.validate_inputs() {
            return;
        }

        self.is_running = true;
        self.progress = 0.0;
        self.status = "Starting...".to_string();
        self.result.clear();
        self.passwords_tried = 0;

        let input_file = self.input_file.clone();
        let min_length: u32 = self.min_length.parse().unwrap_or(1);
        let max_length: u32 = self.max_length.parse().unwrap_or(8);
        let shared_state = Arc::clone(&self.shared_state);

        self.thread_handle = Some(thread::spawn(move || {
            if let Err(e) = PdfCrackerApp::run_cracker(input_file, min_length, max_length, shared_state) {
                let mut state = shared_state.lock().unwrap();
                state.result = format!("Error: {}", e);
                state.status = "Failed".to_string();
            }
        }));
    }

    fn run_cracker(
        pdf_path: String,
        min_length: u32,
        max_length: u32,
        shared_state: Arc<Mutex<SharedState>>
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pdf_bytes = std::fs::read(&pdf_path)?;
        
        {
            let mut state = shared_state.lock().unwrap();
            state.status = "Initializing password generator...".to_string();
        }

        // Use your DefaultQuery producer
        let mut producer = DefaultQuery::new(max_length, min_length);
        let total_passwords = producer.size();
        
        {
            let mut state = shared_state.lock().unwrap();
            state.status = format!("Total passwords to try: {}", total_passwords);
        }

        const BUFFER_SIZE: usize = 1000;
        let (password_sender, password_receiver) = crossbeam::channel::bounded(BUFFER_SIZE);
        let (result_sender, result_receiver) = crossbeam::channel::bounded(1);

        // Producer thread
        let producer_handle = thread::spawn({
            let shared_state = Arc::clone(&shared_state);
            move || {
                let mut count = 0;
                while let Ok(Some(password_bytes)) = producer.next() {
                    if shared_state.lock().unwrap().should_stop {
                        break;
                    }
                    
                    match String::from_utf8(password_bytes.clone()) {
                        Ok(password_str) => {
                            if password_sender.send(password_str).is_err() {
                                break; // Receiver dropped
                            }
                            count += 1;
                            
                            // Update progress occasionally
                            if count % 1000 == 0 {
                                let mut state = shared_state.lock().unwrap();
                                state.passwords_tried = count;
                                state.progress = count as f32 / total_passwords as f32;
                                state.status = format!("Tried {}/{} passwords", count, total_passwords);
                            }
                        }
                        Err(_) => continue, // Skip invalid UTF-8
                    }
                }
                // Signal end of passwords
                drop(password_sender);
            }
        });

        // Consumer threads (multiple workers)
        let num_workers = num_cpus::get().min(8);
        let mut worker_handles = vec![];

        for _ in 0..num_workers {
            let receiver = password_receiver.clone();
            let result_sender = result_sender.clone();
            let pdf_bytes = pdf_bytes.clone();
            let shared_state = Arc::clone(&shared_state);
            
            let handle = thread::spawn(move || {
                while let Ok(password) = receiver.recv() {
                    if shared_state.lock().unwrap().should_stop {
                        break;
                    }
                    
                    if try_password(&pdf_bytes, &password) {
                        let _ = result_sender.send(Some(password));
                        return;
                    }
                }
            });
            worker_handles.push(handle);
        }

        drop(password_receiver);
        drop(result_sender);

        // Wait for result
        let found_password = result_receiver.recv().unwrap_or(None);

        // Cleanup
        producer_handle.join().unwrap();
        for handle in worker_handles {
            handle.join().unwrap();
        }

        // Final state update
        let mut state = shared_state.lock().unwrap();
        if let Some(password) = found_password {
            state.result = format!("Password found: {}", password);
            state.status = "Success!".to_string();
            state.progress = 1.0;
        } else {
            if !state.should_stop {
                state.result = "Password not found".to_string();
                state.status = "Completed".to_string();
            }
        }

        Ok(())
    }

    fn stop_cracking(&mut self) {
        if self.is_running {
            let mut state = self.shared_state.lock().unwrap();
            state.should_stop = true;
            self.is_running = false;
            self.status = "Stopping...".to_string();
        }
    }

    fn validate_inputs(&self) -> bool {
        if self.input_file.trim().is_empty() {
            return false;
        }
        if self.min_length.trim().is_empty() || !self.min_length.chars().all(|c| c.is_numeric()) {
            return false;
        }
        if self.max_length.trim().is_empty() || !self.max_length.chars().all(|c| c.is_numeric()) {
            return false;
        }
        let min: u32 = self.min_length.parse().unwrap_or(0);
        let max: u32 = self.max_length.parse().unwrap_or(0);
        min > 0 && max >= min
    }

    fn update_from_thread(&mut self) {
        let state = self.shared_state.lock().unwrap();
        self.progress = state.progress;
        self.status = state.status.clone();
        self.result = state.result.clone();
        self.passwords_tried = state.passwords_tried;
        
        if let Some(handle) = &self.thread_handle {
            if handle.is_finished() {
                drop(state);
                let mut state = self.shared_state.lock().unwrap();
                state.should_stop = false;
                self.is_running = false;
            }
        }
    }
}

fn try_password(pdf_contents: &[u8], password: &str) -> bool {
    pdf::file::FileOptions::cached()
        .password(password.as_bytes())
        .load(pdf_contents)
        .is_ok()
}

impl eframe::App for PdfCrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_from_thread();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PDF Password Cracker");
            
            // File selection
            ui.horizontal(|ui| {
                ui.label("PDF File:");
                ui.text_edit_singleline(&mut self.input_file);
                
                if ui.button("📁").clicked() && !self.is_running {
                    if let Some(file) = FileDialog::new().add_filter("PDF", &["pdf"]).pick_file() {
                        self.input_file = file.display().to_string();
                    }
                }
            });
            
            ui.add_space(10.0);
            
            // Password length range
            ui.horizontal(|ui| {
                ui.label("Min Length:");
                ui.text_edit_singleline(&mut self.min_length)
                    .on_hover_text("Minimum password length");
                
                ui.label("Max Length:");
                ui.text_edit_singleline(&mut self.max_length)
                    .on_hover_text("Maximum password length");
            });
            
            ui.add_space(20.0);
            
            // Control buttons
            ui.horizontal(|ui| {
                if ui.button("Start Cracking").clicked() && !self.is_running {
                    self.start_cracking();
                }
                
                if self.is_running && ui.button("Stop").clicked() {
                    self.stop_cracking();
                }
            });
            
            ui.add_space(10.0);
            
            // Progress and status
            if self.is_running {
                ui.add(egui::ProgressBar::new(self.progress).show_percentage());
                ui.label(&self.status);
                if self.passwords_tried > 0 {
                    ui.label(format!("Passwords tried: {}", self.passwords_tried));
                }
            }
            
            // Results
            if !self.result.is_empty() {
                ui.separator();
                ui.label("Result:");
                ui.text_edit_multiline(&mut self.result);
            }
        });

        if self.is_running {
            ctx.request_repaint();
        }
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(500.0, 400.0)),
        resizable: false,
        ..Default::default()
    };
    
    eframe::run_native(
        "PDF Cracker",
        native_options,
        Box::new(|_cc| {
            Box::new(PdfCrackerApp {
                min_length: "1".to_string(),
                max_length: "4".to_string(), // Start small for testing
                shared_state: Arc::new(Mutex::new(SharedState::default())),
                ..Default::default()
            })
        }),
    )
}
