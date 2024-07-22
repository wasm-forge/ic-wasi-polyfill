#!/bin/bash

dfx canister call benchmark_test_backend read_text '("test.txt", 99999988:nat64, 10:nat64)'
