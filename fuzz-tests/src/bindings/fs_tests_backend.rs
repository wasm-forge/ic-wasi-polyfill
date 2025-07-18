// This is an experimental feature used to generate Rust bindings from Candid.
// THIS IS A GENERATED FILE. DO NOT EDIT THIS FILE TO AVOID DATA LOSS.
#![allow(dead_code, unused_imports, non_snake_case)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};

pub struct FsTestsBackendCanister {
    pub canister_id: Principal,
    pub caller: super::Caller,
}

impl FsTestsBackendCanister {
    pub fn basic_fs_test(&self) -> super::CallBuilder<()> {
        let args = Encode!();
        self.caller.call(
            self.canister_id,
            super::CallMode::Update,
            "basic_fs_test",
            args,
        )
    }
    pub fn compute_file_hash(&self, arg0: String) -> super::CallBuilder<String> {
        let args = Encode!(&arg0);
        self.caller.call(
            self.canister_id,
            super::CallMode::Update,
            "compute_file_hash",
            args,
        )
    }
    pub fn do_fs_test(&self) -> super::CallBuilder<String> {
        let args = Encode!();
        self.caller.call(
            self.canister_id,
            super::CallMode::Update,
            "do_fs_test",
            args,
        )
    }
    pub fn generate_random_fs(&self, arg0: u64, arg1: u64, arg2: u64) -> super::CallBuilder<u64> {
        let args = Encode!(&arg0, &arg1, &arg2);
        self.caller.call(
            self.canister_id,
            super::CallMode::Update,
            "generate_random_fs",
            args,
        )
    }
    pub fn get_log(&self) -> super::CallBuilder<String> {
        let args = Encode!();
        self.caller
            .call(self.canister_id, super::CallMode::Query, "get_log", args)
    }
    pub fn greet(&self, arg0: String) -> super::CallBuilder<String> {
        let args = Encode!(&arg0);
        self.caller
            .call(self.canister_id, super::CallMode::Query, "greet", args)
    }
    pub fn read_file(&self, arg0: String) -> super::CallBuilder<String> {
        let args = Encode!(&arg0);
        self.caller
            .call(self.canister_id, super::CallMode::Update, "read_file", args)
    }
    pub fn scan_directory(&self, arg0: String) -> super::CallBuilder<String> {
        let args = Encode!(&arg0);
        self.caller.call(
            self.canister_id,
            super::CallMode::Update,
            "scan_directory",
            args,
        )
    }
    pub fn test_create_dir_all(&self, arg0: String) -> super::CallBuilder<()> {
        let args = Encode!(&arg0);
        self.caller.call(
            self.canister_id,
            super::CallMode::Update,
            "test_create_dir_all",
            args,
        )
    }
    pub fn test_read_dir(&self, arg0: String) -> super::CallBuilder<Vec<String>> {
        let args = Encode!(&arg0);
        self.caller.call(
            self.canister_id,
            super::CallMode::Update,
            "test_read_dir",
            args,
        )
    }
    pub fn write_file(&self, arg0: String, arg1: String) -> super::CallBuilder<()> {
        let args = Encode!(&arg0, &arg1);
        self.caller.call(
            self.canister_id,
            super::CallMode::Update,
            "write_file",
            args,
        )
    }
}
pub const CANISTER_ID: Principal =
    Principal::from_slice(&[255, 255, 255, 255, 255, 224, 0, 2, 1, 1]); // lz3um-vp777-77777-aaaba-cai

pub fn new(caller: &super::Caller, canister_id: Principal) -> FsTestsBackendCanister {
    FsTestsBackendCanister {
        canister_id,
        caller: caller.clone(),
    }
}

pub fn deploy(deployer: &super::Deployer) -> super::DeployBuilder<FsTestsBackendCanister> {
    let args = Encode!();
    let result = deployer.deploy(args, new);
    let result = if let Some(id) = canister_id() {
        result.with_canister_id(id)
    } else {
        result
    };
    if let Some(wasm) = wasm() {
        result.with_wasm(wasm)
    } else {
        result
    }
}
pub fn canister_id() -> Option<Principal> {
    Some(Principal::from_text("lz3um-vp777-77777-aaaba-cai").unwrap())
}

pub fn wasm() -> Option<Vec<u8>> {
    let mut path = std::path::PathBuf::new();
    path.push("../target/wasm32-wasip1/release/fs_tests_backend_nowasi.wasm");
    let wasm = std::fs::read(path.as_path())
        .unwrap_or_else(|_| panic!("wasm binary not found: {:?}", path));
    Some(wasm)
}
