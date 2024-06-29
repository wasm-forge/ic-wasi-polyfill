//use candid::encode_one;
//use pocket_ic::PocketIc;

#[cfg(test)]
fn compile_hello_test() {
    use std::process::Command;
    let output = Command::new("echo")
        .arg("Hello world")
        .output()
        .expect("Failed to execute command");
    println!("{output:?}");

    let output = Command::new("pwd")
        .output()
        .expect("Failed to execute command");
    println!("{output:?}");

    let output = Command::new("cd")
        .arg("/home/stas/")
        .output()
        .expect("Failed to execute command");
    println!("{output:?}");

/* 
    let output = Command::new("pwd")
        .output()
        .expect("Failed to execute command");
    println!("{output:?}");
*/
}

#[test]
fn test_counter_canister() {
    compile_hello_test();

    /*
    let pic = PocketIc::new();
    // Create an empty canister as the anonymous principal and add cycles.
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);

    //let wasm_bytes = load_counter_wasm(...);

    pic.install_canister(canister_id, wasm_bytes, vec![], None);

    // 'inc' is a counter canister method.
    call_counter_canister(&pic, canister_id, "inc");

    // Check if it had the desired effect.
    let reply = call_counter_canister(&pic, canister_id, "read");

    assert_eq!(reply, WasmResult::Reply(vec![0, 0, 0, 1]));

    */
}
