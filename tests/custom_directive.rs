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
    impl OnFieldDefinition for Auth {
        async fn before_field_resolve(&self, ctx: &Context<'_>) -> FieldResult<()> {
            if let Some(role) = ctx.data_opt::<Role>() {
                if *role == self.role {
                    return Ok(());
                }
            }
            Err("forbidden".into())
        }
    }

    #[Directive]
    struct GTZero {}

    impl<'a> OnInputValue<'a, i32> for GTZero {
        fn input_value(&'a self, value: i32) -> InputValueResult<i32> {
            if value <= 0 {
                return Err(InputValueError::Custom(
                    "Must be greater than 0".to_string(),
                ));
            }
            Ok(value)
        }
    }

    #[InputObject]
    struct MyInputObj {
        #[field(directive(GTZero))]
        value: i32,
    }

    struct Query {
        value: i32,
    }

    #[Object]
    impl Query {
        #[field(directive(Auth(role = "Role::Admin")))]
        async fn value(&self) -> i32 {
            10
        }

        #[field(directive(Auth(role = "Role::Admin")))]
        async fn value_ref(&self) -> &i32 {
            &self.value
        }

        async fn test_arg(&self, #[arg(directive(GTZero))] input: i32) -> i32 {
            input
        }

        async fn test_input_obj(&self, input: MyInputObj) -> i32 {
            input.value
        }
    }

    let schema = Schema::new(Query { value: 99 }, EmptyMutation, EmptySubscription);

    assert_eq!(
        QueryBuilder::new("{ value }")
            .data(Role::Admin)
            .execute(&schema)
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "value": 10,
        })
    );

    assert_eq!(
        QueryBuilder::new("{ value }")
            .data(Role::Guest)
            .execute(&schema)
            .await
            .unwrap_err(),
        Error::Query {
            pos: Pos { line: 1, column: 3 },
            path: Some(serde_json::json!(["value"])),
            err: QueryError::FieldError {
                err: "forbidden".to_string(),
                extended_error: None,
            },
        }
    );

    assert_eq!(
        QueryBuilder::new("{ valueRef }")
            .data(Role::Admin)
            .execute(&schema)
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "valueRef": 99,
        })
    );

    assert_eq!(
        QueryBuilder::new("{ testArg(input: 10) }")
            .execute(&schema)
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "testArg": 10,
        })
    );

    assert_eq!(
        QueryBuilder::new("{ testArg(input: -10) }")
            .execute(&schema)
            .await
            .unwrap_err(),
        Error::Query {
            pos: Pos {
                column: 18,
                line: 1
            },
            path: None,
            err: QueryError::ParseInputValue {
                reason: "Must be greater than 0".to_string()
            }
        }
    );

    assert_eq!(
        QueryBuilder::new("{ testInputObj(input: { value: 10 }) }")
            .execute(&schema)
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "testInputObj": 10,
        })
    );

    assert_eq!(
        QueryBuilder::new("{ testInputObj(input: { value: -10 }) }")
            .execute(&schema)
            .await
            .unwrap_err(),
        Error::Query {
            pos: Pos {
                column: 23,
                line: 1
            },
            path: None,
            err: QueryError::ParseInputValue {
                reason: "Must be greater than 0".to_string()
            }
        }
    );
}
