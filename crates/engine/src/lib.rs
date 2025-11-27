#[macro_use]
extern crate log;

pub mod producers {
    pub use producer::*;
}

pub mod crackers {
    pub use cracker::{PDFCracker, PDFCrackerState};
}

const BUFFER_SIZE: usize = 200;

use std::sync::Arc;
use crossbeam::channel::{Receiver, Sender, TryRecvError};
use producer::Producer;
use cracker::{PDFCracker, PDFCrackerState};

pub fn crack_file(
    no_workers: usize,
    cracker: PDFCracker,
    mut producer: Box<dyn Producer>,
    callback: Box<dyn Fn()>,
) -> anyhow::Result<Option<Vec<u8>>> {
    let (sender, r): (Sender<Vec<u8>>, Receiver<_>) = crossbeam::channel::bounded(BUFFER_SIZE);
    let (success_sender, success_reader) = crossbeam::channel::unbounded::<Vec<u8>>();
    let mut handles = vec![];
    let cracker_handle = Arc::from(cracker);

    for _ in 0..no_workers {
        let success = success_sender.clone();
        let r2 = r.clone();
        let c2 = cracker_handle.clone();
        let id: std::thread::JoinHandle<()> = std::thread::spawn(move || {
            let Ok(mut cracker) = PDFCrackerState::from_cracker(&c2) else {
                return
            };

            while let Ok(passwd) = r2.recv() {
                if cracker.attempt(&passwd) {
                    success.send(passwd).unwrap_or_default();
                    return;
                }
            }
        });
        handles.push(id);
    }
    
    drop(r);
    drop(success_sender);

    info!("Starting password cracking job...");

    let mut success = None;

    loop {
        match success_reader.try_recv() {
            Ok(password) => {
                success = Some(password);
                break;
            }
            Err(e) => {
                match e {
                    TryRecvError::Empty => {}
                    TryRecvError::Disconnected => {
                        error!("All workers have exited prematurely");
                        break;
                    }
                }
            }
        }

        match producer.next() {
            Ok(Some(password)) => {
                if sender.send(password).is_err() {
                    error!("unable to send next password since channel is closed");
                }
                callback()
            }
            Ok(None) => {
                trace!("out of passwords, exiting loop");
                break;
            }
            Err(error_msg) => {
                error!("error occured while sending: {error_msg}");
                break;
            }
        }
    }

    drop(sender);

    let found_password = match success {
        Some(result) => Some(result),
        None => {
            match success_reader.recv() {
                Ok(result) => Some(result),
                Err(e) => {
                    debug!("{}", e);
                    None
                }
            }
        }
    };

    Ok(found_password)
}
