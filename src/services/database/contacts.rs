//! Contact Operations
//!
//! CRUD operations for contacts cache.

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::params;
use tracing::info;

use crate::models::message::Contact;

use super::service::{DatabaseService, CONTACT_BATCH_SIZE};

impl DatabaseService {
    /// Save or update a contact
    pub fn upsert_contact(&self, phone: &str, name: Option<&str>, is_business: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO contacts (phone, name, is_business, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![phone, name, is_business as i32, now],
        )?;
        Ok(())
    }

    /// Get contact by phone
    pub fn get_contact(&self, phone: &str) -> Result<Option<Contact>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT phone, name, is_business, last_seen, updated_at
             FROM contacts WHERE phone = ?1",
            params![phone],
            |row| {
                let updated_str: String = row.get(4)?;
                let last_seen_str: Option<String> = row.get(3)?;
                Ok(Contact {
                    phone: row.get(0)?,
                    name: row.get(1)?,
                    is_business: row.get::<_, i32>(2)? != 0,
                    last_seen: last_seen_str
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                    updated_at: DateTime::parse_from_rfc3339(&updated_str)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            },
        );

        match result {
            Ok(contact) => Ok(Some(contact)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Batch insert/update contacts (more efficient for syncing)
    pub fn put_all_contacts(&self, contacts: &[Contact]) -> Result<usize> {
        if contacts.is_empty() {
            return Ok(0);
        }

        let conn = self.conn.lock().unwrap();
        let mut count = 0;

        for chunk in contacts.chunks(CONTACT_BATCH_SIZE) {
            for contact in chunk {
                let now = Utc::now().to_rfc3339();
                let last_seen = contact.last_seen.map(|t| t.to_rfc3339());

                conn.execute(
                    "INSERT INTO contacts (phone, name, is_business, last_seen, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(phone) DO UPDATE SET 
                        name = COALESCE(excluded.name, contacts.name),
                        is_business = excluded.is_business,
                        last_seen = COALESCE(excluded.last_seen, contacts.last_seen),
                        updated_at = excluded.updated_at",
                    params![
                        contact.phone,
                        contact.name,
                        contact.is_business as i32,
                        last_seen,
                        now
                    ],
                )?;
                count += 1;
            }
        }

        info!("Batch inserted/updated {} contacts", count);
        Ok(count)
    }

    /// Get all contacts
    pub fn get_all_contacts(&self) -> Result<Vec<Contact>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT phone, name, is_business, last_seen, updated_at FROM contacts ORDER BY name",
        )?;

        let mut contacts = Vec::new();
        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            let last_seen_str: Option<String> = row.get(3)?;
            let updated_str: String = row.get(4)?;

            contacts.push(Contact {
                phone: row.get(0)?,
                name: row.get(1)?,
                is_business: row.get::<_, i32>(2)? != 0,
                last_seen: last_seen_str
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                updated_at: DateTime::parse_from_rfc3339(&updated_str)?.with_timezone(&Utc),
            });
        }

        Ok(contacts)
    }
}
