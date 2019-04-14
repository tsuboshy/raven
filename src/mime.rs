use std::str::FromStr;
use std::string::ToString;

use crate::charset::Charset;

use self::Mime::*;
use self::TextMime::*;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Mime {
    ApplicationOctetStream,
    ApplicationPdf,
    ImageJpeg,
    ImageGif,
    ImagePng,
    Text {
        text_type: TextMime,
        charset: Option<Charset>,
    },
    Other {
        mime_type_string: String,
    },
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum TextMime {
    ApplicationFormUrlencoded,
    ApplicationJson,
    ApplicationXml,
    ApplicationStreamJson,
    ApplicationXHtmlXml,
    TextPlain,
    TextEventStream,
    TextMarkDown,
    TextXml,
    TextOther { text_mime_type_string: String },
}

impl Mime {
    pub fn set_charset_when_text_mime(&mut self, new_charset: Charset) {
        match self {
            Text { charset, .. } => *charset = Some(new_charset),

            _ => (),
        }
    }

    pub fn is_text(&self) -> bool {
        match self {
            Text { .. } => true,
            _ => false,
        }
    }
}

impl ToString for Mime {
    fn to_string(&self) -> String {
        match self {
            Text {
                text_type,
                charset: Some(charset),
            } => match text_type {
                ApplicationFormUrlencoded => format!(
                    "application/x-www-form-urlencoded; charset={}",
                    charset.to_string()
                ),
                ApplicationJson => format!("application/json; charset={}", charset.to_string()),
                ApplicationXml => format!("application/xml; charset={}", charset.to_string()),
                ApplicationStreamJson => {
                    format!("application/stream+json; charset={}", charset.to_string())
                }
                ApplicationXHtmlXml => {
                    format!("application/xhtml+xml; charset={}", charset.to_string())
                }
                TextPlain => format!("text/plain; charset={}", charset.to_string()),
                TextEventStream => format!("text/event-stream; charset={}", charset.to_string()),
                TextMarkDown => format!("text/markdown; charset={}", charset.to_string()),
                TextXml => format!("text/xml; charset={}", charset.to_string()),
                TextOther {
                    text_mime_type_string,
                } => format!("{}; charset={}", text_mime_type_string, charset.to_string()),
            },

            Text {
                text_type,
                charset: None,
            } => match text_type {
                ApplicationFormUrlencoded => "application/x-www-form-urlencoded".to_owned(),
                ApplicationJson => "application/json".to_owned(),
                ApplicationXml => "application/xml".to_owned(),
                ApplicationStreamJson => "application/stream+json".to_owned(),
                ApplicationXHtmlXml => "application/xhtml+xml".to_owned(),
                TextPlain => "text/plain".to_owned(),
                TextEventStream => "text/event-stream".to_owned(),
                TextMarkDown => "text/markdown".to_owned(),
                TextXml => "text/xml".to_owned(),
                TextOther {
                    text_mime_type_string,
                } => text_mime_type_string.to_owned(),
            },

            ApplicationOctetStream => "application/octet-stream".to_owned(),

            ApplicationPdf => "application/pdf".to_owned(),

            ImageJpeg => "img/jpeg".to_owned(),

            ImageGif => "img/gif".to_owned(),

            ImagePng => "img/png".to_owned(),

            Other { mime_type_string } => mime_type_string.to_owned(),
        }
    }
}

impl FromStr for Mime {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_case = s.to_lowercase();
        let split_by_semicolon: Vec<&str> = lower_case.trim().split(";").collect();

        let main_type: &str = split_by_semicolon
            .first()
            .ok_or("mime type is empty".to_owned())?
            .trim();

        let charset: Option<Charset> = split_by_semicolon
            .iter()
            .filter(|param| param.contains("charset="))
            .collect::<Vec<_>>()
            .first()
            .and_then(|charset| Charset::from_str(charset).ok());

        match main_type {
            "img/jpeg" => Ok(ImageJpeg),
            "img/gif" => Ok(ImageGif),
            "img/png" => Ok(ImagePng),
            "application/octet-stream" => Ok(ApplicationOctetStream),
            "application/pdf" => Ok(ApplicationPdf),
            "application/x-www-form-urlencoded" => Ok(Text {
                charset,
                text_type: ApplicationFormUrlencoded,
            }),
            "application/json" => Ok(Text {
                charset,
                text_type: ApplicationJson,
            }),
            "application/xml" => Ok(Text {
                charset,
                text_type: ApplicationXml,
            }),
            "application/stream+json" => Ok(Text {
                charset,
                text_type: ApplicationStreamJson,
            }),
            "application/xhtml+xml" => Ok(Text {
                charset,
                text_type: ApplicationXHtmlXml,
            }),
            "text/plain" => Ok(Text {
                charset,
                text_type: TextPlain,
            }),
            "text/event-stream" => Ok(Text {
                charset,
                text_type: TextEventStream,
            }),
            "text/markdown" => Ok(Text {
                charset,
                text_type: TextMarkDown,
            }),
            "text/xml" => Ok(Text {
                charset,
                text_type: TextXml,
            }),
            other => {
                if other.contains("text") {
                    Ok(Text {
                        charset,
                        text_type: TextOther {
                            text_mime_type_string: other.to_owned(),
                        },
                    })
                } else {
                    Ok(Other {
                        mime_type_string: other.to_owned(),
                    })
                }
            }
        }
    }
}
