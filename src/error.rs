use reqwest::Error;
use serde::Deserialize;
use std::convert::Infallible;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

pub struct GraphQLError {
    pub message: String,
    pub json: Option<Vec<GraphQLErrorMessage>>,
}

// https://spec.graphql.org/June2018/#sec-Errors
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum GraphQLErrorMessage {
    ConventionalError {
        message: String,
        locations: Option<Vec<GraphQLErrorLocation>>,
        extensions: Option<serde_json::Value>,
        path: Option<Vec<GraphQLErrorPathParam>>,
    },
    UnconventionalError(serde_json::Value),
}

impl GraphQLErrorMessage {
    fn message(&self) -> String {
        match self {
            Self::ConventionalError { message, .. } => message.to_string(),
            Self::UnconventionalError(value) => value.to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct GraphQLErrorLocation {
    pub line: u32,
    pub column: u32,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum GraphQLErrorPathParam {
    String(String),
    Number(u32),
}

impl FromStr for GraphQLError {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            message: String::from(s),
            json: None,
        })
    }
}

impl std::error::Error for GraphQLError {}

impl GraphQLError {
    pub fn from_json(json: Vec<GraphQLErrorMessage>) -> Self {
        Self {
            message: String::from("Look at json field for more details"),
            json: Some(json),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn json(&self) -> &Option<Vec<GraphQLErrorMessage>> {
        &self.json
    }
}

fn format(err: &GraphQLError, f: &mut Formatter<'_>) -> fmt::Result {
    // Print the main error message
    writeln!(f, "\nGQLClient Error: {}", err.message)?;

    // Check if query errors have been received
    if err.json.is_none() {
        return Ok(());
    }

    let errors = err.json.as_ref();

    for err in errors.unwrap() {
        writeln!(f, "Message: {}", err.message())?;
    }

    Ok(())
}

impl fmt::Display for GraphQLError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        format(self, f)
    }
}

impl fmt::Debug for GraphQLError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format(self, f)
    }
}

impl std::convert::From<reqwest::Error> for GraphQLError {
    fn from(error: Error) -> Self {
        Self {
            message: error.to_string(),
            json: None,
        }
    }
}
