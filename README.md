# deck-share-screenshot

## Overview
This is a tool to share screenshot for Steamdeck.  
It can share screenshot by http server with QR code.

## Requires
* Python 3
* pygobject
* gtk3
* twisted
* qrcode
* pillow

## How to install
1. Prepare whl files for pygobject on Ubuntu 22.04 machine.
```
sudo apt install build-essential python3-dev libgirepository1.0-dev
pip wheel pygobject
```
2. Install tool and dependencies on Steamdeck as desktop mode.
```
git clone https://github.com/gitcrtn/deck-share-screenshot.git sharess/
cd sharess
# put whl files in here.
./install_deps.sh
./create_desktop.sh
```

## Usage
1. Click desktop file on Desktop.
2. Select application or ALL from combobox.
3. Select image from listbox.
4. Click share button.
5. Scan QR code from your phone.
6. Access to scanned URL.
7. Download image file to your phone.

## How to run for development
1. Run with environment values.
```
. venv/bin/activate
HOMEDIR=/path/to/homedir CACHEDIR=/path/to/cachedir python sharess.py
```

## How to uninstall
1. Remove tool.
```
rm ~/Desktop/sharess.desktop
rm -rf /path/to/sharess/
```
2. Remove cache directory.
```
rm -rf ~/.cache/sharess/
```

## Task list
- [x] share feature
- [ ] refresh button to refetch applist.json
- [ ] support flatpak for game mode on Steamdeck

## License
[MIT](https://github.com/gitcrtn/deck-share-screenshot/blob/master/LICENSE)

## Author
[Carotene](https://github.com/gitcrtn)
