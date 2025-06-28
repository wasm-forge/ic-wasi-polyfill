mod canister;
fn main() {
    let scan = canister::do_fs_test_basic();

    println!("{scan}");
}
