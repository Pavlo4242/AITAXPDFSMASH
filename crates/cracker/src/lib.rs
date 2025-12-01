use std::collections::hash_map::HashMap;
use anyhow::anyhow;
use pdf::PdfError;
use pdf::any::AnySync;
use pdf::file::{Cache, Storage}; // Removed NoLog
use pdf::object::{ParseOptions, PlainRef};

#[derive(Clone)]
pub struct PDFCracker(Vec<u8>);

impl PDFCracker {
    pub fn from_file(path: &str) -> Result<Self, io::Error> {
        let pdf_file: Vec<u8> = fs::read(path)?;
        Ok(Self(pdf_file))
    }
}

type ObjectCache = SimpleCache<Result<AnySync, Arc<PdfError>>>;
type StreamCache = SimpleCache<Result<Arc<[u8]>, Arc<PdfError>>>;

pub struct PDFCrackerState(Storage<Vec<u8>, ObjectCache, StreamCache>); // Removed NoLog generic argument

impl PDFCrackerState {
    pub fn from_cracker(pdf_file: &PDFCracker) -> anyhow::Result<Self> {
        let res = Storage::with_cache(
            pdf_file.0.clone(),
            ParseOptions::strict(),
            SimpleCache::new(),
            SimpleCache::new()
        ); // Removed NoLog argument

        match res {
            Ok(storage) => Ok(Self(storage)),
            Err(err) => Err(anyhow!(err).context("Failed to init cracker")),
        }
    }

    pub fn attempt(&mut self, password: &[u8]) -> bool {
        self.0.load_storage_and_trailer_password(password).is_ok()
    }
}

struct SimpleCache<T>(RefCell<HashMap<PlainRef, T>>);

impl<T: Clone> SimpleCache<T> {
    fn new() -> Self {
        Self(RefCell::new(HashMap::new()))
    }
}

impl<T: Clone> Cache<T> for SimpleCache<T> {
    fn get_or_compute(&self, key: PlainRef, compute: impl FnOnce() -> T) -> T {
        let mut hash = self.0.borrow_mut();
        match hash.get(&key) {
            Some(value) => value.clone(),
            None => {
                let value = compute();
                hash.insert(key, value.clone());
                value
            }
        }
    }
}
