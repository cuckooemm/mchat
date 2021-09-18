#[derive(Clone, Default, Deserialize)]
/// The global configuration
pub struct Config {
    /// The server configuration
    pub server: ServerConfig,

    /// The logger configuration
    pub log: LogConfig,
}

#[derive(Clone, Default, Deserialize)]
pub struct ServerConfig {
    /// The server IP address
    pub addr: String,

}

#[derive(Clone, Default, Deserialize)]
pub struct LogConfig {
    /// The logging level
    pub level: String,
    /// The log file path
    pub path: String,
}