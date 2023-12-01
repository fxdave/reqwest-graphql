use crate::error::{GraphQLError, GraphQLErrorMessage};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

pub struct GQLClient<'a> {
    endpoint: &'a str,
    header_map: HeaderMap,
}

#[derive(Serialize)]
struct RequestBody<'a, T: Serialize> {
    query: &'a str,
    variables: T,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum GraphQLResponse<T> {
    ConventionResponse {
        data: Option<T>,
        errors: Option<Vec<GraphQLErrorMessage>>,
    },
    UnconventionalResponse(serde_json::Value),
}

impl<'a> GQLClient<'a> {
    pub fn new(endpoint: &'a str) -> Self {
        Self {
            endpoint,
            header_map: HeaderMap::new(),
        }
    }

    pub fn new_with_headers(endpoint: &'a str, headers: HashMap<&str, &str>) -> Self {
        let mut header_map = HeaderMap::new();

        for (str_key, str_value) in headers {
            let key = HeaderName::from_str(str_key).unwrap();
            let val = HeaderValue::from_str(str_value).unwrap();

            header_map.insert(key, val);
        }

        Self {
            endpoint,
            header_map,
        }
    }

    pub async fn query<K>(&self, query: &'a str) -> Result<K, GraphQLError>
    where
        K: for<'de> Deserialize<'de>,
    {
        self.query_with_vars::<K, ()>(query, ()).await
    }

    pub async fn query_with_vars<K, T: Serialize>(
        &self,
        query: &'a str,
        variables: T,
    ) -> Result<K, GraphQLError>
    where
        K: for<'de> Deserialize<'de>,
    {
        let client = Client::new();
        let body = RequestBody { query, variables };

        let request = client
            .post(self.endpoint)
            .json(&body)
            .headers(self.header_map.clone());

        let raw_response = request.send().await?;
        let json_response = raw_response.json::<GraphQLResponse<K>>().await;

        // Check whether JSON is parsed successfully
        match json_response {
            Ok(GraphQLResponse::ConventionResponse { data, errors: None }) => Ok(data.unwrap()),
            Ok(GraphQLResponse::ConventionResponse {
                errors: Some(errors),
                ..
            }) => Err(GraphQLError::from_json(errors)),
            Ok(GraphQLResponse::UnconventionalResponse(value)) => Err(GraphQLError {
                message: "Couldn't parse the result.".into(),
                json: Some(vec![GraphQLErrorMessage::UnconventionalError(value)]),
            }),
            Err(_e) => Err(GraphQLError::from_str("Failed to parse response").unwrap()),
        }
    }
}
