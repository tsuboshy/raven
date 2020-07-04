use reqwest::{Client, Error as RequwestError};
use serde_json::Value;
use std::fmt::{Display, Formatter};
use std::time::Duration;

/// Since rs-es crate does not support creation of index template yet,
/// raven application do PUT request to es to create index template.

pub fn create_es_index_if_not_exists(
    url: &str,
    template_name: &str,
    template_json: &Value,
) -> Result<(), CreateEsIndexTemplateError> {
    let rest_client = Client::builder()
        .timeout(Some(Duration::from_secs(5)))
        .build()?;

    let template_endpoint = format!("{}/_template/{}", url.trim_end_matches("/"), template_name);

    if template_already_exists(&rest_client, &template_endpoint)? {
        Ok(())
    } else {
        create_index_template(&rest_client, &template_endpoint, &template_json)?;
        Ok(())
    }
}

/// es reference: https://www.elastic.co/guide/en/elasticsearch/reference/current/indices-templates.html#indices-templates-exists
fn template_already_exists(
    rest_client: &Client,
    template_endpoint: &str,
) -> Result<bool, RequwestError> {
    let head_response_status_code = rest_client
        .head(template_endpoint)
        .send()?
        .status()
        .as_u16();

    Ok(head_response_status_code == 200)
}

/// es reference: https://www.elastic.co/guide/en/elasticsearch/reference/current/indices-templates.html#indices-templates
fn create_index_template(
    rest_client: &Client,
    template_endpoint: &str,
    template_json: &Value,
) -> Result<(), RequwestError> {
    rest_client
        .put(template_endpoint)
        .json(template_json)
        .send()?
        .error_for_status()?;

    Ok(())
}

#[derive(Debug)]
pub struct CreateEsIndexTemplateError(String);

impl From<RequwestError> for CreateEsIndexTemplateError {
    fn from(e: RequwestError) -> Self {
        CreateEsIndexTemplateError(e.to_string())
    }
}

impl Display for CreateEsIndexTemplateError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "failed to create elasticsearch index template: {}",
            self.0
        )
    }
}
