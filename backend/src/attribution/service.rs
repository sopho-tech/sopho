use crate::attribution::dto::{AttributedUserDto, AttributionDto};
use crate::attribution::error::AttributionError;
use crate::authorization::constants::ActorKind;
use crate::common::AppState;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct AttributionNames {
    full_names_by_user_id: HashMap<Uuid, String>,
}

impl AttributionNames {
    pub fn describe(
        &self,
        actor: &str,
        user_id: Option<Uuid>,
    ) -> Result<AttributionDto, AttributionError> {
        let actor_kind = ActorKind::from_str(actor).map_err(AttributionError::UnknownActor)?;
        let user = match (actor_kind, user_id) {
            (ActorKind::System, None) => None,
            (ActorKind::System, Some(user_id)) => {
                return Err(AttributionError::SystemNamesUser(user_id));
            }
            (_, None) => return Err(AttributionError::MissingUser(actor_kind)),
            (_, Some(user_id)) => Some(AttributedUserDto {
                user_id,
                full_name: self
                    .full_names_by_user_id
                    .get(&user_id)
                    .cloned()
                    .ok_or(AttributionError::UnknownUser(user_id))?,
            }),
        };
        Ok(AttributionDto {
            actor: actor_kind,
            user,
        })
    }
}

pub async fn load_names(
    app_state: &AppState,
    user_ids: impl IntoIterator<Item = Option<Uuid>>,
) -> Result<AttributionNames, AttributionError> {
    let distinct_user_ids: HashSet<Uuid> = user_ids.into_iter().flatten().collect();
    if distinct_user_ids.is_empty() {
        return Ok(AttributionNames {
            full_names_by_user_id: HashMap::new(),
        });
    }
    let users = crate::authentication::service::find_users_by_ids(
        app_state,
        distinct_user_ids.into_iter().collect(),
    )
    .await?;
    Ok(AttributionNames {
        full_names_by_user_id: users
            .into_iter()
            .map(|user| (user.id, user.full_name))
            .collect(),
    })
}
