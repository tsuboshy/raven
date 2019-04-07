use crate::charset::Charset;
use std::string::ToString;

#[derive(Debug, Eq, PartialEq)]
pub enum Mime {
    ApplicationFormUrlencoded { charset: Charset },
    ApplicationOctetStream,
    ApplicationJson { charset: Charset },
    ApplicationPdf,
    ApplicationXml { charset: Charset },
    ApplicationStreamJson { charset: Charset },
    ApplicationXHtmlXml { charset: Charset },
    ImageJpeg,
    ImageGif,
    ImagePng,
    TextPlain { charset: Charset },
    TextEventStream { charset: Charset },
    TextMarkDown { charset: Charset },
    TextXml { charset: Charset },
    Other { charset_string: String },
}

use self::Mime::*;

impl ToString for Mime {
    fn to_string(&self) -> String {
        match self {
            ApplicationFormUrlencoded { charset } => format!(
                "application/x-www-form-urlencoded; charset={}",
                charset.to_string()
            ),
            ApplicationOctetStream => "application/octet-stream".to_owned(),

            ApplicationJson { charset } => {
                format!("application/json; charset={}", charset.to_string())
            }
            ApplicationPdf => "application/pdf".to_owned(),

            ApplicationXml { charset } => {
                format!("application/xml; charset={}", charset.to_string())
            }
            ApplicationStreamJson { charset } => {
                format!("application/stream+json; charset={}", charset.to_string())
            }
            ApplicationXHtmlXml { charset } => {
                format!("application/xhtml+xml; charset={}", charset.to_string())
            }

            ImageJpeg => "img/jpeg".to_owned(),

            ImageGif => "img/gif".to_owned(),

            ImagePng => "img/png".to_owned(),

            TextPlain { charset } => format!("text/plain; charset={}", charset.to_string()),

            TextEventStream { charset } => {
                format!("text/event-stream; charset={}", charset.to_string())
            }
            TextMarkDown { charset } => format!("text/markdown; charset={}", charset.to_string()),

            TextXml { charset } => format!("text/xml; charset={}", charset.to_string()),

            Other { charset_string } => charset_string.to_owned(),
        }
    }
}
