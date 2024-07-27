#!/bin/bash

dfx canister call benchmark_test_backend read_chunk '(99999988:nat64, 10:nat64)'
