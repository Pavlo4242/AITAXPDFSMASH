use super::Producer;

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
            .chain(b'['..=b'')
            .chain(b'{'..=b'~')
            .collect();

        char_set.sort();
        
        Self {
            max_length,
            min_length,
            current: vec![char_set[0]; min_length.try_into().unwrap()],
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
            if self.current.len() > self.max_length.try_into().unwrap() {
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
