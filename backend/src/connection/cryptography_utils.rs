use crate::common::cryptography_utils;
use crate::entity::connection;

fn encrypt_field(key: &str, value: &str) -> Result<String, aes_gcm::Error> {
    cryptography_utils::encrypt(key.to_string(), value.to_string())
}

fn decrypt_field(key: &str, value: &str) -> anyhow::Result<String> {
    cryptography_utils::decrypt(key.to_string(), value.to_string())
}

pub fn encrypt_connection(entity: &mut connection::Model, key: &str) -> Result<(), aes_gcm::Error> {
    entity.database = encrypt_field(key, &entity.database)?;
    entity.host = entity
        .host
        .as_ref()
        .map(|s| encrypt_field(key, s))
        .transpose()?;
    entity.password = entity
        .password
        .as_ref()
        .map(|s| encrypt_field(key, s))
        .transpose()?;
    entity.port = entity
        .port
        .as_ref()
        .map(|s| encrypt_field(key, s))
        .transpose()?;
    entity.schema = entity
        .schema
        .as_ref()
        .map(|s| encrypt_field(key, s))
        .transpose()?;
    entity.username = entity
        .username
        .as_ref()
        .map(|s| encrypt_field(key, s))
        .transpose()?;
    Ok(())
}

pub fn decrypt_connection(entity: &mut connection::Model, key: &str) -> anyhow::Result<()> {
    entity.database = decrypt_field(key, &entity.database)?;
    entity.host = entity
        .host
        .as_ref()
        .map(|encrypted_host| decrypt_field(key, encrypted_host))
        .transpose()?;
    entity.password = entity
        .password
        .as_ref()
        .map(|encrypted_password| decrypt_field(key, encrypted_password))
        .transpose()?;
    entity.port = entity
        .port
        .as_ref()
        .map(|encrypted_port| decrypt_field(key, encrypted_port))
        .transpose()?;
    entity.schema = entity
        .schema
        .as_ref()
        .map(|encrypted_schema| decrypt_field(key, encrypted_schema))
        .transpose()?;
    entity.username = entity
        .username
        .as_ref()
        .map(|encrypted_username| decrypt_field(key, encrypted_username))
        .transpose()?;
    Ok(())
}
