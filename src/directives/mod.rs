use crate::{Context, Directive, FieldResult};

#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait OnFieldDefinition<T: Sync + Send + 'static>: Directive {
    async fn before_field_resolve(&self, ctx: &Context<'_>) -> FieldResult<()> {
        Ok(())
    }

    async fn after_field_resolve(
        &self,
        ctx: &Context<'_>,
        result: &mut T,
    ) -> FieldResult<()> {
        Ok(())
    }
}
