// Session management implementation for WhatsApp Engine
// This module handles session persistence, phone number extraction, and session IDs

use crate::{WhatsAppError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

/// Session data structure for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub session_id: String,
    pub phone_number: Option<String>,
    pub authenticated_at: chrono::DateTime<chrono::Utc>,
    pub browser_cookies: Option<String>, // JSON serialized cookies
    pub local_storage: Option<String>,   // JSON serialized local storage
    pub session_storage: Option<String>, // JSON serialized session storage
}

/// Session manager handles persistent authentication state
#[derive(Debug)]
pub struct SessionManager {
    session_dir: PathBuf,
    current_session: Option<SessionData>,
}

impl SessionManager {
    /// Create a new session manager with the specified session directory
    pub fn new(session_dir: PathBuf) -> Self {
        Self {
            session_dir,
            current_session: None,
        }
    }

    /// Generate a new unique session ID
    pub fn generate_session_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Save session data to disk
    pub async fn save_session(&mut self, session_data: SessionData) -> Result<()> {
        // Ensure session directory exists
        fs::create_dir_all(&self.session_dir).await
            .map_err(|e| WhatsAppError::FileError {
                details: format!("Failed to create session directory: {}", e)
            })?;

        let session_file = self.session_dir.join(format!("{}.json", session_data.session_id));
        let json_data = serde_json::to_string_pretty(&session_data)
            .map_err(|e| WhatsAppError::SerializationError {
                details: format!("Failed to serialize session data: {}", e)
            })?;

        fs::write(&session_file, json_data).await
            .map_err(|e| WhatsAppError::FileError {
                details: format!("Failed to save session file: {}", e)
            })?;

        self.current_session = Some(session_data);
        tracing::info!("Session saved successfully");
        Ok(())
    }

    /// Load the most recent session from disk
    pub async fn load_latest_session(&mut self) -> Result<Option<SessionData>> {
        if !self.session_dir.exists() {
            return Ok(None);
        }

        let mut entries = fs::read_dir(&self.session_dir).await
            .map_err(|e| WhatsAppError::FileError {
                details: format!("Failed to read session directory: {}", e)
            })?;

        let mut latest_session: Option<SessionData> = None;
        let mut latest_time = chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).unwrap();

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| WhatsAppError::FileError {
                details: format!("Failed to read directory entry: {}", e)
            })? {
            
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_session_from_file(&path).await {
                    Ok(session_data) => {
                        if session_data.authenticated_at > latest_time {
                            latest_time = session_data.authenticated_at;
                            latest_session = Some(session_data);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load session from {:?}: {}", path, e);
                    }
                }
            }
        }

        self.current_session = latest_session.clone();
        Ok(latest_session)
    }

    /// Load session data from a specific file
    async fn load_session_from_file(&self, path: &PathBuf) -> Result<SessionData> {
        let content = fs::read_to_string(path).await
            .map_err(|e| WhatsAppError::FileError {
                details: format!("Failed to read session file: {}", e)
            })?;

        let session_data: SessionData = serde_json::from_str(&content)
            .map_err(|e| WhatsAppError::SerializationError {
                details: format!("Failed to parse session data: {}", e)
            })?;

        Ok(session_data)
    }

    /// Get the current session data
    pub fn get_current_session(&self) -> Option<&SessionData> {
        self.current_session.as_ref()
    }

    /// Create a new session with generated ID
    pub fn create_new_session(&mut self, phone_number: Option<String>) -> SessionData {
        let session_data = SessionData {
            session_id: Self::generate_session_id(),
            phone_number,
            authenticated_at: chrono::Utc::now(),
            browser_cookies: None,
            local_storage: None,
            session_storage: None,
        };

        self.current_session = Some(session_data.clone());
        session_data
    }

    /// Update session with browser data
    pub async fn update_session_browser_data(
        &mut self,
        cookies: Option<String>,
        local_storage: Option<String>,
        session_storage: Option<String>,
    ) -> Result<()> {
        if let Some(ref mut session) = self.current_session {
            session.browser_cookies = cookies;
            session.local_storage = local_storage;
            session.session_storage = session_storage;

            // Clone session before calling save to avoid borrow conflicts
            let session_to_save = session.clone();
            self.save_session(session_to_save).await?;
        }
        Ok(())
    }

    /// Update session with phone number
    pub async fn update_session_phone_number(&mut self, phone_number: String) -> Result<()> {
        if let Some(ref mut session) = self.current_session {
            session.phone_number = Some(phone_number);
            
            // Clone session before calling save to avoid borrow conflicts
            let session_to_save = session.clone();
            self.save_session(session_to_save).await?;
        }
        Ok(())
    }

    /// Clean up old session files (keep only the latest N sessions)
    pub async fn cleanup_old_sessions(&self, keep_count: usize) -> Result<()> {
        if !self.session_dir.exists() {
            return Ok(());
        }

        let mut entries = fs::read_dir(&self.session_dir).await
            .map_err(|e| WhatsAppError::FileError {
                details: format!("Failed to read session directory: {}", e)
            })?;

        let mut session_files = Vec::new();

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| WhatsAppError::FileError {
                details: format!("Failed to read directory entry: {}", e)
            })? {
            
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(session_data) = self.load_session_from_file(&path).await {
                    session_files.push((path, session_data.authenticated_at));
                }
            }
        }

        // Sort by authentication time (newest first)
        session_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Delete old session files
        for (path, _) in session_files.into_iter().skip(keep_count) {
            if let Err(e) = fs::remove_file(&path).await {
                tracing::warn!("Failed to remove old session file {:?}: {}", path, e);
            } else {
                tracing::info!("Removed old session file: {:?}", path);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_session_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let session_manager = SessionManager::new(temp_dir.path().to_path_buf());
        
        assert!(session_manager.get_current_session().is_none());
    }

    #[tokio::test]
    async fn test_session_id_generation() {
        let session_id1 = SessionManager::generate_session_id();
        let session_id2 = SessionManager::generate_session_id();
        
        assert_ne!(session_id1, session_id2);
        assert_eq!(session_id1.len(), 36); // UUID format
    }

    #[tokio::test]
    async fn test_create_new_session() {
        let temp_dir = TempDir::new().unwrap();
        let mut session_manager = SessionManager::new(temp_dir.path().to_path_buf());
        
        let phone = Some("+1234567890".to_string());
        let session = session_manager.create_new_session(phone.clone());
        
        assert_eq!(session.phone_number, phone);
        assert!(!session.session_id.is_empty());
        assert!(session_manager.get_current_session().is_some());
    }

    #[tokio::test]
    async fn test_save_and_load_session() {
        let temp_dir = TempDir::new().unwrap();
        let mut session_manager = SessionManager::new(temp_dir.path().to_path_buf());
        
        // Create and save a session
        let phone = Some("+1234567890".to_string());
        let session = session_manager.create_new_session(phone.clone());
        session_manager.save_session(session.clone()).await.unwrap();
        
        // Create a new session manager and load the session
        let mut new_session_manager = SessionManager::new(temp_dir.path().to_path_buf());
        let loaded_session = new_session_manager.load_latest_session().await.unwrap();
        
        assert!(loaded_session.is_some());
        let loaded = loaded_session.unwrap();
        assert_eq!(loaded.session_id, session.session_id);
        assert_eq!(loaded.phone_number, session.phone_number);
    }

    #[tokio::test]
    async fn test_update_session_phone_number() {
        let temp_dir = TempDir::new().unwrap();
        let mut session_manager = SessionManager::new(temp_dir.path().to_path_buf());
        
        // Create session without phone number
        let session = session_manager.create_new_session(None);
        session_manager.save_session(session).await.unwrap();
        
        // Update with phone number
        let phone = "+1234567890".to_string();
        session_manager.update_session_phone_number(phone.clone()).await.unwrap();
        
        let current_session = session_manager.get_current_session().unwrap();
        assert_eq!(current_session.phone_number, Some(phone));
    }

    #[tokio::test]
    async fn test_cleanup_old_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let mut session_manager = SessionManager::new(temp_dir.path().to_path_buf());
        
        // Create multiple sessions
        for i in 0..5 {
            let phone = Some(format!("+123456789{}", i));
            let session = session_manager.create_new_session(phone);
            session_manager.save_session(session).await.unwrap();
            
            // Small delay to ensure different timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        // Clean up, keeping only 2 sessions
        session_manager.cleanup_old_sessions(2).await.unwrap();
        
        // Verify only 2 files remain
        let mut entries = fs::read_dir(temp_dir.path()).await.unwrap();
        let mut file_count = 0;
        while let Some(_) = entries.next_entry().await.unwrap() {
            file_count += 1;
        }
        assert_eq!(file_count, 2);
    }
}
