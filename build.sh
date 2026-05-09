#!/bin/sh
cargo build -r
cp target/release/pkg Paketerstellung/Arch/pkg-bin
cp man/pkg.1 Paketerstellung/Arch/pkg.1

cp target/release/pkg Paketerstellung/pkg_amd64/pkg
cp man/pkg.1 Paketerstellung/pkg_amd64/pkg.1

cp target/release/pkg Paketerstellung/pkg_arm64/pkg
cp man/pkg.1 Paketerstellung/pkg_arm64/pkg.1
