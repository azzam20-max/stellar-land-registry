#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype,
    symbol_short, Env, String, Symbol, Vec
};

// =====================
// STRUKTUR DATA
// =====================

#[contracttype]
#[derive(Clone, Debug)]
pub struct LandRecord {
    pub cert_id: String,       // ID Sertifikat (unik, misal: "SHM-001")
    pub location: String,      // Lokasi / alamat tanah
    pub area_sqm: u64,         // Luas tanah dalam meter persegi
    pub owner_name: String,    // Nama pemilik saat ini
    pub owner_id: String,      // NIK / ID pemilik
    pub registered_at: u64,    // Timestamp pendaftaran
    pub is_active: bool,       // Status (false = sudah ditransfer/dihapus)
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct TransferHistory {
    pub cert_id: String,
    pub from_owner: String,    // Nama pemilik lama
    pub to_owner: String,      // Nama pemilik baru
    pub transferred_at: u64,   // Timestamp transfer
    pub notes: String,         // Catatan (misal: "Jual beli", "Waris")
}

// =====================
// STORAGE KEYS
// =====================
const LAND_DATA: Symbol = symbol_short!("LAND_DATA");
const HIST_DATA: Symbol = symbol_short!("HIST_DATA");

// =====================
// CONTRACT
// =====================
#[contract]
pub struct LandRegistryContract;

#[contractimpl]
impl LandRegistryContract {

    // -----------------------------------------------
    // 1. Daftarkan tanah baru
    // -----------------------------------------------
    pub fn register_land(
        env: Env,
        cert_id: String,
        location: String,
        area_sqm: u64,
        owner_name: String,
        owner_id: String,
    ) -> String {
        let mut lands: Vec<LandRecord> = env
            .storage().instance()
            .get(&LAND_DATA)
            .unwrap_or(Vec::new(&env));

        // Cek duplikat Certificate ID
        for i in 0..lands.len() {
            let land = lands.get(i).unwrap();
            if land.cert_id == cert_id {
                return String::from_str(&env, "ERROR: Certificate ID already registered");
            }
        }

        let new_land = LandRecord {
            cert_id,
            location,
            area_sqm,
            owner_name,
            owner_id,
            registered_at: env.ledger().timestamp(),
            is_active: true,
        };

        lands.push_back(new_land);
        env.storage().instance().set(&LAND_DATA, &lands);

        String::from_str(&env, "Land registered successfully")
    }

    // -----------------------------------------------
    // 2. Transfer kepemilikan tanah
    // -----------------------------------------------
    pub fn transfer_land(
        env: Env,
        cert_id: String,
        new_owner_name: String,
        new_owner_id: String,
        notes: String,
    ) -> String {
        let mut lands: Vec<LandRecord> = env
            .storage().instance()
            .get(&LAND_DATA)
            .unwrap_or(Vec::new(&env));

        let mut histories: Vec<TransferHistory> = env
            .storage().instance()
            .get(&HIST_DATA)
            .unwrap_or(Vec::new(&env));

        for i in 0..lands.len() {
            let land = lands.get(i).unwrap();

            if land.cert_id == cert_id {
                if !land.is_active {
                    return String::from_str(&env, "ERROR: Land record is inactive");
                }

                // Simpan riwayat transfer
                let history = TransferHistory {
                    cert_id: land.cert_id.clone(),
                    from_owner: land.owner_name.clone(),
                    to_owner: new_owner_name.clone(),
                    transferred_at: env.ledger().timestamp(),
                    notes,
                };
                histories.push_back(history);

                // Update data pemilik
                let updated_land = LandRecord {
                    cert_id: land.cert_id,
                    location: land.location,
                    area_sqm: land.area_sqm,
                    owner_name: new_owner_name,
                    owner_id: new_owner_id,
                    registered_at: land.registered_at,
                    is_active: true,
                };

                lands.set(i, updated_land);
                env.storage().instance().set(&LAND_DATA, &lands);
                env.storage().instance().set(&HIST_DATA, &histories);

                return String::from_str(&env, "Ownership transferred successfully");
            }
        }

        String::from_str(&env, "ERROR: Land record not found")
    }

    // -----------------------------------------------
    // 3. Lihat semua data tanah
    // -----------------------------------------------
    pub fn get_all_lands(env: Env) -> Vec<LandRecord> {
        env.storage().instance()
            .get(&LAND_DATA)
            .unwrap_or(Vec::new(&env))
    }

    // -----------------------------------------------
    // 4. Lihat riwayat transfer berdasarkan cert_id
    // -----------------------------------------------
    pub fn get_transfer_history(env: Env, cert_id: String) -> Vec<TransferHistory> {
        let histories: Vec<TransferHistory> = env
            .storage().instance()
            .get(&HIST_DATA)
            .unwrap_or(Vec::new(&env));

        let mut result: Vec<TransferHistory> = Vec::new(&env);

        for i in 0..histories.len() {
            let h = histories.get(i).unwrap();
            if h.cert_id == cert_id {
                result.push_back(h);
            }
        }

        result
    }

    // -----------------------------------------------
    // 5. Nonaktifkan sertifikat (misal: sengketa)
    // -----------------------------------------------
    pub fn deactivate_land(env: Env, cert_id: String, reason: String) -> String {
        let mut lands: Vec<LandRecord> = env
            .storage().instance()
            .get(&LAND_DATA)
            .unwrap_or(Vec::new(&env));

        for i in 0..lands.len() {
            let land = lands.get(i).unwrap();

            if land.cert_id == cert_id {
                let updated = LandRecord {
                    cert_id: land.cert_id.clone(),
                    location: land.location.clone(),
                    area_sqm: land.area_sqm,
                    owner_name: land.owner_name.clone(),
                    owner_id: land.owner_id.clone(),
                    registered_at: land.registered_at,
                    is_active: false,
                };
                lands.set(i, updated);
                env.storage().instance().set(&LAND_DATA, &lands);

                // Simpan alasan nonaktif ke history
                let mut histories: Vec<TransferHistory> = env
                    .storage().instance()
                    .get(&HIST_DATA)
                    .unwrap_or(Vec::new(&env));

                let log = TransferHistory {
                    cert_id,
                    from_owner: String::from_str(&env, "SYSTEM"),
                    to_owner: String::from_str(&env, "DEACTIVATED"),
                    transferred_at: env.ledger().timestamp(),
                    notes: reason,
                };
                histories.push_back(log);
                env.storage().instance().set(&HIST_DATA, &histories);

                return String::from_str(&env, "Land record deactivated");
            }
        }

        String::from_str(&env, "ERROR: Land record not found")
    }
}

mod test;
