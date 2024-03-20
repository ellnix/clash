pub struct TokenStream<'a> {
    input: &'a str,
}

impl<'a> From<&'a str> for TokenStream<'a> {
    fn from(input: &'a str) -> Self {
        Self { input }
    }
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        match self.input.chars().next() {
            Some('\n') => {
                self.input = &self.input[1..];
                Some("\n")
            }
            Some(' ') => {
                self.input = &self.input[1..];
                Some("")
            }
            Some(_) => {
                let split_index = self.input.find(&[' ', '\n']).unwrap_or(self.input.len());
                let (token, rest) = self.input.split_at(split_index);

                self.input = if rest.starts_with(" ") { &rest[1..] } else { rest };

                Some(token)
            }
            None => None,
        }
    }
}
