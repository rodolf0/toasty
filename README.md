## Setup

Gnome docs https://developer.gnome.org/SearchProvider/

- You need a .search-provider.ini file somewhere in $(datadir)/gnome-shell/search-providers.
  For example: /usr/share/gnome-shell/search-providers/rodolf0.toasty.search-provider.ini
  There's a template at rodolf0.toasty.search-provider.ini
  NOTE: tried setting this in .local/share/gnome-shell/search-providers/ but didn't get loaded.

- The content has to reference a valid `.desktop` application in DesktopId
  For example: Toasty.desktop
