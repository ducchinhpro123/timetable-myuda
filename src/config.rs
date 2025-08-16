use dotenv::dotenv;
use std::env;

// #[derive(Parser)]
#[derive(Debug)]
pub struct UserConfig {
    username: Option<String>,
    password: Option<String>,
}

impl UserConfig {
    /// Create a new UserConfig struct with default username and password that fetches from .env file.
    /// If .env file is empty, return empty string for both username and password with error message provided
    ///
    /// # Examples
    /// let user = UserConfig::from_env();
    /// ```
    pub fn from_env() -> Self {
        dotenv().ok();
        Self {
            username: env::var("UDA_USERNAME").ok().or_else(|| {
                eprintln!("Warning: UDA_USERNAME not set, using None");
                None
            }),
            password: env::var("UDA_PASSWORD").ok().or_else(|| {
                eprintln!("Warning: UDA_PASSWORD not set, using None");
                None
            }),
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.username.is_none() {
            return Err("Username not set");
        }
        if self.password.is_none() {
            return Err("Password not set");
        }
        Ok(())
    }

    pub fn get_username(&self) -> Option<&String> {
        self.username.as_ref()
    }
    pub fn get_password(&self) -> Option<&String> {
        self.password.as_ref()
    }
}
