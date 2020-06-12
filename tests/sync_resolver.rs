use async_graphql::*;

#[async_std::test]
pub async fn test_sync_resolver() {
    #[SimpleObject(sync)]
    struct MySimpleObject {
        value1: i32,
        #[field(ref)]
        value2: i32,
    }

    struct MyObj;

    #[Object]
    impl MyObj {
        fn value1(&self) -> i32 {
            10
        }
    }

    #[Interface(field(name = "value1", type = "i32", sync))]
    enum MyInterface {
        MySimpleObject(MySimpleObject),
        MyObj(MyObj),
    }

    struct Query;

    #[Object]
    impl Query {
        fn obj(&self) -> MyInterface {
            MyObj.into()
        }

        fn simple_obj(&self) -> MyInterface {
            MySimpleObject {
                value1: 10,
                value2: 20,
            }
            .into()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let resp = schema
        .execute(
            r#"
            {
                obj {
                    value1
                }
                simpleObj {
                    value1
                    ... on MySimpleObject {
                        value2
                    }
                }
            }
        "#,
        )
        .await
        .unwrap();
    assert_eq!(
        resp.data,
        serde_json::json!({
            "obj": { "value1": 10 },
            "simpleObj": { "value1": 10, "value2": 20 },
        })
    );
}
