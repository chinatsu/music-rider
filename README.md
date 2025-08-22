# music-rider

ride along to ~~the loudness~~ amplitudes of your favorite album

the code is a mess rn

the program connects to an iconsole+ bike (mine is a `iConsole+0028`), loads up an album or a song, and plays it back once a connection to the bike has been established.

at regular intervals (configurable with the `-f` flag) during playback, a fast-fourier transform is applied to the current sample buffer, and reduced to a single number representing the average amplitude across the whole spectrum.
this number is then transformed again to a suitable level for the bike.

it is possible to configure the level scale with the `-s` flag, values below 1.0 effectively reduces the maximum level (though, it scales the average amplitude number), values above 1.0 makes it so that the maximum level is reached more often.


## usage

```sh
cargo install --path .

bike path/to/album
bike path/to/album/song.flac
```

```
audiosurf irl or something

Usage: bike [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to the directory containing FLAC files

Options:
  -s, --scale <SCALE>          Scale factor for the maximum level (limits the maximum level) [default: 1]
  -n, --no-discovery           Disable bike discovery (enables playback without a bike, emits level changes to stdout)
  -f, --frequency <FREQUENCY>  Update frequency in seconds [default: 3]
  -h, --help                   Print help
  -V, --version                Print version
```


## acknowledgements

these codebases have helped me a lot of the way!

- https://github.com/jetoneza/cycling_trainer
- https://github.com/BostonLeeK/iconsole-plus-client
