#!/bin/sh

# try reading a file if it exists, then 
#dfx canister call fs_tests_backend read_file '(".test_hidden/testdir/test.txt")'

# write some content into file
dfx canister call fs_tests_backend write_file '(".test_hidden/testdir/test.txt", "some test content")'

# try reading again
dfx canister call fs_tests_backend read_file '(".test_hidden/testdir/test.txt")'

# compute hash
dfx canister call fs_tests_backend compute_file_hash '(".test_hidden/testdir/test.txt")'

