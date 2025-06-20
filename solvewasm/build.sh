#!/bin/bash

# Set the directory of the script
SELF=$(readlink -f -- "$0")
DIR=$(dirname -- "$SELF")

# Get current directory
CWD=$(pwd)

# Restore current directory on exit
trap cleanup INT

cleanup() {
	cd "$CWD" || exit 1
}

# Parse command line options
OPEN=N
SINGLE=N

usage() {
	echo "Usage: $0 [-o] [-s]"
	echo "  Where:"
	echo "    -o = open page after generating"
	echo "    -s = Build single page site"
	exit 1
}

while getopts ":os" OPT
do
    case "${OPT}" in
        o)
            OPEN=Y
            ;;
        s)
            SINGLE=Y
            ;;
        *)
            usage
            ;;
    esac
done

shift $((OPTIND-1))

launch_url() {
	url="$1"
	shift

	delay="$1"

	if [ "$delay" != "" ]
	then
		sleep "$delay"
	fi

	if command -v xdg-open > /dev/null
	then
		echo "Opening with 'xdg-open': $url"
		xdg-open "$url"
	elif command -v open > /dev/null
	then
		echo "Opening with 'open': $url"
		open "$url"
	else
		echo "No command to open a URL"
		exit 1
	fi
}

# Change to the directory of the script
cd "$DIR" || exit 1

# Run wasm-pack to build the web assembly library and js linkage
wasm-pack build --target web || exit 1

# Run cargo to build the packaging binary
cargo build --manifest-path package/Cargo.toml --release || exit 1

if [ "$SINGLE" == "Y" ]
then
	# Package in to the single directory
	../target/release/package --single template/index.html . || exit 1

	# Print file details
	ls -l index.html

	if [ "$OPEN" == "Y" ]
	then
		# Open the generated file
		launch_url index.html
	fi
else
	# Package in to the html directory
	../target/release/package template/index.html html || exit 1

	# Print file details
	ls -l html

	if [ "$OPEN" == "Y" ]
	then
		# Run python web server
		launch_url http://127.0.0.1:8000/ 2 &

		python3 -m http.server 8000 --bind 0.0.0.0 -d html
	fi
fi
