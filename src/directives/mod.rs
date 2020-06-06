use crate::{Context, FieldResult, InputValueResult, InputValueType};

#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait OnFieldDefinition {
    async fn before_field_resolve(&self, ctx: &Context<'_>) -> FieldResult<()> {
        Ok(())
    }
}

#[allow(unused_variables)]
pub trait OnInputValue<'a, T: InputValueType + Sync + Send + 'a> {
    fn input_value(&'a self, value: T) -> InputValueResult<T> {
        Ok(value)
    }
}
