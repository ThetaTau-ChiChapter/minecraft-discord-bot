use figment::{
    Figment,
    providers::{Env, Format, Yaml},
};
use serde::Deserialize;

/// Configuration for the discord bot
#[derive(Debug, Deserialize, Clone)]
pub struct Bot {
    /// Discord application token
    pub discord_token: String,
}

/// Configuration for the database connection
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Database {
    /// Database URL
    pub url: String,
    /// Connection pool size
    #[serde(default)]
    pub pool_size: Option<u32>,
    /// Logging level
    #[serde(default)]
    pub schema: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Minecraft {
    /// Minecraft server rcon url
    pub rcon_address: String,
    /// Rcon server password
    pub rcon_password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub bot: Bot,
    pub database: Database,
    pub minecraft: Minecraft
}

impl Config {
    /// Parse the configuration from YAML files and environment variables
    pub fn parse(path: &str) -> Result<Self, crate::Error> {
        // Read local files first
        let mut obj = Figment::new();

        // get the names of all config files under the config dir
        for f in list_directory_files(path, "discord-", "yaml").unwrap_or_default() {
            obj = obj.merge(Yaml::file(f));
        }

        Ok(obj
            .merge(Env::prefixed("DISCORD_BOT__").split("__"))
            .extract()?)
    }
}

fn list_directory_files(
    dir: &str,
    prefix: &str,
    extension: &str,
) -> Result<Vec<String>, std::io::Error> {
    fn collect_files_recursively(
        dir: &std::path::Path,
        prefix: &str,
        extension: &str,
        files: &mut Vec<String>,
    ) -> Result<(), std::io::Error> {
        for entry in std::fs::read_dir(dir)? {
            let Ok(entry) = entry else {
                continue;
            };

            let path = entry.path();
            if path.is_dir() {
                collect_files_recursively(&path, prefix, extension, files)?;
                continue;
            }
            println!("Found file: {}", path.display());

            let matches_prefix = path
                .file_name()
                .map(|name| name.to_string_lossy().starts_with(prefix))
                .unwrap_or(false);
            let matches_extension = path
                .extension()
                .map(|ext| ext == extension)
                .unwrap_or(false);

            if matches_prefix && matches_extension {
                files.push(path.to_string_lossy().into_owned());
            }
        }

        Ok(())
    }

    let mut files = Vec::new();
    collect_files_recursively(std::path::Path::new(dir), prefix, extension, &mut files)?;
    files.sort();
    println!("Found config files: {:?}", files);
    Ok(files)
}
