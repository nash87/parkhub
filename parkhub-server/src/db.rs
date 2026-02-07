//! Database Module
//!
//! Provides persistent storage using redb (pure Rust embedded database).
//! Supports optional AES-256-GCM encryption for data at rest.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use redb::{Database as RedbDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use parkhub_common::models::{
    Booking, HomeofficeSettings, LotLayout, ParkingLot, ParkingSlot, User, Vehicle,
    WaitlistEntry, PushSubscription,
};

// ═══════════════════════════════════════════════════════════════════════════════
// TABLE DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════════

const USERS: TableDefinition<&str, &[u8]> = TableDefinition::new("users");
const USERS_BY_USERNAME: TableDefinition<&str, &str> = TableDefinition::new("users_by_username");
const USERS_BY_EMAIL: TableDefinition<&str, &str> = TableDefinition::new("users_by_email");
const SESSIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("sessions");
const BOOKINGS: TableDefinition<&str, &[u8]> = TableDefinition::new("bookings");
const PARKING_LOTS: TableDefinition<&str, &[u8]> = TableDefinition::new("parking_lots");
const PARKING_SLOTS: TableDefinition<&str, &[u8]> = TableDefinition::new("parking_slots");
const SLOTS_BY_LOT: TableDefinition<&str, &[u8]> = TableDefinition::new("slots_by_lot");
const VEHICLES: TableDefinition<&str, &[u8]> = TableDefinition::new("vehicles");
const SETTINGS: TableDefinition<&str, &str> = TableDefinition::new("settings");
const HOMEOFFICE: TableDefinition<&str, &[u8]> = TableDefinition::new("homeoffice");
const LOT_LAYOUTS: TableDefinition<&str, &[u8]> = TableDefinition::new("lot_layouts");
const WAITLIST: TableDefinition<&str, &[u8]> = TableDefinition::new("waitlist");
const PUSH_SUBSCRIPTIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("push_subscriptions");

// Settings keys
const SETTING_SETUP_COMPLETED: &str = "setup_completed";
const SETTING_DB_VERSION: &str = "db_version";
const SETTING_ENCRYPTION_SALT: &str = "encryption_salt";

const CURRENT_DB_VERSION: &str = "2";

// ═══════════════════════════════════════════════════════════════════════════════
// DATABASE CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub encryption_enabled: bool,
    pub passphrase: Option<String>,
    pub create_if_missing: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// SESSION
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub user_id: Uuid,
    pub username: String,
    pub role: String,
    pub refresh_token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Session {
    pub fn new(user_id: Uuid, duration_hours: i64) -> Self {
        let now = Utc::now();
        let refresh_token = format!("rt_{}", Uuid::new_v4());
        Self {
            user_id,
            username: String::new(),
            role: String::new(),
            refresh_token,
            created_at: now,
            expires_at: now + chrono::Duration::hours(duration_hours),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DATABASE STATISTICS
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct DatabaseStats {
    pub users: u64,
    pub bookings: u64,
    pub parking_lots: u64,
    pub slots: u64,
    pub sessions: u64,
    pub vehicles: u64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENCRYPTION HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    fn new(passphrase: &str, salt: &[u8]) -> Result<Self> {
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, 100_000, &mut key);
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;
        Ok(Self { cipher })
    }

    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 12 {
            return Err(anyhow!("Invalid encrypted data: too short"));
        }
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption failed: {}", e))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DATABASE IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

pub struct Database {
    inner: Arc<RwLock<RedbDatabase>>,
    encryptor: Option<Encryptor>,
    encryption_enabled: bool,
}

impl Database {
    pub fn open(config: DatabaseConfig) -> Result<Self> {
        let db_path = config.path.join("parkhub.redb");
        let db_exists = db_path.exists();
        if !db_exists && !config.create_if_missing {
            return Err(anyhow!(
                "Database not found at {:?} and create_if_missing is false",
                db_path
            ));
        }

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create data directory")?;
        }

        info!("Opening database at {:?}", db_path);
        let db = RedbDatabase::create(&db_path).context("Failed to create/open database")?;

        // Initialize tables
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(USERS)?;
            let _ = write_txn.open_table(USERS_BY_USERNAME)?;
            let _ = write_txn.open_table(USERS_BY_EMAIL)?;
            let _ = write_txn.open_table(SESSIONS)?;
            let _ = write_txn.open_table(BOOKINGS)?;
            let _ = write_txn.open_table(PARKING_LOTS)?;
            let _ = write_txn.open_table(PARKING_SLOTS)?;
            let _ = write_txn.open_table(SLOTS_BY_LOT)?;
            let _ = write_txn.open_table(VEHICLES)?;
            let _ = write_txn.open_table(SETTINGS)?;
            let _ = write_txn.open_table(HOMEOFFICE)?;
            let _ = write_txn.open_table(LOT_LAYOUTS)?;
            let _ = write_txn.open_table(WAITLIST)?;
            let _ = write_txn.open_table(PUSH_SUBSCRIPTIONS)?;
        }
        write_txn.commit()?;

        // Set up encryption if enabled
        let encryptor = if config.encryption_enabled {
            let passphrase = config
                .passphrase
                .as_ref()
                .ok_or_else(|| anyhow!("Encryption enabled but no passphrase provided"))?;

            let salt = {
                let read_txn = db.begin_read()?;
                let table = read_txn.open_table(SETTINGS)?;
                match table.get(SETTING_ENCRYPTION_SALT)? {
                    Some(value) => {
                        hex::decode(value.value()).context("Invalid salt in database")?
                    }
                    None => {
                        let mut salt = [0u8; 32];
                        rand::thread_rng().fill_bytes(&mut salt);
                        let write_txn = db.begin_write()?;
                        {
                            let mut table = write_txn.open_table(SETTINGS)?;
                            table.insert(SETTING_ENCRYPTION_SALT, hex::encode(&salt).as_str())?;
                        }
                        write_txn.commit()?;
                        salt.to_vec()
                    }
                }
            };

            Some(Encryptor::new(passphrase, &salt)?)
        } else {
            None
        };

        if !db_exists {
            let write_txn = db.begin_write()?;
            {
                let mut table = write_txn.open_table(SETTINGS)?;
                table.insert(SETTING_DB_VERSION, CURRENT_DB_VERSION)?;
            }
            write_txn.commit()?;
        }

        Ok(Self {
            inner: Arc::new(RwLock::new(db)),
            encryptor,
            encryption_enabled: config.encryption_enabled,
        })
    }

    pub fn is_encrypted(&self) -> bool {
        self.encryption_enabled
    }

    pub async fn is_fresh(&self) -> Result<bool> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(SETTINGS)?;
        match table.get(SETTING_SETUP_COMPLETED)? {
            Some(value) => Ok(value.value() != "true"),
            None => Ok(true),
        }
    }

    pub async fn mark_setup_completed(&self) -> Result<()> {
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(SETTINGS)?;
            table.insert(SETTING_SETUP_COMPLETED, "true")?;
        }
        write_txn.commit()?;
        info!("Database setup marked as completed");
        Ok(())
    }

    pub async fn stats(&self) -> Result<DatabaseStats> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        Ok(DatabaseStats {
            users: read_txn.open_table(USERS)?.len()?,
            bookings: read_txn.open_table(BOOKINGS)?.len()?,
            parking_lots: read_txn.open_table(PARKING_LOTS)?.len()?,
            slots: read_txn.open_table(PARKING_SLOTS)?.len()?,
            sessions: read_txn.open_table(SESSIONS)?.len()?,
            vehicles: read_txn.open_table(VEHICLES)?.len()?,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // INTERNAL HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    fn serialize<T: serde::Serialize>(&self, value: &T) -> Result<Vec<u8>> {
        let json = serde_json::to_vec(value).context("Failed to serialize")?;
        if let Some(ref enc) = self.encryptor {
            enc.encrypt(&json)
        } else {
            Ok(json)
        }
    }

    fn deserialize<T: serde::de::DeserializeOwned>(&self, data: &[u8]) -> Result<T> {
        let json = if let Some(ref enc) = self.encryptor {
            enc.decrypt(data)?
        } else {
            data.to_vec()
        };
        serde_json::from_slice(&json).context("Failed to deserialize")
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SESSION OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub async fn save_session(&self, token: &str, session: &Session) -> Result<()> {
        let data = self.serialize(session)?;
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(SESSIONS)?;
            table.insert(token, data.as_slice())?;
        }
        write_txn.commit()?;
        debug!("Saved session for user: {}", session.username);
        Ok(())
    }

    pub async fn get_session(&self, token: &str) -> Result<Option<Session>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(SESSIONS)?;
        match table.get(token)? {
            Some(value) => {
                let session: Session = self.deserialize(value.value())?;
                if session.expires_at < Utc::now() {
                    Ok(None)
                } else {
                    Ok(Some(session))
                }
            }
            None => Ok(None),
        }
    }

    pub async fn delete_session(&self, token: &str) -> Result<bool> {
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        let existed = {
            let mut table = write_txn.open_table(SESSIONS)?;
            let existed = table.remove(token)?.is_some();
            existed
        };
        write_txn.commit()?;
        Ok(existed)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // USER OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub async fn save_user(&self, user: &User) -> Result<()> {
        let id = user.id.to_string();
        let data = self.serialize(user)?;
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(USERS)?;
            table.insert(id.as_str(), data.as_slice())?;
            let mut idx = write_txn.open_table(USERS_BY_USERNAME)?;
            idx.insert(user.username.as_str(), id.as_str())?;
            let mut email_idx = write_txn.open_table(USERS_BY_EMAIL)?;
            email_idx.insert(user.email.as_str(), id.as_str())?;
        }
        write_txn.commit()?;
        debug!("Saved user: {} ({})", user.username, user.id);
        Ok(())
    }

    pub async fn get_user(&self, id: &str) -> Result<Option<User>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(USERS)?;
        match table.get(id)? {
            Some(value) => Ok(Some(self.deserialize(value.value())?)),
            None => Ok(None),
        }
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let idx = read_txn.open_table(USERS_BY_USERNAME)?;
        let user_id = match idx.get(username)? {
            Some(id) => id.value().to_string(),
            None => return Ok(None),
        };
        let table = read_txn.open_table(USERS)?;
        match table.get(user_id.as_str())? {
            Some(value) => Ok(Some(self.deserialize(value.value())?)),
            None => Ok(None),
        }
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let idx = read_txn.open_table(USERS_BY_EMAIL)?;
        let user_id = match idx.get(email)? {
            Some(id) => id.value().to_string(),
            None => return Ok(None),
        };
        let table = read_txn.open_table(USERS)?;
        match table.get(user_id.as_str())? {
            Some(value) => Ok(Some(self.deserialize(value.value())?)),
            None => Ok(None),
        }
    }

    pub async fn list_users(&self) -> Result<Vec<User>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(USERS)?;
        let mut users = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            users.push(self.deserialize(value.value())?);
        }
        Ok(users)
    }

    pub async fn delete_user(&self, id: &str) -> Result<bool> {
        let user = match self.get_user(id).await? {
            Some(u) => u,
            None => return Ok(false),
        };
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(USERS)?;
            table.remove(id)?;
            let mut idx = write_txn.open_table(USERS_BY_USERNAME)?;
            idx.remove(user.username.as_str())?;
            let mut email_idx = write_txn.open_table(USERS_BY_EMAIL)?;
            email_idx.remove(user.email.as_str())?;
        }
        write_txn.commit()?;
        debug!("Deleted user: {}", id);
        Ok(true)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PARKING LOT OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub async fn save_parking_lot(&self, lot: &ParkingLot) -> Result<()> {
        let id = lot.id.to_string();
        let data = self.serialize(lot)?;
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(PARKING_LOTS)?;
            table.insert(id.as_str(), data.as_slice())?;
        }
        write_txn.commit()?;
        debug!("Saved parking lot: {} ({})", lot.name, lot.id);
        Ok(())
    }

    pub async fn get_parking_lot(&self, id: &str) -> Result<Option<ParkingLot>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(PARKING_LOTS)?;
        match table.get(id)? {
            Some(value) => Ok(Some(self.deserialize(value.value())?)),
            None => Ok(None),
        }
    }

    pub async fn list_parking_lots(&self) -> Result<Vec<ParkingLot>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(PARKING_LOTS)?;
        let mut lots = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            lots.push(self.deserialize(value.value())?);
        }
        Ok(lots)
    }

    pub async fn delete_parking_lot(&self, id: &str) -> Result<bool> {
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        let existed = {
            let mut table = write_txn.open_table(PARKING_LOTS)?;
            let existed = table.remove(id)?.is_some();
            existed
        };
        write_txn.commit()?;
        if existed {
            debug!("Deleted parking lot: {}", id);
        }
        Ok(existed)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PARKING SLOT OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub async fn save_parking_slot(&self, slot: &ParkingSlot) -> Result<()> {
        let id = slot.id.to_string();
        let lot_id = slot.lot_id.to_string();
        let data = self.serialize(slot)?;
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(PARKING_SLOTS)?;
            table.insert(id.as_str(), data.as_slice())?;
            let mut idx = write_txn.open_table(SLOTS_BY_LOT)?;
            let key = format!("{}:{}", lot_id, id);
            idx.insert(key.as_str(), data.as_slice())?;
        }
        write_txn.commit()?;
        debug!("Saved parking slot: {} (lot: {})", slot.id, slot.lot_id);
        Ok(())
    }

    pub async fn get_parking_slot(&self, id: &str) -> Result<Option<ParkingSlot>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(PARKING_SLOTS)?;
        match table.get(id)? {
            Some(value) => Ok(Some(self.deserialize(value.value())?)),
            None => Ok(None),
        }
    }

    pub async fn list_slots_by_lot(&self, lot_id: &str) -> Result<Vec<ParkingSlot>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(SLOTS_BY_LOT)?;
        let prefix = format!("{}:", lot_id);
        let mut slots = Vec::new();
        for entry in table.iter()? {
            let (key, value) = entry?;
            if key.value().starts_with(&prefix) {
                slots.push(self.deserialize(value.value())?);
            }
        }
        Ok(slots)
    }

    pub async fn update_slot_status(
        &self,
        slot_id: &str,
        status: parkhub_common::models::SlotStatus,
    ) -> Result<bool> {
        let mut slot = match self.get_parking_slot(slot_id).await? {
            Some(s) => s,
            None => return Ok(false),
        };
        slot.status = status;
        self.save_parking_slot(&slot).await?;
        Ok(true)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BOOKING OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub async fn save_booking(&self, booking: &Booking) -> Result<()> {
        let id = booking.id.to_string();
        let data = self.serialize(booking)?;
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(BOOKINGS)?;
            table.insert(id.as_str(), data.as_slice())?;
        }
        write_txn.commit()?;
        debug!("Saved booking: {}", booking.id);
        Ok(())
    }

    pub async fn get_booking(&self, id: &str) -> Result<Option<Booking>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(BOOKINGS)?;
        match table.get(id)? {
            Some(value) => Ok(Some(self.deserialize(value.value())?)),
            None => Ok(None),
        }
    }

    pub async fn list_bookings(&self) -> Result<Vec<Booking>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(BOOKINGS)?;
        let mut bookings = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            bookings.push(self.deserialize(value.value())?);
        }
        Ok(bookings)
    }

    pub async fn list_bookings_by_user(&self, user_id: &str) -> Result<Vec<Booking>> {
        let all_bookings = self.list_bookings().await?;
        Ok(all_bookings
            .into_iter()
            .filter(|b| b.user_id.to_string() == user_id)
            .collect())
    }

    pub async fn delete_booking(&self, id: &str) -> Result<bool> {
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        let existed = {
            let mut table = write_txn.open_table(BOOKINGS)?;
            let existed = table.remove(id)?.is_some();
            existed
        };
        write_txn.commit()?;
        if existed {
            debug!("Deleted booking: {}", id);
        }
        Ok(existed)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // VEHICLE OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub async fn save_vehicle(&self, vehicle: &Vehicle) -> Result<()> {
        let id = vehicle.id.to_string();
        let data = self.serialize(vehicle)?;
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(VEHICLES)?;
            table.insert(id.as_str(), data.as_slice())?;
        }
        write_txn.commit()?;
        debug!("Saved vehicle: {} ({})", vehicle.plate, vehicle.id);
        Ok(())
    }

    pub async fn get_vehicle(&self, id: &str) -> Result<Option<Vehicle>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(VEHICLES)?;
        match table.get(id)? {
            Some(value) => Ok(Some(self.deserialize(value.value())?)),
            None => Ok(None),
        }
    }

    pub async fn list_vehicles_by_user(&self, user_id: &str) -> Result<Vec<Vehicle>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(VEHICLES)?;
        let mut vehicles = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            let vehicle: Vehicle = self.deserialize(value.value())?;
            if vehicle.user_id.to_string() == user_id {
                vehicles.push(vehicle);
            }
        }
        Ok(vehicles)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HOMEOFFICE OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub async fn save_homeoffice_settings(&self, settings: &HomeofficeSettings) -> Result<()> {
        let id = settings.user_id.to_string();
        let data = self.serialize(settings)?;
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(HOMEOFFICE)?;
            table.insert(id.as_str(), data.as_slice())?;
        }
        write_txn.commit()?;
        debug!("Saved homeoffice settings for user: {}", settings.user_id);
        Ok(())
    }

    pub async fn get_homeoffice_settings(&self, user_id: &str) -> Result<Option<HomeofficeSettings>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(HOMEOFFICE)?;
        match table.get(user_id)? {
            Some(value) => Ok(Some(self.deserialize(value.value())?)),
            None => Ok(None),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // LOT LAYOUT OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub async fn save_lot_layout(&self, lot_id: &str, layout: &LotLayout) -> Result<()> {
        let data = self.serialize(layout)?;
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(LOT_LAYOUTS)?;
            table.insert(lot_id, data.as_slice())?;
        }
        write_txn.commit()?;
        debug!("Saved lot layout for lot: {}", lot_id);
        Ok(())
    }

    pub async fn get_lot_layout(&self, lot_id: &str) -> Result<Option<LotLayout>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(LOT_LAYOUTS)?;
        match table.get(lot_id)? {
            Some(value) => Ok(Some(self.deserialize(value.value())?)),
            None => Ok(None),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SETTINGS OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(SETTINGS)?;
        match table.get(key)? {
            Some(value) => Ok(Some(value.value().to_string())),
            None => Ok(None),
        }
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(SETTINGS)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WAITLIST OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub async fn save_waitlist_entry(&self, entry: &WaitlistEntry) -> Result<()> {
        let id = entry.id.to_string();
        let data = self.serialize(entry)?;
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(WAITLIST)?;
            table.insert(id.as_str(), data.as_slice())?;
        }
        write_txn.commit()?;
        debug!("Saved waitlist entry: {}", entry.id);
        Ok(())
    }

    pub async fn list_waitlist_by_lot(&self, lot_id: &str, date: Option<&str>) -> Result<Vec<WaitlistEntry>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(WAITLIST)?;
        let mut entries = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            let e: WaitlistEntry = self.deserialize(value.value())?;
            if e.lot_id.to_string() == lot_id {
                if let Some(d) = date {
                    if e.date == d { entries.push(e); }
                } else {
                    entries.push(e);
                }
            }
        }
        entries.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(entries)
    }

    pub async fn delete_waitlist_entry(&self, id: &str) -> Result<bool> {
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        let existed = {
            let mut table = write_txn.open_table(WAITLIST)?;
            let x = table.remove(id)?.is_some(); x
        };
        write_txn.commit()?;
        Ok(existed)
    }

    pub async fn list_all_waitlist(&self) -> Result<Vec<WaitlistEntry>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(WAITLIST)?;
        let mut entries = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            entries.push(self.deserialize(value.value())?);
        }
        Ok(entries)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PUSH SUBSCRIPTION OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════════

    pub async fn save_push_subscription(&self, sub: &PushSubscription) -> Result<()> {
        let key = format!("{}:{}", sub.user_id, sub.endpoint);
        let data = self.serialize(sub)?;
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(PUSH_SUBSCRIPTIONS)?;
            table.insert(key.as_str(), data.as_slice())?;
        }
        write_txn.commit()?;
        debug!("Saved push subscription for user: {}", sub.user_id);
        Ok(())
    }

    pub async fn list_push_subscriptions_by_user(&self, user_id: &str) -> Result<Vec<PushSubscription>> {
        let db = self.inner.read().await;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(PUSH_SUBSCRIPTIONS)?;
        let prefix = format!("{}:", user_id);
        let mut subs = Vec::new();
        for entry in table.iter()? {
            let (key, value) = entry?;
            if key.value().starts_with(&prefix) {
                subs.push(self.deserialize(value.value())?);
            }
        }
        Ok(subs)
    }


    /// Delete a vehicle by ID
    pub async fn delete_vehicle(&self, id: &str) -> Result<bool> {
        let db = self.inner.write().await;
        let write_txn = db.begin_write()?;
        let existed = {
            let mut table = write_txn.open_table(VEHICLES)?;
            let result = table.remove(id)?.is_some();
            result
        };
        write_txn.commit()?;
        if existed {
            tracing::debug!("Deleted vehicle: {}", id);
        }
        Ok(existed)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_config(path: PathBuf, encrypted: bool) -> DatabaseConfig {
        DatabaseConfig {
            path,
            encryption_enabled: encrypted,
            passphrase: if encrypted {
                Some("test-passphrase".to_string())
            } else {
                None
            },
            create_if_missing: true,
        }
    }

    #[tokio::test]
    async fn test_database_create() {
        let dir = tempdir().unwrap();
        let config = test_config(dir.path().to_path_buf(), false);
        let db = Database::open(config).unwrap();
        assert!(!db.is_encrypted());
        assert!(db.is_fresh().await.unwrap());
    }

    #[tokio::test]
    async fn test_database_encrypted() {
        let dir = tempdir().unwrap();
        let config = test_config(dir.path().to_path_buf(), true);
        let db = Database::open(config).unwrap();
        assert!(db.is_encrypted());
    }

    #[tokio::test]
    async fn test_setup_completed() {
        let dir = tempdir().unwrap();
        let config = test_config(dir.path().to_path_buf(), false);
        let db = Database::open(config).unwrap();
        assert!(db.is_fresh().await.unwrap());
        db.mark_setup_completed().await.unwrap();
        assert!(!db.is_fresh().await.unwrap());
    }

    #[tokio::test]
    async fn test_settings() {
        let dir = tempdir().unwrap();
        let config = test_config(dir.path().to_path_buf(), false);
        let db = Database::open(config).unwrap();
        assert!(db.get_setting("test_key").await.unwrap().is_none());
        db.set_setting("test_key", "test_value").await.unwrap();
        assert_eq!(
            db.get_setting("test_key").await.unwrap(),
            Some("test_value".to_string())
        );
    }

    #[tokio::test]
    async fn test_stats() {
        let dir = tempdir().unwrap();
        let config = test_config(dir.path().to_path_buf(), false);
        let db = Database::open(config).unwrap();
        let stats = db.stats().await.unwrap();
        assert_eq!(stats.users, 0);
        assert_eq!(stats.bookings, 0);
    }
}
