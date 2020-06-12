use async_graphql::*;

#[async_std::test]
pub async fn test_sync_resolver() {
    #[SimpleObject(sync)]
    struct MySimpleObject {
        value1: i32,
        #[field(ref)]
        value2: i32,
    }
}