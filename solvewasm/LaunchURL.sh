#!/bin/bash

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
