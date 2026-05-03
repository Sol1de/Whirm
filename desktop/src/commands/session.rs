use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, QueryOrder, QuerySelect};
use tauri::State;

use crate::db::entities::{connection_session, proxy};
use crate::AppState;

#[tauri::command]
pub async fn open_session(
    proxy_id: String,
    ip_address: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let now = Utc::now();

    if let Some(proxy_model) = proxy::Entity::find_by_id(&proxy_id)
        .one(&state.db)
        .await
        .map_err(|e| e.to_string())?
    {
        let active = proxy::ActiveModel {
            id: Set(proxy_model.id),
            last_used_at: Set(Some(now)),
            updated_at: Set(now),
            ..Default::default()
        };

        active.update(&state.db).await.map_err(|e| e.to_string())?;
    }

    let session = connection_session::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        proxy_id: Set(Some(proxy_id)),
        connected_at: Set(now),
        disconnected_at: Set(None),
        ip_address: Set(Some(ip_address)),
    };

    let result = session
        .insert(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let id = result.id;

    *state.active_session_id.lock().unwrap_or_else(|e| e.into_inner()) = Some(id.clone());

    Ok(id)
}

#[tauri::command]
pub async fn close_session(session_id: String, state: State<'_, AppState>) -> Result<(), String> {
    if let Some(session) = connection_session::Entity::find_by_id(&session_id)
        .one(&state.db)
        .await
        .map_err(|e| e.to_string())?
    {
        let active = connection_session::ActiveModel {
            id: Set(session.id),
            disconnected_at: Set(Some(Utc::now())),
            ..Default::default()
        };

        active.update(&state.db).await.map_err(|e| e.to_string())?;
    }

    state
        .active_session_id
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take_if(|id| id == &session_id);

    Ok(())
}

#[tauri::command]
pub async fn get_sessions(
    limit: Option<u64>,
    state: State<'_, AppState>,
) -> Result<Vec<connection_session::Model>, String> {
    let mut query = connection_session::Entity::find()
        .order_by_desc(connection_session::Column::ConnectedAt);

    if let Some(n) = limit {
        query = query.limit(n);
    }

    Ok(query.all(&state.db).await.map_err(|e| e.to_string())?)
}
