#!/bin/bash

# Run wasm-pack to build the web assembly library and js linkage
wasm-pack build --target web

if [ $? -ne 0 ]
then
	exit
fi

# Run cargo to build the packaging binary
cargo build --manifest-path package/Cargo.toml --release

if [ $? -ne 0 ]
then
	exit
fi

# Package in to the html directory
../target/release/package template/index.htm html

if [ $? -ne 0 ]
then
	exit
fi

# Print file details
ls -l html

# Run python web server
./LaunchURL.sh http://127.0.0.1:8000/ 2 &

python3 -m http.server 8000 --bind 127.0.0.1 -d html
