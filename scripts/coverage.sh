#!/bin/bash


cargo llvm-cov --features report_wasi_calls,transient,skip_unimplemented_functions --ignore-filename-regex='(wasi_mock\.rs|bindings|canisters)' --workspace --html