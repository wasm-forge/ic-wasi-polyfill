#!/bin/bash

dfx canister call benchmark_test_backend append_chunk '("abc1234567", 10_000_000: nat64, 100_000_000: nat64 )'


