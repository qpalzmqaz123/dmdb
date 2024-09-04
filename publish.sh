#!/bin/bash

echo ">>> Publish dmdb-sys"
cargo publish -p dmdb-sys --registry=xnt

echo ">>> Publish dmdb"
cargo publish -p dmdb --registry=xnt

