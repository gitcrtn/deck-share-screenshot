#!/usr/bin/env bash
if [ -d ./venv ]; then
    echo venv already exists.
    exit 1
fi

if [ ! -f ./pycairo-*.whl -o ! -f ./PyGObject-*.whl ]; then
    echo Prepare whl files for pygobject.
    exit 1
fi

python3 -m venv venv
. venv/bin/activate
pip install ./pycairo-*.whl
pip install ./PyGObject-*.whl
pip install -r requirements.txt
deactivate
