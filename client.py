"""
client.py

http client utility.

Written by Carotene.
"""
from twisted.internet.defer import inlineCallbacks
from twisted.internet.utils import getProcessOutputAndValue


@inlineCallbacks
def request_get(reactor, url, timeout=5):
    """
    Request GET method.

    Args:
        reactor (reactor): twisted reactor
        url (str): target URL

    Keyword Args:
        timeout (int): timeout seconds (default: 5)

    Returns:
        bytes: response body or None
    """
    out, err, code = yield getProcessOutputAndValue(
        '/usr/bin/curl',
        ['-m', str(timeout), '-o', '-', url],
        reactor=reactor
    )

    if code != 0:
        return None

    return out


# EOF
