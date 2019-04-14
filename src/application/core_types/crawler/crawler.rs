use super::{encoding::Encoding, request::Method, CrawlerError, CrawlerRequest, CrawlerResult};
use crate::mime::{Mime, TextMime};
use chrono::Local;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Error, Response};
use std::{
    collections::HashMap,
    hash::Hash,
    io::{Error as IOError, ErrorKind},
    str::FromStr,
    time::Duration,
};

pub trait Crawler {
    fn crawl(&self, request: &CrawlerRequest) -> Result<CrawlerResult, CrawlerError> {
        crawler_default_impl(request)
    }
}

/// execute crawl using reqwest.
/// if request ends up server error or timeout,
/// this function retries to request up to request.retry_max
pub fn crawler_default_impl(request: &CrawlerRequest) -> Result<CrawlerResult, CrawlerError> {
    let header_maps = create_header_map(&request.header)?;

    let client = Client::builder()
        .timeout(Duration::from_secs(request.timeout.into()))
        .default_headers(header_maps)
        .build()
        .map_err(|err| other_error!("failed to build http client: {}", err))?;

    let url = match create_query_strings(&request.query_params) {
        Some(query_strings) => format!("{}?{}", &request.url, query_strings),
        None => request.url.to_owned(),
    };

    let mut retry_count: u8 = 0;
    let start_datetime = Local::now().timestamp_millis();

    loop {
        let mut response_result: Result<Response, Error> = match &request.method {
            Method::Get => client.get(&url).send(),
            Method::Post => client.post(&url).form(&request.body_params).send(),
        };

        match response_result {
            Ok(ref mut response) => {
                let end_datetime = Local::now().timestamp_millis();
                let mut response_body: Vec<u8> = vec![];
                response.copy_to(&mut response_body).map_err(|e: Error| {
                    CrawlerError::OtherError {
                        error_detail: e.to_string(),
                    }
                })?;

                let mut response_content_type = response
                    .headers()
                    .get("Content-Type")
                    .and_then(|header_value: &HeaderValue| header_value.to_str().ok())
                    .and_then(|mime_str| Mime::from_str(mime_str).ok())
                    .or_else(|| {
                        text_plain_if_input_charset_setting_exists(&request.encoding_setting)
                    })
                    .unwrap_or(Mime::ApplicationOctetStream);

                let body_converted_encoding: Vec<u8> = convert_encoding_if_need(
                    &request.encoding_setting,
                    &mut response_content_type,
                    response_body,
                )?;

                let raven_response = CrawlerResult {
                    response_status: response.status().as_u16(),
                    response_header: header_map_to_hash_map(response.headers()),
                    response_body: body_converted_encoding,
                    mills_takes_to_complete_to_request: end_datetime - start_datetime,
                    retry_count,
                    response_content_type,
                };

                if response.status().is_success() {
                    return Ok(raven_response);
                } else if response.status().is_client_error() {
                    return Err(CrawlerError::ClientError(raven_response));
                } else if response.status().is_server_error() && retry_count >= request.max_retry {
                    return Err(CrawlerError::ServerError(raven_response));
                } else {
                    retry_count += 1;
                    continue;
                }
            }
            Err(error) => {
                let cast_to_hyper_error = error
                    .get_ref()
                    .and_then(|e| e.downcast_ref::<IOError>())
                    .map(|e: &IOError| e.kind());

                if let Some(ErrorKind::TimedOut) = cast_to_hyper_error {
                    if retry_count >= request.max_retry {
                        return Err(CrawlerError::TimeoutError {
                            timeout_second: request.timeout,
                            retry_count,
                        });
                    } else {
                        retry_count += 1;
                        continue;
                    }
                } else {
                    return Err(other_error!("request error: {}", error));
                }
            }
        }
    }
}

fn convert_encoding_if_need(
    encoding_setting: &Option<Encoding>,
    detected_mime: &mut Mime,
    target: Vec<u8>,
) -> Result<Vec<u8>, CrawlerError> {
    if let Some(Encoding { input, output }) = encoding_setting {
        match detected_mime {
            Mime::Text { charset, .. } => {
                let from_charset = input
                    .clone()
                    .or_else(|| charset.clone())
                    .ok_or(CrawlerError::CharsetConversionError {
                    error_detail:
                        "failed to detect charset. please configure config charset in yaml file."
                            .to_owned(),
                })?;
                *charset = Some(output.clone());
                Ok(from_charset.convert_to(output, target))
            }

            _ => Ok(target),
        }
    } else {
        Ok(target)
    }
}

fn text_plain_if_input_charset_setting_exists(encoding_setting: &Option<Encoding>) -> Option<Mime> {
    if let Some(Encoding {
        input: Some(input_encoding),
        ..
    }) = encoding_setting
    {
        Some(Mime::Text {
            text_type: TextMime::TextPlain,
            charset: Some(input_encoding.clone()),
        })
    } else {
        None
    }
}

fn create_query_strings<T>(param_map: &HashMap<T, T>) -> Option<String>
where
    T: Eq + Hash + AsRef<str>,
{
    if param_map.is_empty() {
        return None;
    }

    let mut query_strings: Vec<String> = vec![];
    for (key, val) in param_map {
        query_strings.push(format!("{}={}", key.as_ref(), val.as_ref()));
    }

    Some(query_strings.join("&"))
}

fn create_header_map<T>(headers: &HashMap<T, T>) -> Result<HeaderMap, CrawlerError>
where
    T: Eq + Hash + AsRef<str>,
{
    let mut header_map: HeaderMap<HeaderValue> = HeaderMap::with_capacity(headers.len());

    for (key, val) in headers {
        let parsed_header_key = HeaderName::from_str(key.as_ref());
        let parsed_header_val = HeaderValue::from_str(val.as_ref());
        match (parsed_header_key, parsed_header_val) {
            (Ok(header_name), Ok(header_val)) => {
                header_map.insert(header_name, header_val);
            }

            (Err(err), _) => return Err(other_error!("failed to parse header key: {}", err)),

            (_, Err(err)) => return Err(other_error!("failed to parse header val: {}", err)),
        }
    }
    Ok(header_map)
}

fn header_map_to_hash_map(header_map: &HeaderMap) -> HashMap<String, String> {
    let mut string_map: HashMap<String, String> = HashMap::new();
    for (key, val) in header_map.iter() {
        if let Ok(header_val) = String::from_utf8(val.as_ref().to_vec()) {
            string_map.insert(key.as_str().to_owned(), header_val);
        }
    }
    string_map
}

//#[ignore]
#[test]
fn try_crawler() {
    struct TestCrawler;
    impl Crawler for TestCrawler {};

    let raven_request = CrawlerRequest {
        url: "https://yakkun.com/sm/zukan/n213".to_owned(),
        method: Method::Get,
        header: hashmap!("User-Agent".to_owned() => "application".to_owned()),
        timeout: 1,
        max_retry: 1,
        query_params: HashMap::new(),
        body_params: HashMap::new(),
        encoding_setting: None,
    };

    let response: CrawlerResult = TestCrawler.crawl(&raven_request).unwrap();
    dbg!(&response.response_header);

    use std::io::Write;
    let mut file = std::fs::File::create("/var/tmp/crawler_test.html").unwrap();
    file.write(&response.response_body).unwrap();
}
