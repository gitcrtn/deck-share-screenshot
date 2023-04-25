"""
app.py

application class and data classes.

- SharessApp
- SteamApp
- Image

Written by Carotene.
"""
import os
import glob
import json
from uuid import uuid4
from dataclasses import dataclass

from twisted.internet.defer import inlineCallbacks

from client import request_get


URL_GET_APP_LIST = 'https://api.steampowered.com/ISteamApps/GetAppList/v2/'

URL_SHARE_FORMAT = 'http://{server_host}/{uuid}'

SCREENSHOT_PATH = '.steam/steam/userdata/*/*/remote/*/screenshots/*.jpg'
BUTTON_IMAGE_PATH = '.local/share/Steam/tenfoot/resource/images/library/controller/api/*.png'

JSON_API_APPLIST = 'applist.json'


@dataclass
class SteamApp:
    """
    SteamApp

    application data by app_id.
    """
    app_id: str
    title: str


@dataclass
class Image:
    """
    Image

    image data by filepath.
    """
    filepath: str
    filename: str
    app_id: str


class SharessApp:
    """SharessApp"""

    def __init__(self, myreactor):
        """
        constructor

        Args:
            myreactor (reactor): twisted reactor
        """
        self.reactor = myreactor
        self._server_host = ''
        self._app_ids = {}
        self.images = {}
        self.steam_apps = {}
        self._shared_image = None
        self._shared_uuid = None

    def set_server_host(self, ip, port):
        """
        Set server host.

        Args:
            ip (str): IP address
            port (int|str): port
        """
        self._server_host = f'{ip}:{port}'

    def has_image(self, uuid_path):
        """
        Check if image is shared with specified uuid path.

        Args:
            uuid_path (bytes): URL path

        Returns:
            bool: True if image is shared by specified uuid path.
        """
        return uuid_path.decode()[1:] == self._shared_uuid

    def get_image(self, uuid_path):
        """
        Get image.

        Args:
            uuid_path (bytes): URL path

        Returns:
            Image: shared image
        """
        if self.has_image(uuid_path):
            return self._shared_image
        return None

    def share(self, image):
        """
        Start to share image.

        Args:
            image (Image): target image
        """
        self._shared_uuid = str(uuid4())
        self._shared_image = image
        return URL_SHARE_FORMAT.format(
            server_host=self._server_host,
            uuid=self._shared_uuid)

    def stop_share(self):
        """
        Stop to share image.
        """
        self._shared_uuid = None
        self._shared_image = None

    def _is_empty_env(self, value):
        """
        Check if env value is empty.

        Args:
            value (str): target env value or None

        Returns:
            bool: True if value is empty
        """
        return not value or not value.strip()

    def check_env(self):
        """
        Check environment values.
        """
        self.homedir = os.getenv('HOMEDIR')

        if self._is_empty_env(self.homedir):
            self.homedir = os.getenv('HOME')

        if self._is_empty_env(self.homedir):
            raise RuntimeError('HOME not defined.')

        self.cachedir = os.getenv('CACHEDIR')

        if self._is_empty_env(self.cachedir):
            self.cachedir = os.path.join(self.homedir, '.cache', 'sharess')

        if not os.path.isdir(self.cachedir):
            os.makedirs(self.cachedir)

        if not os.path.isdir(self.cachedir):
            raise RuntimeError('CACHEDIR not found.')

        print(f'HOMEDIR: {self.homedir}')
        print(f'CACHEDIR: {self.cachedir}')

    def get_app_title(self, appid):
        """
        Get application title.

        Args:
            appid (str): application id

        Returns:
            str: application title or None
        """
        return self._app_ids.get(appid)

    def _get_images(self):
        """
        Get image files from storage and create data objects.
        """
        found_images = glob.glob(self.ss_search_path)

        for image_path in found_images:
            path_items = image_path.split('/')
            filename = path_items[-1]
            appid = path_items[-3]
            image = Image(image_path, filename, appid)
            if appid not in self.images:
                self.images[appid] = {}
            self.images[appid][filename] = image

        for appid in self.images:
            title = self.get_app_title(appid)
            app = SteamApp(appid, title)
            self.steam_apps[appid] = app

    def _load_appids(self):
        """
        Load app ids dictionary from applist.json.
        """
        with open(self.applist_json_path, 'rb') as f:
            applist = json.load(f)
        for item in applist['applist']['apps']:
            self._app_ids[str(item['appid'])] = item['name']

    @inlineCallbacks
    def _update_applist(self):
        """
        Update applist.json to fetch Web API.
        """
        body = yield request_get(self.reactor, URL_GET_APP_LIST)
        if not body:
            return
        with open(self.applist_json_path, 'wb') as f:
            f.write(body)

    @inlineCallbacks
    def setup(self):
        """
        Setup application.
        """
        self.ss_search_path = os.path.join(self.homedir, SCREENSHOT_PATH)
        self.button_search_path = os.path.join(self.homedir, BUTTON_IMAGE_PATH)
        self.applist_json_path = os.path.join(self.cachedir, JSON_API_APPLIST)

        if not os.path.isfile(self.applist_json_path):
            yield self._update_applist()

        self._load_appids()
        self._get_images()


# EOF
