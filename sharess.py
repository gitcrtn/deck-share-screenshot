"""
sharess.py

GUI classes for deck-share-screenshot.

- SharessUI
- ImageRow

Written by Carotene.
"""
import gi
gi.require_version('Gtk', '3.0')

from twisted.internet import gtk3reactor
gtk3reactor.install()

from twisted.internet.defer import inlineCallbacks

from gi.repository import Gtk, GdkPixbuf, GLib
import qrcode

from server import Server
from app import SharessApp


class ImageRow(Gtk.ListBoxRow):
    """ImageRow"""

    def __init__(self, image):
        """
        constructor

        Args:
            image (app.Image): image instance
        """
        super().__init__()
        self.data = image
        self.add(Gtk.Label(label=image.filename))


class SharessUI:
    """SharessUI"""

    def __init__(self, myreactor):
        """
        constructor

        Args:
            myreactor (reactor): twisted reactor
        """
        self.reactor = myreactor
        self.app = SharessApp(self.reactor)
        self.server = Server(self.reactor, self.app)
        self.app.check_env()
        self._current_app = 'ALL'
        self._steam_apps = {}
        self._selected_image = None
        self._image_pixbuf = None

    def setup_widgets(self):
        """
        setup widgets.
        """
        self.builder = Gtk.Builder()
        self.builder.add_from_file('sharess.glade')

        self.window = self.builder.get_object('main_window')
        self.window.connect("destroy", Gtk.main_quit)

        self.dialog = self.builder.get_object('dialog_share')
        self.label_share = self.builder.get_object('label_share')
        self.image_share_qr = self.builder.get_object('image_share_qr')
        self.bt_share_cancel = self.builder.get_object('bt_share_cancel')
        self.bt_share_cancel.connect('clicked', self.on_share_closed)

        self.bt_share = self.builder.get_object('bt_share')
        self.bt_share.connect('clicked', self.on_share_button_clicked)

        self.image_preview = self.builder.get_object('image_preview')
        self.image_preview.connect('size-allocate', self.on_image_resized)

        self.cb_app_filter = self.builder.get_object('cb_app_filter')
        self.cb_app_filter.set_entry_text_column(0)
        self.cb_app_filter.connect("changed", self.on_app_filter_changed)

        self.lb_images = self.builder.get_object('lb_images')
        self.lb_images.connect("row-selected", self.on_image_row_selected)
        self.lb_images.set_filter_func(self.filter_images, None, False)
        self.lb_images.set_sort_func(self.sort_images, None, False)

    def update_image_list(self):
        """
        Update listbox for images.
        """
        if self._current_app != 'ALL':
            for image in self.app.images[self._current_app.app_id].values():
                self.lb_images.add(ImageRow(image))
            return

        for images in self.app.images.values():
            for image in images.values():
                self.lb_images.add(ImageRow(image))

    def update_app_filters(self):
        """
        Update combobox for application filter.
        """
        self.cb_app_filter.remove_all()
        self._steam_apps.clear()

        self.cb_app_filter.append_text('ALL')

        labels = []

        for steam_app in self.app.steam_apps.values():
            label_text = f'{steam_app.title} ({steam_app.app_id})'
            self._steam_apps[label_text] = steam_app
            labels.append(label_text)

        labels.sort()

        for label_text in labels:
            self.cb_app_filter.append_text(label_text)

        self.cb_app_filter.set_active(0)

    def load_images(self):
        """
        Load widgets for images.
        """
        self.update_app_filters()
        self.update_image_list()

    def sort_images(self, row_1, row_2, unused_data, unused_notify_destroy):
        """
        Sort images for listbox.

        Args:
            row_1 (ImageRow): image row
            row_2 (ImageRow): image row
        """
        return row_1.data.filename.lower() < row_2.data.filename.lower()

    def filter_images(self, row, unused_data, unused_notify_destroy):
        """
        Filter images for listbox with selected application.

        Args:
            row (ImageRow): target image row
        """
        if self._current_app == 'ALL':
            return True
        return row.data.app_id == self._current_app.app_id

    def on_share_closed(self, *args, **kwargs):
        """
        An event handler for stopping to share image
        when cancel button is clicked on share dialog.
        """
        self.app.stop_share()
        self.dialog.set_keep_above(False)
        self.dialog.hide()

    def on_image_resized(self, *args, **kwargs):
        """
        An event handler for resizing image
        when window is resized.
        """
        if not self._image_pixbuf:
            return
        self._set_scaled_image()

    def on_share_button_clicked(self, *args, **kwargs):
        """
        An event handler for starting to share image
        when share button is clicked on main window.
        """
        if not self._selected_image:
            return
        url = self.app.share(self._selected_image)
        im = qrcode.make(url)
        data = GLib.Bytes.new(im.convert('RGB').tobytes())
        width, height = im.size
        pixbuf = GdkPixbuf.Pixbuf.new_from_bytes(
            data, GdkPixbuf.Colorspace.RGB,
            False, 8, width, height, width * 3)
        self.label_share.set_text(f'Access to: {url}')
        self.image_share_qr.set_from_pixbuf(pixbuf)
        self.dialog.show_all()
        self.dialog.set_keep_above(True)

    def _set_scaled_image(self):
        """
        Set pixbuf to preview image to fit widget size.
        """
        allocation = self.image_preview.get_allocation()
        pixbuf = self._image_pixbuf.scale_simple(
            allocation.width,
            allocation.height,
            GdkPixbuf.InterpType.BILINEAR)
        self.image_preview.set_from_pixbuf(pixbuf)

    def on_image_row_selected(self, lb, row):
        """
        An event handler for updating preview
        when image is selected on listbox.
        """
        self._selected_image = row.data
        self._image_pixbuf = GdkPixbuf.Pixbuf.new_from_file(self._selected_image.filepath)
        self._set_scaled_image()

    def on_app_filter_changed(self, combo):
        """
        An event handler for updating selected application
        when application is selected on combobox.
        """
        text = combo.get_active_text()

        if text is not None:
            if text == 'ALL':
                self._current_app = 'ALL'
            else:
                self._current_app = self._steam_apps[text]

            self.lb_images.invalidate_filter()
            self.lb_images.invalidate_sort()

    @inlineCallbacks
    def _run(self):
        """
        entrypoint
        """
        yield self.app.setup()
        self.server.setup()
        self.setup_widgets()
        self.load_images()
        self.window.show_all()
        self.window.maximize()

    def run(self):
        """
        entrypoint to call from external.
        """
        self.reactor.callWhenRunning(self._run)
        self.reactor.run()


def main():
    """
    entrypoint for global space
    """
    from twisted.internet import reactor
    ui = SharessUI(reactor)
    ui.run()


if __name__ == '__main__':
    main()


# EOF
