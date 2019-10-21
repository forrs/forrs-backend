use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub database: DbConfig,
}

#[derive(Deserialize, Debug)]
pub struct DbConfig {
    host: String,
    port: u16,
    db_name: String,
    user: String,
    password: String,
}

impl std::str::FromStr for Config {
    type Err = toml::de::Error;
    fn from_str(s: &str) -> Result<Config, Self::Err> {
        toml::from_str(s)
    }
}

impl From<&DbConfig> for tokio_postgres::Config {
    fn from(s: &DbConfig) -> tokio_postgres::Config {
        let mut c = tokio_postgres::Config::new();
        c.host(&s.host)
            .port(s.port)
            .dbname(&s.db_name)
            .user(&s.user)
            .password(&s.password);
        c
    }
}
