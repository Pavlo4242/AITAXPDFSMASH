use super::Producer;

pub struct DateProducer {
    current: usize,
    end: usize,
    inner: Box<dyn Iterator<Item = String>>,
    counter: usize,
}

fn pregenerate_dates() -> Vec<String> {
    let mut results = Vec::new();
    for month in 1..13 {
        for date in 1..32 {
            let date: String = if date < 10 {
                format!("0{}", date)
            } else {
                date.to_string()
            };

            let month: String = if month < 10 {
                format!("0{}", month)
            } else {
                month.to_string()
            };

            results.push(format!("{}{}", date, month))
        }
    }
    results
}

impl DateProducer {
    pub fn new(start: usize, end: usize) -> Self {
        let dates = pregenerate_dates().into_iter().cycle();

        Self {
            current: start,
            end,
            inner: Box::from(dates),
            counter: 0,
        }
    }
}

impl Producer for DateProducer {
    fn next(&mut self) -> Result<Option<Vec<u8>>, String> {
        if self.current > self.end {
            debug!("stopping at year {}", self.current);
            Ok(None)
        } else {
            if self.counter == 12 * 31 {
                self.counter = 0;
                self.current += 1;
            } else {
                self.counter += 1;
            }

            let next = self.inner.next().unwrap();
            let password = format!("{:04}{:04}", next, self.current).into_bytes();
            debug!("Sending {} from DateProducer", String::from_utf8_lossy(&password));
            Ok(Some(password))
        }
    }

    fn size(&self) -> usize {
        let mut years = self.end - self.current;
        if years == 0 {
            years = 1;
        }
        12 * 31 * years
    }
}
