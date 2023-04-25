#!/usr/bin/env bash
sed -i -e "s|TOOLDIR|`pwd`|g" sharess.desktop
cp sharess.desktop ~/Desktop/
