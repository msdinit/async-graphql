//! Module for custom directives

pub mod validators;

use crate::{Context, FieldResult, InputValueResult};

/// Custom directive on field.
#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait OnFieldDefinition {
    /// Called when resolving field start.
    async fn before_field_resolve(&self, ctx: &Context<'_>) -> FieldResult<()>;
}

/// Custom directive on input value.
///
/// `Object` field parameters, `InputObject` fields are input values.
#[allow(unused_variables)]
pub trait OnInputValue<'a, T: Sync + Send + 'a> {
    /// Called when input value parsed.
    fn input_value(&'a self, value: T) -> InputValueResult<T>;
}
