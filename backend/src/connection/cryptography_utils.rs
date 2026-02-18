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
    entity.host = encrypt_field(key, &entity.host)?;
    entity.password = encrypt_field(key, &entity.password)?;
    entity.port = encrypt_field(key, &entity.port)?;
    if let Some(ref s) = entity.schema {
        entity.schema = Some(encrypt_field(key, s)?);
    }
    entity.username = encrypt_field(key, &entity.username)?;
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
    entity.host = try_decrypt(key, &entity.host);
    entity.password = try_decrypt(key, &entity.password);
    entity.port = try_decrypt(key, &entity.port);
    if let Some(ref s) = entity.schema {
        entity.schema = Some(try_decrypt(key, s));
    }
    entity.username = try_decrypt(key, &entity.username);
}
