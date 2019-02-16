pub use std::collections::HashMap;

#[macro_export]
macro_rules! hashmap {
    ( $( $key:expr => $val:expr ),* ) => {
        {
            let mut map = $crate::macros::HashMap::new();
            $( map.insert($key, $val); )*
            map
        }
    };

    ( $( $key:expr => $val:expr ),+ , ) => {
        hashmap![ $( $key => $val ),* ];
    };
}

#[macro_export]
macro_rules! copy_ref_to_other_map {
    ( $src:ident, $dest:ident ) => {
        for (key, val) in $src.iter() {
            $crate::macros::HashMap::insert(&mut $dest, key, val);
        }
    };
}
