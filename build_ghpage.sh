#!/bin/sh

CLICOLOR_FORCE=1 solvewasm/build.sh || exit 1

rm -rf .github/ghpage || exit 1

cp -r solvewasm/html .github/ghpage || exit 1
