pub struct InputParser {
    string: String,
    char_indices: Vec<(usize, char)>,
    last_vec_index: usize,
    last_char_index: usize,
}

impl InputParser {
    pub fn new(raw_string: String) -> InputParser {
        InputParser {
            string: raw_string.clone(),
            char_indices: raw_string.char_indices().collect(),
            last_vec_index: 0,
            last_char_index: 0,
        }
    }
}

impl Iterator for InputParser {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last_char_index >= self.char_indices.len() {
            return None;
        }

        loop {
            if let Some((next_index, next_char)) = self.char_indices.get(self.last_vec_index) {
                if !next_char.is_whitespace() {
                    break;
                }
                self.last_vec_index += 1;
                self.last_char_index = next_index + 1;
            } else {
                return None;
            }
        }

        let mut has_quote = false;

        if let Some((next_index, next_char)) = self.char_indices.get(self.last_vec_index) {
            let next_index = *next_index;
            let next_char = *next_char;
            if next_char == '"' {
                self.last_vec_index += 1;
                self.last_char_index = next_index + 1;
                has_quote = true;
            }
        }

        let mut this_vec_index = self.last_vec_index;
        let mut this_char_index = self.last_char_index;
        loop {
            if let Some((next_index, next_char)) = self.char_indices.get(this_vec_index) {
                let next_index = *next_index;
                let next_char = *next_char;
                this_vec_index += 1;
                this_char_index = next_index;
                if next_char == '"' && has_quote {
                    let result_str = self.string[self.last_char_index .. this_char_index].to_string();
                    self.last_char_index = this_char_index + 1;
                    self.last_vec_index = this_vec_index + 1;

                    return Some(result_str);
                }
                if next_char.is_whitespace() && !has_quote {
                    let result_str = self.string[self.last_char_index .. this_char_index].to_string();
                    self.last_char_index = this_char_index + 1;
                    self.last_vec_index = this_vec_index;

                    return Some(result_str);
                }
            } else {
                this_char_index += 1;
                let result_str = self.string[self.last_char_index .. this_char_index].to_string();
                self.last_char_index = this_char_index;

                return Some(result_str);
            }
        }
    }
}
