use async_graphql::directives::*;
use async_graphql::*;

#[async_std::test]
pub async fn test_custom_directive() {
    #[Enum]
    enum Role {
        Guest,
        Member,
        Admin,
    }

    #[Directive]
    struct Auth {
        role: Role,
    }

    #[async_trait::async_trait]
    impl<T: Sync + Send + 'static> OnFieldDefinition<T> for Auth {
        async fn before_field_resolve(&self, ctx: &Context<'_>) -> FieldResult<()> {
            if let Some(role) = ctx.data_opt::<Role>() {
                if *role == self.role {
                    return Ok(());
                }
            }
            Err("forbidden".into())
        }
    }

    struct Query;

    #[Object]
    impl Query {
        #[field(directive(Auth(role = "Role::Admin")))]
        async fn value(&self) -> i32 {
            10
        }
    }
}
