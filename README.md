# music-rider

ride along to ~~the loudness~~ amplitudes of your favorite album

https://github.com/user-attachments/assets/effcd5a9-989f-49a0-a52a-80dea591ef85

the code is a mess rn

the program connects to an iconsole+ bike (mine is a `iConsole+0028`), loads up an album or a song, and plays it back once a connection to the bike has been established.

during playback, a fast-fourier transform is applied to the current sample buffer, and reduced to a single number representing the average amplitude across the whole spectrum.
this number is then transformed again to a suitable level for the bike.

it is possible to configure the level scale with the `-s` flag, values below 1.0 effectively reduces the maximum level (though, it scales the average amplitude number), values above 1.0 makes it so that the maximum level is reached more often.


## usage

```sh
cargo install --path .

bike path/to/album
bike path/to/album/song.flac
bike -h # for various options
```

## known problems

- ctrl+c/`SIGINT` does not cleanly disconnect the bike nor flush the audio stack nicely
    - you can always reconnect the bike by rerunning the program, and load a short audio track in.
      once the track has finished playing, a clean shutdown/disconnect is performed
    - i know of ctrlc crates and all, but i just haven't yet figured out how to send those signals here and there
- the bluetooth connection doesn't always find the bike
    - tbh, just restart the program until it does
    - i suspect one of the cases for me is that my bluetooth dongle sets up two devices.
      when the "wrong" device discovers the bike, the other one doesn't see it or something
- resistance doesn't really follow the music well...
    - `--no-read` helps a lot, since the read function blocks the thread for a while
    - the analyzer only takes the sample it can read, so maybe if the thread gets blocked, the samples get out of sync
        - so maybe the analyzer should live in the music player after all
        - this way we could maybe have a bidirectional channel where the main thread can ask the music thread "hey, what's the average amplitude these days?"
        - then the music thread can send back an answer that should fall better in line with what the bike should be set to, without it being too time sensitive
    - it seems that blasting the bike with write packets makes it a little angry though
        - the bluetooth connection was dropped once out of the blue, and the bike refused reconnection for a while after
- support for more bikes..
    - i'm too dumb for `dyn` stuff man

also note, the analysis stuff isn't perfect yet.
i'm experimenting with various techniques, and i gotta kill my legs for a bit to figure out if i like one or the other.


## acknowledgements

these codebases have helped me a lot of the way!

- https://github.com/jetoneza/cycling_trainer
- https://github.com/BostonLeeK/iconsole-plus-client
