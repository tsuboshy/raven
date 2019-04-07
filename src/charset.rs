use encoding_rs::*;
use serde::de::{Deserialize, Deserializer, Error, Unexpected, Visitor};
use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;
use std::string::ToString;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Charset {
    Big5,
    EucJp,
    EucKr,
    Gbk,
    Ibm866,
    Iso2022Jp,
    Iso885910,
    Iso885913,
    Iso885914,
    Iso885915,
    Iso885916,
    Iso88592,
    Iso88593,
    Iso88594,
    Iso88595,
    Iso88596,
    Iso88597,
    Iso88598,
    Iso88598I,
    Koi8R,
    Koi8U,
    ShiftJis,
    Utf16be,
    Utf16le,
    Utf8,
    Gb18030,
    Macintosh,
    Replacement,
    Windows1250,
    Windows1251,
    Windows1252,
    Windows1253,
    Windows1254,
    Windows1255,
    Windows1256,
    Windows1257,
    Windows1258,
    Windows874,
    XMacCyrillic,
}

use self::Charset::*;

impl Charset {
    pub fn get_encoding(&self) -> &'static Encoding {
        match self {
            Big5 => BIG5,
            EucJp => EUC_JP,
            EucKr => EUC_KR,
            Gbk => GBK,
            Ibm866 => IBM866,
            Iso2022Jp => ISO_2022_JP,
            Iso885910 => ISO_8859_10,
            Iso885913 => ISO_8859_13,
            Iso885914 => ISO_8859_14,
            Iso885915 => ISO_8859_15,
            Iso885916 => ISO_8859_16,
            Iso88592 => ISO_8859_2,
            Iso88593 => ISO_8859_3,
            Iso88594 => ISO_8859_4,
            Iso88595 => ISO_8859_5,
            Iso88596 => ISO_8859_6,
            Iso88597 => ISO_8859_7,
            Iso88598 => ISO_8859_8,
            Iso88598I => ISO_8859_8_I,
            Koi8R => KOI8_R,
            Koi8U => KOI8_U,
            ShiftJis => SHIFT_JIS,
            Utf16be => UTF_16BE,
            Utf16le => UTF_16LE,
            Utf8 => UTF_8,
            Gb18030 => GB18030,
            Macintosh => MACINTOSH,
            Replacement => REPLACEMENT,
            Windows1250 => WINDOWS_1250,
            Windows1251 => WINDOWS_1251,
            Windows1252 => WINDOWS_1252,
            Windows1253 => WINDOWS_1253,
            Windows1254 => WINDOWS_1254,
            Windows1255 => WINDOWS_1255,
            Windows1256 => WINDOWS_1256,
            Windows1257 => WINDOWS_1257,
            Windows1258 => WINDOWS_1258,
            Windows874 => WINDOWS_874,
            XMacCyrillic => X_MAC_CYRILLIC,
        }
    }

    pub fn convert_to(&self, other: &Charset, target: Vec<u8>) -> Vec<u8> {
        let utf8: Cow<str> = self.get_encoding().decode(&target).0;
        other.get_encoding().encode(utf8.as_ref()).0.to_vec()
    }
}

impl FromStr for Charset {
    type Err = String;

    fn from_str(s: &str) -> Result<Charset, String> {
        let snake_case_string: String = s.to_lowercase();
        match &snake_case_string as &str {
            "big5" => Ok(Big5),
            "euc-jp" => Ok(EucJp),
            "euc-kr" => Ok(EucKr),
            "gbk" => Ok(Gbk),
            "ibm866" => Ok(Ibm866),
            "iso-2022-jp" => Ok(Iso2022Jp),
            "iso-8859-10" => Ok(Iso885910),
            "iso-8859-13" => Ok(Iso885913),
            "iso-8859-14" => Ok(Iso885914),
            "iso-8859-15" => Ok(Iso885915),
            "iso-8859-16" => Ok(Iso885916),
            "iso-8859-2" => Ok(Iso88592),
            "iso-8859-3" => Ok(Iso88593),
            "iso-8859-4" => Ok(Iso88594),
            "iso-8859-5" => Ok(Iso88595),
            "iso-8859-6" => Ok(Iso88596),
            "iso-8859-7" => Ok(Iso88597),
            "iso-8859-8" => Ok(Iso88598),
            "iso-8859-8-i" => Ok(Iso88598I),
            "koi8-r" => Ok(Koi8R),
            "koi8-u" => Ok(Koi8U),
            "shift_jis" => Ok(ShiftJis),
            "utf-16be" => Ok(Utf16be),
            "utf-16le" => Ok(Utf16le),
            "utf-8" => Ok(Utf8),
            "gb18030" => Ok(Gb18030),
            "macintosh" => Ok(Macintosh),
            "replacement" => Ok(Replacement),
            "windows-1250" => Ok(Windows1250),
            "windows-1251" => Ok(Windows1251),
            "windows-1252" => Ok(Windows1252),
            "windows-1253" => Ok(Windows1253),
            "windows-1254" => Ok(Windows1254),
            "windows-1255" => Ok(Windows1255),
            "windows-1256" => Ok(Windows1256),
            "windows-1257" => Ok(Windows1257),
            "windows-1258" => Ok(Windows1258),
            "windows-874" => Ok(Windows874),
            "x-mac-cyrillic" => Ok(XMacCyrillic),
            _ => Err(format!("{} is not supported!", s)),
        }
    }
}

impl ToString for Charset {
    fn to_string(&self) -> String {
        match self {
            Big5 => "big5".to_owned(),
            EucJp => "euc-jp".to_owned(),
            EucKr => "euc-kr".to_owned(),
            Gbk => "gbk".to_owned(),
            Ibm866 => "ibm866".to_owned(),
            Iso2022Jp => "iso-2022-jp".to_owned(),
            Iso885910 => "iso-8859-10".to_owned(),
            Iso885913 => "iso-8859-13".to_owned(),
            Iso885914 => "iso-8859-14".to_owned(),
            Iso885915 => "iso-8859-15".to_owned(),
            Iso885916 => "iso-8859-16".to_owned(),
            Iso88592 => "iso-8859-2".to_owned(),
            Iso88593 => "iso-8859-3".to_owned(),
            Iso88594 => "iso-8859-4".to_owned(),
            Iso88595 => "iso-8859-5".to_owned(),
            Iso88596 => "iso-8859-6".to_owned(),
            Iso88597 => "iso-8859-7".to_owned(),
            Iso88598 => "iso-8859-8".to_owned(),
            Iso88598I => "iso-8859-8-i".to_owned(),
            Koi8R => "koi8-r".to_owned(),
            Koi8U => "koi8-u".to_owned(),
            ShiftJis => "shift_jis".to_owned(),
            Utf16be => "utf-16be".to_owned(),
            Utf16le => "utf-16le".to_owned(),
            Utf8 => "utf-8".to_owned(),
            Gb18030 => "gb18030".to_owned(),
            Macintosh => "macintosh".to_owned(),
            Replacement => "replacement".to_owned(),
            Windows1250 => "windows-1250".to_owned(),
            Windows1251 => "windows-1251".to_owned(),
            Windows1252 => "windows-1252".to_owned(),
            Windows1253 => "windows-1253".to_owned(),
            Windows1254 => "windows-1254".to_owned(),
            Windows1255 => "windows-1255".to_owned(),
            Windows1256 => "windows-1256".to_owned(),
            Windows1257 => "windows-1257".to_owned(),
            Windows1258 => "windows-1258".to_owned(),
            Windows874 => "windows-874".to_owned(),
            XMacCyrillic => "x-mac-cyrillic".to_owned(),
        }
    }
}

struct CharsetVisitor;
impl<'de> Deserialize<'de> for Charset {
    fn deserialize<D>(deserializer: D) -> Result<Charset, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CharsetVisitor)
    }
}

impl<'de> Visitor<'de> for CharsetVisitor {
    type Value = Charset;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            r#"your charset is not supported! please check the document."#
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Charset, E>
    where
        E: Error,
    {
        match Charset::from_str(v) {
            Ok(charset) => Ok(charset),
            Err(_) => Err(E::invalid_type(Unexpected::Str(v), &self)),
        }
    }
}
