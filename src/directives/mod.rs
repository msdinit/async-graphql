use crate::context::ContextDirective;
use crate::FieldResult;

#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait OnFieldDefinition {
    async fn before_field_resolve(&self, ctx: &ContextDirective<'_>) -> FieldResult<()> {
        Ok(())
    }

    async fn after_field_resolve<T: Send>(
        &self,
        ctx: &ContextDirective<'_>,
        result: &mut T,
    ) -> FieldResult<()> {
        Ok(())
    }
}
