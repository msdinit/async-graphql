//! A helper module that supports HTTP

mod graphiql_source;
mod into_query_builder;
mod multipart_stream;
mod playground_source;
mod stream_body;

use itertools::Itertools;

pub use graphiql_source::graphiql_source;
pub use multipart_stream::multipart_stream;
pub use playground_source::{playground_source, GraphQLPlaygroundConfig};
pub use stream_body::StreamBody;

use crate::query::{IntoBatchQueryBuilder, IntoQueryBuilderOpts, BatchQueryBuilder, IntoQueryBuilder, QueryBuilderTypes, QueryBuilder};
use crate::{Error, ParseRequestError, Pos, QueryError, QueryResponse, Result, Variables, BatchQueryResponse};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer, Deserialize, de};

/// Deserializable GraphQL Request object
#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct GQLRequest {
    /// Query source
    pub query: String,

    /// Operation name for this query
    #[serde(rename = "operationName")]
    pub operation_name: Option<String>,

    /// Variables for this query
    pub variables: Option<serde_json::Value>,
}

#[async_trait::async_trait]
impl IntoQueryBuilder for GQLRequest {
    async fn into_query_builder_opts(
        self,
        _opts: &IntoQueryBuilderOpts,
    ) -> std::result::Result<QueryBuilder, ParseRequestError> {
        let mut builder = QueryBuilder::new(self.query);
        if let Some(operation_name) = self.operation_name {
            builder = builder.operation_name(operation_name);
        }
        if let Some(variables) = self.variables {
            if let Ok(variables) = Variables::parse_from_json(variables) {
                builder = builder.variables(variables);
            }
        }
        Ok(builder)
    }
}

/// Batch support for GraphQL requests, which is either a single query, or an array of queries
#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub enum BatchGQLRequest {
    /// Single query
    Single(GQLRequest),
    /// Non-empty array of queries
    #[serde(deserialize_with = "deserialize_non_empty_vec")]
    Batch(Vec<GQLRequest>)
}

fn deserialize_non_empty_vec<'de, D, T>(deserializer: D) -> std::result::Result<Vec<T>, D::Error>
    where
        D: de::Deserializer<'de>,
        T: Deserialize<'de>,
{
    use de::Error as _;

    let v = Vec::<T>::deserialize(deserializer)?;
    if v.is_empty() {
        Err(D::Error::invalid_length(0, &"a positive integer"))
    } else {
        Ok(v)
    }
}

#[async_trait::async_trait]
impl IntoBatchQueryBuilder for BatchGQLRequest {
    async fn into_batch_query_builder_opts(
        self,
        _opts: &IntoQueryBuilderOpts,
    ) -> std::result::Result<BatchQueryBuilder, ParseRequestError> {
        match self {
            BatchGQLRequest::Single(request) => Ok(
                BatchQueryBuilder{ builder: QueryBuilderTypes::Single(request.into_query_builder_opts(_opts).await?), ctx_data: None }
            ),
            BatchGQLRequest::Batch(requests) => {
                let futures = requests.into_iter().map(|request| request.into_query_builder_opts(_opts));
                Ok(BatchQueryBuilder{ builder: QueryBuilderTypes::Batch(futures::future::try_join_all(futures).await?), ctx_data: None })
            }
        }
    }
}

/// Serializable GraphQL Response object
pub struct GQLResponse(pub Result<QueryResponse>);

impl Serialize for GQLResponse {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match &self.0 {
            Ok(res) => {
                let mut map = serializer.serialize_map(None)?;
                if let Some(label) = &res.label {
                    map.serialize_key("label")?;
                    map.serialize_value(label)?;
                }
                if let Some(path) = &res.path {
                    map.serialize_key("path")?;
                    map.serialize_value(path)?;
                }
                map.serialize_key("data")?;
                map.serialize_value(&res.data)?;
                if res.extensions.is_some() {
                    map.serialize_key("extensions")?;
                    map.serialize_value(&res.extensions)?;
                }
                map.end()
            }
            Err(err) => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_key("errors")?;
                map.serialize_value(&GQLError(err))?;
                map.end()
            }
        }
    }
}

/// Serializable GraphQL Response object
#[derive(Serialize)]
#[serde(untagged)]
pub enum BatchGQLResponse {
    Single(GQLResponse),
    Batch(Vec<GQLResponse>)
}

impl From<BatchQueryResponse> for BatchGQLResponse {
    fn from(item: BatchQueryResponse) -> Self {
        match item {
            BatchQueryResponse::Single(resp) => BatchGQLResponse::Single(GQLResponse(resp)),
            BatchQueryResponse::Batch(responses) => BatchGQLResponse::Batch(responses.into_iter().map(GQLResponse).collect())
        }
    }
}


// impl Serialize for BatchGQLResponse {
//     fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
//         match &self.0 {
//             BatchQueryResponse::Single(resp) => resp.seria
//             Ok(res) => {
//                 let mut map = serializer.serialize_map(None)?;
//                 if let Some(label) = &res.label {
//                     map.serialize_key("label")?;
//                     map.serialize_value(label)?;
//                 }
//                 if let Some(path) = &res.path {
//                     map.serialize_key("path")?;
//                     map.serialize_value(path)?;
//                 }
//                 map.serialize_key("data")?;
//                 map.serialize_value(&res.data)?;
//                 if res.extensions.is_some() {
//                     map.serialize_key("extensions")?;
//                     map.serialize_value(&res.extensions)?;
//                 }
//                 map.end()
//             }
//             Err(err) => {
//                 let mut map = serializer.serialize_map(None)?;
//                 map.serialize_key("errors")?;
//                 map.serialize_value(&GQLError(err))?;
//                 map.end()
//             }
//         }
//     }
// }

/// Serializable error type
pub struct GQLError<'a>(pub &'a Error);

impl<'a> Serialize for GQLError<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            Error::Parse(err) => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                seq.serialize_element(&serde_json::json! ({
                    "message": err.message,
                    "locations": [{"line": err.pos.line, "column": err.pos.column}]
                }))?;
                seq.end()
            }
            Error::Query { pos, path, err } => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                if let QueryError::FieldError {
                    err,
                    extended_error,
                } = err
                {
                    let mut map = serde_json::Map::new();

                    map.insert("message".to_string(), err.to_string().into());
                    map.insert(
                        "locations".to_string(),
                        serde_json::json!([{"line": pos.line, "column": pos.column}]),
                    );

                    if let Some(path) = path {
                        map.insert("path".to_string(), path.clone());
                    }

                    if let Some(obj @ serde_json::Value::Object(_)) = extended_error {
                        map.insert("extensions".to_string(), obj.clone());
                    }

                    seq.serialize_element(&serde_json::Value::Object(map))?;
                } else {
                    seq.serialize_element(&serde_json::json!({
                        "message": err.to_string(),
                        "locations": [{"line": pos.line, "column": pos.column}]
                    }))?;
                }
                seq.end()
            }
            Error::Rule { errors } => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                for error in errors {
                    seq.serialize_element(&serde_json::json!({
                        "message": error.message,
                        "locations": error.locations.iter().map(|pos| serde_json::json!({"line": pos.line, "column": pos.column})).collect_vec(),
                    }))?;
                }
                seq.end()
            }
        }
    }
}

struct GQLErrorPos<'a>(&'a Pos);

impl<'a> Serialize for GQLErrorPos<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("line", &self.0.line)?;
        map.serialize_entry("column", &self.0.column)?;
        map.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pos;
    use serde_json::json;

    #[test]
    fn test_request() {
        let request: GQLRequest = serde_json::from_value(json! ({
            "query": "{ a b c }"
        }))
        .unwrap();
        assert!(request.variables.is_none());
        assert!(request.operation_name.is_none());
        assert_eq!(request.query, "{ a b c }");
    }

    #[test]
    fn test_batch_request() {
        let request: BatchGQLRequest = serde_json::from_value(json! ([{
            "query": "{ a b c }"
        }, {
            "query": "{ d e f }"
        }]))
            .unwrap();
        if let BatchGQLRequest::Batch(requests) = request {
            assert_eq!(requests[0].query, "{ a b c }");
            assert_eq!(requests[1].query, "{ d e f }");
        } else {
            panic!("Batch query not parsed as a batch")
        }
    }

    #[test]
    fn test_batch_request_single_operation() {
        let request: BatchGQLRequest = serde_json::from_value(json! ({
            "query": "{ a b c }"
        }))
            .unwrap();
        if let BatchGQLRequest::Single(request) = request {
            assert_eq!(request.query, "{ a b c }");
        } else {
            panic!("Batch query not parsed as a batch")
        }
    }

    #[test]
    fn test_request_with_operation_name() {
        let request: GQLRequest = serde_json::from_value(json! ({
            "query": "{ a b c }",
            "operationName": "a"
        }))
        .unwrap();
        assert!(request.variables.is_none());
        assert_eq!(request.operation_name.as_deref(), Some("a"));
        assert_eq!(request.query, "{ a b c }");
    }

    #[test]
    fn test_request_with_variables() {
        let request: GQLRequest = serde_json::from_value(json! ({
            "query": "{ a b c }",
            "variables": {
                "v1": 100,
                "v2": [1, 2, 3],
                "v3": "str",
            }
        }))
        .unwrap();
        assert_eq!(
            request.variables,
            Some(json!({
                "v1": 100,
                "v2": [1, 2, 3],
                "v3": "str",
            }))
        );
        assert!(request.operation_name.is_none());
        assert_eq!(request.query, "{ a b c }");
    }

    #[test]
    fn test_response_data() {
        let resp = GQLResponse(Ok(QueryResponse {
            label: None,
            path: None,
            data: json!({"ok": true}),
            extensions: None,
            cache_control: Default::default(),
        }));
        assert_eq!(
            serde_json::to_value(resp).unwrap(),
            json! ({
                "data": {
                    "ok": true,
                }
            })
        );
    }

    #[test]
    fn test_field_error_with_extension() {
        let err = Error::Query {
            pos: Pos {
                line: 10,
                column: 20,
            },
            path: None,
            err: QueryError::FieldError {
                err: "MyErrorMessage".to_owned(),
                extended_error: Some(json!({
                    "code": "MY_TEST_CODE"
                })),
            },
        };

        let resp = GQLResponse(Err(err.into()));

        assert_eq!(
            serde_json::to_value(resp).unwrap(),
            json!({
                "errors": [{
                    "message":"MyErrorMessage",
                    "extensions": {
                        "code": "MY_TEST_CODE"
                    },
                    "locations": [{"line": 10, "column": 20}]
                }]
            })
        );
    }

    #[test]
    fn test_response_error_with_pos() {
        let resp = GQLResponse(Err(Error::Query {
            pos: Pos {
                line: 10,
                column: 20,
            },
            path: None,
            err: QueryError::NotSupported,
        }));
        assert_eq!(
            serde_json::to_value(resp).unwrap(),
            json!({
                "errors": [{
                    "message":"Not supported.",
                    "locations": [
                        {"line": 10, "column": 20}
                    ]
                }]
            })
        );
    }
}
