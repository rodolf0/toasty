#!/usr/bin/env bash
set -e; set -o pipefail

USER=rudolph
HOME=/home/rudolph

# Seems need to install search provider globally :-(
# - https://gitlab.gnome.org/GNOME/gnome-shell/-/issues/3060
cp rodolf0.toasty.search-provider.ini \
  /usr/share/gnome-shell/search-providers

# Need a valid .desktop app to work
# Installing it globally to keep in sync with above
cp toasty.desktop /usr/share/applications/

install -d -o $USER "$HOME/.config/systemd/user"
install -o $USER toasty.service "$HOME/.config/systemd/user/toasty.service"

# run systemctl enable --user toasty
