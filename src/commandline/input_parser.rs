pub struct InputParser<'a> {
    string: String,
    char_indices: std::str::CharIndices<'a>,
    last_index: usize,
}

impl<'a> InputParser<'a> {
    pub fn new(raw_string: String) -> InputParser<'a> {
        InputParser {
            string: raw_string.clone(),
            char_indices: raw_string.char_indices(),
            last_index: 0,
        }
    }
}

impl<'a> Iterator for InputParser<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last_index >= self.string.len() {
            return None;
        }
        let mut has_quote = false;
        let mut this_index = self.last_index;
        loop {
            if let Some((next_index, next_char)) = self.char_indices.next() {
                this_index = next_index;
                if next_char == '"' {
                    has_quote = !has_quote;
                }
                if next_char.is_whitespace() && !has_quote{
                    let result_str = &self.string[self.last_index .. this_index];
                    self.last_index = this_index + 1;

                    return Some(result_str);
                }
            } else {
                this_index += 1;
                let result_str = &self.string[self.last_index .. this_index];
                self.last_index = this_index;

                return Some(result_str);
            }
        }
    }
}
