# navrs
[![Actions Status](https://github.com/alemidev/navrs/actions/workflows/test.yml/badge.svg?branch=dev)](https://github.com/alemidev/navrs/actions/workflows/test.yml)
[![Actions Status](https://github.com/alemidev/navrs/actions/workflows/release.yml/badge.svg)](https://github.com/alemidev/navrs/actions/workflows/release.yml)
[![GitHub last commit](https://img.shields.io/github/last-commit/alemidev/navrs)](https://github.com/alemidev/navrs/commits/dev/)
> terminal music player for subsonic

![demo-gif](https://cdn.alemi.dev/proj/navrs/demo-20260104.gif)

## usage
before running `navrs` you must configure your credentials: in `$HOME/.config/navrs/config.toml` manage navrs configuration:

```toml
[server]
base = "https://my.subsonic.com"

[auth]
username = "your-username"
password = "your-password"

[player]
device = "navrs" # optional
preload = 5 # optional
```

then just running `navrs` will start the TUI and load your favorites

### keybinds
 * `q`: exit
 * `tab` or `1-5`: switch tab
 * `space`: play/pause
 * `n`: next song
 * `r`: restart song
 * `s`: on playing screen, shuffle all favorites
 * `up/down`: on lists, move cursor
 * `left/right`: move playhead
 * `shift/alt`: modifier for faster scrolling/skipping/...
 * `+`: append to queue
 * `=`: play next

 ## building
```sh
$ cargo build --release
```

## credits
thanks to all subsonic clients, they either suck or don't sync too well or are super annoying to compile and buggy

making my own, rolled this in one day, aha sorry flex but srsly please make better players  c':
