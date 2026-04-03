use std::path::PathBuf;

use anyhow::anyhow;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use tokio::fs::{self, try_exists, write};

use crate::{auth::Token, Result};

pub enum UserType {
    Human,
    Agent,
}

#[derive(Clone)]
pub struct SpaceAccess {
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Clone)]
pub struct User {
    // config values
    pub name: String,
    pub path: PathBuf,
    pub email: Option<String>,
    pub activation_token: Option<String>,
    pub space_access: Vec<SpaceAccess>,
    // todo: profile
    // pub user_type: UserType,
}

pub fn try_hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    return argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| anyhow!("could not hash password: {}", err))
        .map(|p| p.to_string());
}

impl User {
    // If the user's directory exists and has a `password` file then the user is activated
    pub async fn is_activated(&self) -> Result<bool> {
        let user_dir_exists = match try_exists(&self.path).await {
            Ok(e) => e,
            Err(err) => {
                return Err(anyhow!("could not lookup user dir: {}", err));
            }
        };

        if (!user_dir_exists) {
            return Ok(false);
        }

        let password_exists = match try_exists(&self.path.join("password")).await {
            Ok(e) => e,
            Err(err) => {
                return Err(anyhow!(
                    "could not lookup activation_attempts file in user dir: {}",
                    err
                ));
            }
        };

        if (!password_exists) {
            return Ok(false);
        }

        return Ok(true);
    }

    pub async fn is_password(&self, password: &str) -> Result<bool> {
        let hash = fs::read_to_string(self.path.join("password"))
            .await
            .map_err(|err| anyhow!("could not read password file: {}", err))?;

        let parsed_hash = argon2::PasswordHash::new(&hash)
            .map_err(|err| anyhow!("could not parse password hash: {}", err))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    // todo: this should accept more args in the future for things like level of access and lifetime
    pub async fn create_session(&self) -> Result<Token> {
        let token = Token::new()?;

        // write the session file
        write(
            self.path.join(format!("session_{}", token.value.clone())),
            format!(
                "{}\n{}\n{}",
                token.value.clone(),
                token.created_at,
                token.expires_at
            ),
        )
        .await
        .map_err(|err| anyhow!(err).context("could not write session file"))?;

        Ok(token)
    }
}
