use super::{MAX_DERIVED_SQL_NAME_BYTES, NameError};

pub(super) struct Output<'a> {
    type_name: &'a str,
    text: String,
}

impl<'a> Output<'a> {
    pub(super) fn new(type_name: &'a str) -> Self {
        Self {
            type_name,
            text: String::with_capacity(MAX_DERIVED_SQL_NAME_BYTES),
        }
    }

    pub(super) fn separator(&mut self) -> Result<(), NameError> {
        if !self.text.is_empty() {
            self.push('_')?;
        }
        Ok(())
    }

    pub(super) fn push(&mut self, ch: char) -> Result<(), NameError> {
        self.check_growth(ch.len_utf8())?;
        self.text.push(ch);
        Ok(())
    }

    pub(super) fn extend(&mut self, piece: &str) -> Result<(), NameError> {
        self.check_growth(piece.len())?;
        self.text.push_str(piece);
        Ok(())
    }

    pub(super) fn finish(self) -> String {
        self.text
    }

    fn check_growth(&self, additional: usize) -> Result<(), NameError> {
        let within_limit = self
            .text
            .len()
            .checked_add(additional)
            .is_some_and(|length| length <= MAX_DERIVED_SQL_NAME_BYTES);
        if !within_limit {
            return Err(NameError::TooLong {
                type_name: self.type_name.to_owned(),
                limit: MAX_DERIVED_SQL_NAME_BYTES,
            });
        }
        Ok(())
    }
}
