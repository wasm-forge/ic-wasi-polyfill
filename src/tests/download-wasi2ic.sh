#!/bin/bash

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  PLATFORM=linux
else
  echo "Unsupported platform $OSTYPE"
  exit 1
fi

curl -Ls https://github.com/wasm-forge/wasi2ic/releases/download/v0.2.11/wasi2ic -o wasi2ic || exit 1

chmod +x wasi2ic




