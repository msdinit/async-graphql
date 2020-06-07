use crate::directives::OnInputValue;
use crate::{InputValueError, InputValueResult};

/// List minimum length validator
pub struct ListMinLength {
    /// Must be greater than or equal to this value.
    length: usize,
}

impl<'a, T: Sync + Send + 'a> OnInputValue<'a, Vec<T>> for ListMinLength {
    fn input_value(&'a self, value: Vec<T>) -> InputValueResult<Vec<T>> {
        if value.len() < self.length {
            return Err(InputValueError::Custom(format!(
                "the value length is {}, must be greater than or equal to {}",
                value.len(),
                self.length
            )));
        }
        Ok(value)
    }
}

/// List maximum length validator
pub struct ListMaxLength {
    /// Must be greater than or equal to this value.
    length: usize,
}

impl<'a, T: Sync + Send + 'a> OnInputValue<'a, Vec<T>> for ListMaxLength {
    fn input_value(&'a self, value: Vec<T>) -> InputValueResult<Vec<T>> {
        if value.len() > self.length {
            return Err(InputValueError::Custom(format!(
                "the value length is {}, must be less than or equal to {}",
                value.len(),
                self.length
            )));
        }
        Ok(value)
    }
}
