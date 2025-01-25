#!/bin/sh

CLICOLOR_FORCE=1 solvewasm/build.sh -s || exit 1

cp solvewasm/index.html . || exit 1
