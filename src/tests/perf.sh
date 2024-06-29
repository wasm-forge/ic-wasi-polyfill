#!ic-repl

function install(wasm, args, cycle) {
  let id = call ic.provisional_create_canister_with_cycles(record { settings = null; amount = cycle });
  let S = id.canister_id;
  let _ = call ic.install_code(
    record {
      arg = args;
      wasm_module = gzip(wasm);
      mode = variant { install };
      canister_id = S;
    }
  );
  S
};

function upgrade(id, wasm, args) {
  call ic.install_code(
    record {
      arg = args;
      wasm_module = gzip(wasm);
      mode = variant { upgrade };
      canister_id = id;
    }
  );
};

function uninstall(id) {
  call ic.stop_canister(record { canister_id = id });
  call ic.delete_canister(record { canister_id = id });
};

function get_memory(cid) {
  let _ = call ic.canister_status(record { canister_id = cid });
  _.memory_size
};

let file = "README.md";

let rs_config = record { start_page = 1; page_limit = 128};

let wasm_name = "benchmark_test/target/wasm32-wasi/release/benchmark_test_backend_nowasi.wasm";



function perf_file_write_10kb_fs() {
  let cid = install(wasm_profiling(wasm_name, rs_config), encode (), null);
  call cid.fs_write_kb_text( "files/test.txt", (10: nat64) );
  flamegraph(cid, "benchmark_10kb_write_fs", "svg/benchmark_10kb_write_fs.svg");
};

function perf_file_write_10kb() {
  let cid = install(wasm_profiling(wasm_name, rs_config), encode (), null);
  call cid.write_kb_text( "files/test.txt", (10: nat64) );
  flamegraph(cid, "benchmark_10kb_write", "svg/benchmark_10kb_write.svg");
};

function perf_file_write_10kb_buf() {
  let cid = install(wasm_profiling(wasm_name, rs_config), encode (), null);
  call cid.write_kb_text_buf( "files/test.txt", (10: nat64) );
  flamegraph(cid, "benchmark_10kb_buf_write", "svg/benchmark_10kb_buf_write.svg");
};

function perf_file_write_10Mb_buf() {
  let cid = install(wasm_profiling(wasm_name, rs_config), encode (), null);
  call cid.write_kb_text_buf( "files/test.txt", (10240: nat64) );
  flamegraph(cid, "benchmark_10Mb_buf_write", "svg/benchmark_10Mb_buf_write.svg");
};

function perf_list_files() {
  let cid = install(wasm_profiling(wasm_name, rs_config), encode (), null);
  call cid.list_files("files");
  flamegraph(cid, "list_files", "svg/list_files.svg");
};

function perf_create_files() {
  call cid.create_files( "files", (100: nat64) );
  flamegraph(cid, "create_files", "svg/create_files.svg");
};

function perf_create_folders() {
  call cid.create_depth_folders("files", (100: nat64));
  flamegraph(cid, "create_depth_folders", "svg/create_depth_folders.svg");
};

/// files

perf_file_write_10kb();
perf_file_write_10kb_buf();
perf_file_write_10kb_fs();

perf_file_write_10Mb_buf();

//perf_create_files();
//perf_delete_files();
//perf_list_files();

/// folders

//perf_create_folders();
//perf_delete_folders();

uninstall(cid);

//call cid.__toggle_tracing();
//call cid.list_files("files");
//call cid.__toggle_tracing();