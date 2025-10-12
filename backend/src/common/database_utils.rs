use crate::entity;

pub fn get_database_url(payload: &entity::connection::Model) -> String {
    format!(
        "postgres://{}:{}@{}:{}/{}",
        payload.username,
        payload.password,
        payload.host,
        payload.port,
        payload.database
    )
}
