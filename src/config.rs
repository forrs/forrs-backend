use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub database: forrs_stm::db::Config,
}

impl std::str::FromStr for Config {
    type Err = toml::de::Error;
    fn from_str(s: &str) -> Result<Config, Self::Err> {
        toml::from_str(s)
    }
}
