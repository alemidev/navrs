# subtui
[![Actions Status](https://github.com/alemidev/subtui/actions/workflows/test.yml/badge.svg?branch=dev)](https://github.com/alemidev/subtui/actions/workflows/test.yml)
[![Actions Status](https://github.com/alemidev/subtui/actions/workflows/release.yml/badge.svg)](https://github.com/alemidev/subtui/actions/workflows/release.yml)
[![GitHub last commit](https://img.shields.io/github/last-commit/alemidev/subtui)](https://github.com/alemidev/subtui/commits/dev/)
> terminal music player for subsonic

![splash](https://cdn.alemi.dev/proj/subtui/screenshot-20260201.png)

## usage
before running `subtui` you must configure your credentials: in `$HOME/.config/subtui/config.toml` manage subtui configuration:

```toml
[server]
base = "https://my.subsonic.com"

[auth]
username = "your-username"
password = "your-password"
```

then just running `subtui` will start the TUI and load your favorites

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
