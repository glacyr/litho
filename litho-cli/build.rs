use vergen::vergen;
use vergen::{Config, ShaKind, TimestampKind};

pub fn main() {
    let mut config = Config::default();
    *config.git_mut().sha_kind_mut() = ShaKind::Short;
    *config.git_mut().commit_timestamp_kind_mut() = TimestampKind::DateOnly;

    vergen(config).unwrap();
}
