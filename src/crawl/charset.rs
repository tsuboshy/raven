use encoding_rs::*;
use serde::de::{Deserialize, Deserializer, Error, Unexpected, Visitor};
use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

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
        match s {
            "big5" => Ok(Big5),
            "euc_jp" => Ok(EucJp),
            "euc_kr" => Ok(EucKr),
            "gbk" => Ok(Gbk),
            "ibm866" => Ok(Ibm866),
            "iso_2022_jp" => Ok(Iso2022Jp),
            "iso_8859_10" => Ok(Iso885910),
            "iso_8859_13" => Ok(Iso885913),
            "iso_8859_14" => Ok(Iso885914),
            "iso_8859_15" => Ok(Iso885915),
            "iso_8859_16" => Ok(Iso885916),
            "iso_8859_2" => Ok(Iso88592),
            "iso_8859_3" => Ok(Iso88593),
            "iso_8859_4" => Ok(Iso88594),
            "iso_8859_5" => Ok(Iso88595),
            "iso_8859_6" => Ok(Iso88596),
            "iso_8859_7" => Ok(Iso88597),
            "iso_8859_8" => Ok(Iso88598),
            "iso_8859_8_i" => Ok(Iso88598I),
            "koi8_r" => Ok(Koi8R),
            "koi8_u" => Ok(Koi8U),
            "shift_jis" => Ok(ShiftJis),
            "utf_16be" => Ok(Utf16be),
            "utf_16le" => Ok(Utf16le),
            "utf_8" => Ok(Utf8),
            "gb18030" => Ok(Gb18030),
            "macintosh" => Ok(Macintosh),
            "replacement" => Ok(Replacement),
            "windows_1250" => Ok(Windows1250),
            "windows_1251" => Ok(Windows1251),
            "windows_1252" => Ok(Windows1252),
            "windows_1253" => Ok(Windows1253),
            "windows_1254" => Ok(Windows1254),
            "windows_1255" => Ok(Windows1255),
            "windows_1256" => Ok(Windows1256),
            "windows_1257" => Ok(Windows1257),
            "windows_1258" => Ok(Windows1258),
            "windows_874" => Ok(Windows874),
            "x_mac_cyrillic" => Ok(XMacCyrillic),
            _ => Err(format!("{} is not supported!", s)),
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
