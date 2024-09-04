#!/bin/bash

VERSION=$1

if [ -z $VERSION ]; then
    echo "Usage $0 X.X.X.[(alpha|beta|rc)-X]" >&2
    exit 1
fi

echo "Bump version $VERSION"

# 更新版本，只修改第一个遇到的 version = ".*" 的字段
sed -i "0,/version = \".*\"/s//version = \"$VERSION\"/" dmdb/Cargo.toml
sed -i "0,/version = \".*\"/s//version = \"$VERSION\"/" dmdb-sys/Cargo.toml

# 修改 lock 文件中的 version，直接让 cargo 自己改
cargo check

