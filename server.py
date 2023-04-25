"""
server.py

server classes.

- Server
- ServerResource

Written by Carotene.
"""
import socket

from twisted.web.server import Site
from twisted.web.resource import Resource


class ServerResource(Resource):
    """ServerResource"""

    isLeaf = True

    def __init__(self, app, *args, **kwargs):
        """
        constructor

        Args:
            app (app.SharessApp): application
        """
        super().__init__(*args, **kwargs)
        self.app = app

    def render_GET(self, request):
        """
        router for GET method.

        Args:
            request (twisted.web.http.Request): request

        Returns:
            bytes: response body
        """
        if not self.app.has_image(request.uri):
            request.setResponseCode(404)
            return b'Not found'

        image = self.app.get_image(request.uri)

        request.setResponseCode(200)
        request.setHeader('Content-Type', 'application/force-download')
        request.setHeader('Content-Disposition', f'attachment; filename="{image.filename}"');

        with open(image.filepath, 'rb') as f:
            return f.read()


class Server:
    """Server"""

    def __init__(self, myreactor, app):
        """
        constructor

        Args:
            myreactor (reactor): twisted reactor
            app (app.SharessApp): application
        """
        self.reactor = myreactor
        self.app = app

    def setup(self):
        """
        Setup server.
        """
        self.site = Site(ServerResource(self.app))
        self.listening = self.reactor.listenTCP(0, self.site)
        self.ip = self.get_local_ip()
        self.port = self.listening.getHost().port
        self.app.set_server_host(self.ip, self.port)

    def get_local_ip(self):
        """
        Get local IP.

        Returns:
            str: IP address
        """
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(("8.8.8.8", 80))
        ip = s.getsockname()[0]
        s.close()
        return ip


# EOF
