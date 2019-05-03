use rs_es::error::EsError;
use rs_es::operations::mapping::Settings;
use serde::Serialize;
use serde_json::Value;
use std::io::Error;

pub trait EsDocument<'a>: Serialize {
    fn elastic_search_template() -> &'a Value;

    fn elastic_search_typename() -> &'a str;

    fn elastic_search_settings() -> Option<&'a Settings>;

    fn elastic_search_index_name(&self) -> &str;
}

pub trait BulkInsertToEs {
    fn bulk_insert<'a, T>(&self, document: &[T]) -> Result<(), EsError>
    where
        T: EsDocument<'a>;

    fn create_index_template<'a, T>(&self, name: &str) -> Result<(), Error>
    where
        T: EsDocument<'a>;
}

// TODO: Errorを定義
