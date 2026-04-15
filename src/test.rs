#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Ledger, Env, String};
    use crate::LandRegistryContractClient;

    #[test]
    fn test_register_land() {
        let env = Env::default();
        let contract_id = env.register_contract(None, crate::LandRegistryContract);
        let client = LandRegistryContractClient::new(&env, &contract_id);

        let result = client.register_land(
            &String::from_str(&env, "SHM-001"),
            &String::from_str(&env, "Jl. Merdeka No.1, Jakarta"),
            &500u64,
            &String::from_str(&env, "Budi Santoso"),
            &String::from_str(&env, "3175010101900001"),
        );

        assert_eq!(result, String::from_str(&env, "Land registered successfully"));
    }

    #[test]
    fn test_duplicate_cert_rejected() {
        let env = Env::default();
        let contract_id = env.register_contract(None, crate::LandRegistryContract);
        let client = LandRegistryContractClient::new(&env, &contract_id);

        client.register_land(
            &String::from_str(&env, "SHM-002"),
            &String::from_str(&env, "Jl. Sudirman No.5, Bandung"),
            &300u64,
            &String::from_str(&env, "Siti Rahayu"),
            &String::from_str(&env, "3273010202850002"),
        );

        // Coba daftar ulang dengan ID yang sama
        let result = client.register_land(
            &String::from_str(&env, "SHM-002"),
            &String::from_str(&env, "Jl. Lain No.10, Surabaya"),
            &200u64,
            &String::from_str(&env, "Penipu"),
            &String::from_str(&env, "0000000000000000"),
        );

        assert_eq!(result, String::from_str(&env, "ERROR: Certificate ID already registered"));
    }

    #[test]
    fn test_transfer_land() {
        let env = Env::default();
        env.ledger().set_timestamp(1000);
        let contract_id = env.register_contract(None, crate::LandRegistryContract);
        let client = LandRegistryContractClient::new(&env, &contract_id);

        client.register_land(
            &String::from_str(&env, "SHM-003"),
            &String::from_str(&env, "Jl. Pahlawan No.3, Yogyakarta"),
            &750u64,
            &String::from_str(&env, "Ahmad Fauzi"),
            &String::from_str(&env, "3404010303800003"),
        );

        let result = client.transfer_land(
            &String::from_str(&env, "SHM-003"),
            &String::from_str(&env, "Dewi Kusuma"),
            &String::from_str(&env, "3404020404900004"),
            &String::from_str(&env, "Jual beli"),
        );

        assert_eq!(result, String::from_str(&env, "Ownership transferred successfully"));

        // Cek history
        let history = client.get_transfer_history(&String::from_str(&env, "SHM-003"));
        assert_eq!(history.len(), 1);
        assert_eq!(history.get(0).unwrap().from_owner, String::from_str(&env, "Ahmad Fauzi"));
        assert_eq!(history.get(0).unwrap().to_owner, String::from_str(&env, "Dewi Kusuma"));
    }

    #[test]
    fn test_deactivate_land() {
        let env = Env::default();
        let contract_id = env.register_contract(None, crate::LandRegistryContract);
        let client = LandRegistryContractClient::new(&env, &contract_id);

        client.register_land(
            &String::from_str(&env, "SHM-004"),
            &String::from_str(&env, "Jl. Diponegoro No.7, Semarang"),
            &400u64,
            &String::from_str(&env, "Hendra Wijaya"),
            &String::from_str(&env, "3374010505750005"),
        );

        let result = client.deactivate_land(
            &String::from_str(&env, "SHM-004"),
            &String::from_str(&env, "Sengketa kepemilikan"),
        );

        assert_eq!(result, String::from_str(&env, "Land record deactivated"));

        // Coba transfer setelah dinonaktifkan - harus gagal
        let transfer_result = client.transfer_land(
            &String::from_str(&env, "SHM-004"),
            &String::from_str(&env, "Orang Lain"),
            &String::from_str(&env, "1234567890123456"),
            &String::from_str(&env, "Tidak valid"),
        );

        assert_eq!(transfer_result, String::from_str(&env, "ERROR: Land record is inactive"));
    }

    #[test]
    fn test_get_all_lands() {
        let env = Env::default();
        let contract_id = env.register_contract(None, crate::LandRegistryContract);
        let client = LandRegistryContractClient::new(&env, &contract_id);

        client.register_land(
            &String::from_str(&env, "SHM-005"),
            &String::from_str(&env, "Jl. A"),
            &100u64,
            &String::from_str(&env, "Pemilik A"),
            &String::from_str(&env, "1111111111111111"),
        );

        client.register_land(
            &String::from_str(&env, "SHM-006"),
            &String::from_str(&env, "Jl. B"),
            &200u64,
            &String::from_str(&env, "Pemilik B"),
            &String::from_str(&env, "2222222222222222"),
        );

        let lands = client.get_all_lands();
        assert_eq!(lands.len(), 2);
    }
}
