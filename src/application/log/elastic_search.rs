use rs_es::error::EsError;
use rs_es::operations::mapping::{Mapping, Settings};
use serde::Serialize;

pub trait EsDocument<'a>: Serialize {
    fn elastic_search_mapping() -> Mapping<'a>;

    fn elastic_search_settings() -> &'a Settings;

    fn elastic_search_index_name(&self) -> &str;

    fn elastic_search_num_of_shard() -> u32;
}

pub trait InsertToEs {
    fn insert<'a, T>(&self, document: T) -> Result<(), EsError>
    where
        T: EsDocument<'a>;
}
