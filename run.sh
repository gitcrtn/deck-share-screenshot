#!/usr/bin/env bash
SH=$(realpath $0)
ROOTDIR=$(dirname $SH)
cd $ROOTDIR

. venv/bin/activate
python sharess.py
