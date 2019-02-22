use super::{
    request::Method as RavenMethod, request::Request, response::CrawlerError,
    response::Response as RavenResponse,
};

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

/// this trait supply crawl function
pub trait Crawler {
    /// execute crawl.
    fn crawl(request: &Request) -> Result<RavenResponse, CrawlerError> {
        default_impl_for_crawler(request)
    }
}

/// execute crawl.
/// if request ends up server error or timeout,
/// this default retry to request up to request.retry_max
pub fn default_impl_for_crawler(request: &Request) -> Result<RavenResponse, CrawlerError> {
    let header_maps = create_header_map(&request.header)?;

    let client_result = Client::builder()
        .timeout(Duration::from_secs(request.timeout.into()))
        .default_headers(header_maps)
        .build();

    let client: Client = match client_result {
        Ok(client) => client,
        Err(err) => return Err(other_error!("failed to build http client: {}", err)),
    };

    let url = match create_query_strings(&request.query_params) {
        Some(query_strings) => format!("{}?{}", &request.url, query_strings),
        None => request.url.to_owned(),
    };

    let mut retry_count: u8 = 0;
    let start_datetime = Local::now().timestamp_millis();

    loop {
        let mut response_result: Result<Response, Error> = match &request.method {
            RavenMethod::Get => client.get(&url).send(),
            RavenMethod::Post => client.post(&url).form(&request.body_params).send(),
        };

        match response_result {
            Ok(ref mut response) => {
                let end_datetime = Local::now().timestamp_millis();
                let mut response_body: Vec<u8> = vec![];
                response.copy_to(&mut response_body).unwrap();
                let raven_respone = RavenResponse {
                    status: response.status().as_u16(),
                    header: HashMap::new(),
                    body: response_body,
                    mills_takes_to_complete_to_request: end_datetime - start_datetime,
                    retry_count,
                };

                if response.status().is_success() {
                    return Ok(raven_respone);
                } else if response.status().is_client_error() {
                    return Err(CrawlerError::ClientError(raven_respone));
                } else if response.status().is_client_error() && retry_count >= request.max_retry {
                    return Err(CrawlerError::ServerError(raven_respone));
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

pub fn create_query_strings<T>(param_map: &HashMap<T, T>) -> Option<String>
where
    T: Eq + Hash + AsRef<str>,
{
    if param_map.is_empty() {
        return None;
    }

    let mut query_strings: Vec<String> = vec![];
    for (key, val) in param_map {
        query_strings.push([key.as_ref(), val.as_ref()].join("="));
    }

    Some(query_strings.join("&"))
}

pub fn create_header_map<T>(headers: &HashMap<T, T>) -> Result<HeaderMap, CrawlerError>
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

#[ignore]
#[test]
fn try_crawler() {
    struct TestCrawler;
    impl Crawler for TestCrawler {};

    let raven_request = Request {
        url: "http://inet-ip.info".to_owned(),
        method: RavenMethod::Get,
        header: hashmap!("User-Agent".to_owned() => "raven".to_owned()),
        ouput_methods: vec![],
        input_charset: "UTF-8".to_owned(),
        output_charset: "UTF-8".to_owned(),
        timeout: 1,
        max_retry: 1,
        val_map: HashMap::new(),
        query_params: HashMap::new(),
        body_params: HashMap::new(),
    };

    let response: RavenResponse = TestCrawler::crawl(&raven_request).unwrap();

    use std::io::Write;
    let mut file = std::fs::File::create("/var/tmp/crawler_test.html").unwrap();
    file.write(&response.body).unwrap();
}
