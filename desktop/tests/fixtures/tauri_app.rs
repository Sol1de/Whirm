#![allow(dead_code)]

use app_lib::AppState;
use sea_orm::DatabaseConnection;
use std::sync::Mutex;

pub struct TestApp {
    pub state: AppState,
}

impl TestApp {
    // Return a plain reference for tests; command logic should be extracted
    // into non-Tauri helpers that accept `&AppState`, with `#[tauri::command]`
    // functions acting as thin wrappers in application code.
    pub fn state_ref(&self) -> &AppState {
        &self.state
    }
}

pub fn make_test_app(db: DatabaseConnection) -> TestApp {
    TestApp {
        state: AppState {
            saved_proxy: Mutex::new(None),
            db,
            active_session_id: Mutex::new(None),
            active_proxy_id: Mutex::new(None),
        },
    }
}
