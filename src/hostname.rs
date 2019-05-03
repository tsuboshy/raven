use rand::random;

pub fn get_hostname() -> &'static str {
    lazy_static! {
        static ref HOSTNAME: String = _hostname::get_hostname().unwrap_or_else(|| {
            let unknown_host = format!("unknown_host_{}", random::<u16>());
            warn!("failed to get hostname. fall back to {}", unknown_host);
            unknown_host
        });
    }

    &*HOSTNAME
}
