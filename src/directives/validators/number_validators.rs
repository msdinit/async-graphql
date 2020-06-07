use crate::directives::OnInputValue;
use crate::{InputValueError, InputValueResult};
use num_traits::Num;
use serde::export::fmt::Display;

/// Number equal validator
pub struct NumberEqual<T> {
    /// Equal this value.
    value: T,
}

impl<'a, T: Num + Sync + Send + Display + 'a> OnInputValue<'a, T> for NumberEqual<T> {
    fn input_value(&'a self, value: T) -> InputValueResult<T> {
        if value != self.value {
            return Err(InputValueError::Custom(format!(
                "the value is {}, must be equal to {}",
                value, self.value
            )));
        }
        Ok(value)
    }
}

/// Number greater then validator
pub struct NumberGreaterThan<T> {
    /// Greater then this value.
    value: T,
}

impl<'a, T: Num + PartialOrd + Sync + Send + Display + 'a> OnInputValue<'a, T>
    for NumberGreaterThan<T>
{
    fn input_value(&'a self, value: T) -> InputValueResult<T> {
        if value <= self.value {
            return Err(InputValueError::Custom(format!(
                "the value is {}, must be greater than {}",
                value, self.value
            )));
        }
        Ok(value)
    }
}

/// Number less then validator
pub struct NumberLessThan<T> {
    /// Less then this value
    value: T,
}

impl<'a, T: Num + PartialOrd + Sync + Send + Display + 'a> OnInputValue<'a, T>
    for NumberLessThan<T>
{
    fn input_value(&'a self, value: T) -> InputValueResult<T> {
        if value > self.value {
            return Err(InputValueError::Custom(format!(
                "the value is {}, must be less than {}",
                value, self.value
            )));
        }
        Ok(value)
    }
}

/// Number nonzero validator
pub struct NumberNonZero;

impl<'a, T: Num + PartialOrd + Sync + Send + Display + 'a> OnInputValue<'a, T> for NumberNonZero {
    fn input_value(&'a self, value: T) -> InputValueResult<T> {
        if value.is_zero() {
            return Err(InputValueError::Custom(format!(
                "the value is {}, must be nonzero",
                value,
            )));
        }
        Ok(value)
    }
}

/// Number range validator
pub struct NumberRange<T> {
    min: T,
    max: T,
}

impl<'a, T: Num + PartialOrd + Sync + Send + Display + 'a> OnInputValue<'a, T> for NumberRange<T> {
    fn input_value(&'a self, value: T) -> InputValueResult<T> {
        if value < self.min || value > self.max {
            return Err(InputValueError::Custom(format!(
                "the value is {}, must be between {} and {}",
                value, self.min, self.max
            )));
        }
        Ok(value)
    }
}
