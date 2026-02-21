use crate::common::cryptography_utils;
use crate::connection::dto::ConnectionDto;
use crate::entity::connection;

fn try_decrypt(key: &str, value: &str) -> String {
    cryptography_utils::decrypt(key.to_string(), value.to_string()).unwrap_or_else(|_| value.to_string())
}

fn encrypt_field(key: &str, value: &str) -> Result<String, aes_gcm::Error> {
    cryptography_utils::encrypt(key.to_string(), value.to_string())
}

pub fn encrypt_connection(entity: &mut connection::Model, key: &str) -> Result<(), aes_gcm::Error> {
    entity.database = encrypt_field(key, &entity.database)?;
    entity.host = entity.host.as_ref().map(|s| encrypt_field(key, s)).transpose()?;
    entity.password = entity.password.as_ref().map(|s| encrypt_field(key, s)).transpose()?;
    entity.port = entity.port.as_ref().map(|s| encrypt_field(key, s)).transpose()?;
    entity.schema = entity.schema.as_ref().map(|s| encrypt_field(key, s)).transpose()?;
    entity.username = entity.username.as_ref().map(|s| encrypt_field(key, s)).transpose()?;
    Ok(())
}

pub fn encrypt_connection_dto(payload: &mut ConnectionDto, key: &str) -> Result<(), aes_gcm::Error> {
    payload.database = encrypt_field(key, &payload.database)?;
    payload.host = encrypt_field(key, &payload.host)?;
    payload.password = encrypt_field(key, &payload.password)?;
    payload.port = encrypt_field(key, &payload.port)?;
    if let Some(ref s) = payload.schema {
        payload.schema = Some(encrypt_field(key, s)?);
    }
    payload.username = encrypt_field(key, &payload.username)?;
    Ok(())
}

pub fn decrypt_connection(entity: &mut connection::Model, key: &str) {
    entity.database = try_decrypt(key, &entity.database);
    entity.host = entity.host.as_ref().map(|s| try_decrypt(key, s));
    entity.password = entity.password.as_ref().map(|s| try_decrypt(key, s));
    entity.port = entity.port.as_ref().map(|s| try_decrypt(key, s));
    entity.schema = entity.schema.as_ref().map(|s| try_decrypt(key, s));
    entity.username = entity.username.as_ref().map(|s| try_decrypt(key, s));
}
