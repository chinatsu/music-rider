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

music-rider path/to/album
music-rider path/to/album/song.flac
music-rider -h # for various options

# or just..

cargo run -- path/to/album
```

## blog

### 2025-08-29

the communication stack for the exercise equipment has been extracted into a separate project: https://crates.io/crates/kondis

### 2025-08-28

the analyzer now lives in the music player, and works in a different way than before.
previously, it'd check out samples in real-time, and give its score.
now, upon loading the file, it goes through the whole song and gives its score for every sample.

then, the playback commences and can pick out the precomputed score for that sample.
this allows us to offset the pointer that picks out the sample, so that the score is from a future part of the song.

my bike takes a little time to apply the resistance, so with tuning the offset with the `-o` flag we can account for more or less latency.

### 2025-08-24

the analysis section of the program has been made into a trait, so that eventually we can have multiple possible analyzers!

i'm leaning towards a not-so-realtime calculation of amplitude, though.
as in, i'd like to compute the whole ride before the song starts playing.
then, at more regular intervals on the bike side, we can query the map and figure out what the bike level is supposed to be.

this should allow for a bit more adjustments as well, once we know the full waveform, we might wanna smooth out parts, delay it a little (e.g. set the level of what's _about_ to play, instead of what is played/just played). the bike takes a tiny bit of time to update its resistance, so decoupling playback and bike stuff even further is probably good.

### 2025-08-23

scaffolding for implementing other types of exercise equipment is in!
it's not beautiful, but hopefully it's an okay starting point for whoever wants to add their rowing machine or whatever.

also, ctrl+c behavior has been improved. now, when closing the program that way cleanly disconnects my bike.
previously, my bike would stay lit up until i either unplugged it or ran the program to completion

## known problems

- ~~ctrl+c/`SIGINT` does not cleanly disconnect the bike nor flush the audio stack nicely~~
    - you can always reconnect the bike by rerunning the program, and load a short audio track in.
      once the track has finished playing, a clean shutdown/disconnect is performed
    - ~~i know of ctrlc crates and all, but i just haven't yet figured out how to send those signals here and there~~
        - **update**: i've started sending shutdown signals to various parts of the program.
        - this change makes the audio player flush its output, and the bike to disconnect
        - but....
- ~~ctrl+c/`SIGINT` while the program is scanning for devices causes a panic~~
    - this isn't as dangerous as it seems;
        - the bluetooth stack stops scanning before the panic,
        - no audio stuff has started (nothing to flush), 
        - nor is there any connection to the device (nothing to disconnect)
    - this makes the program shut down slower though, because it's reliant on a bluetooth event happening before the shutdown signal is noticed
- the bluetooth connection doesn't always find the bike
    - tbh, just restart the program until it does
    - i suspect one of the cases for me is that my bluetooth dongle sets up two devices.
      when the "wrong" device discovers the bike, the other one doesn't see it or something
- ~~resistance doesn't really follow the music well...~~
    - `--no-read` helps a lot, since the read function blocks the thread for a while
    - ~~the analyzer only takes the sample it can read, so maybe if the thread gets blocked, the samples get out of sync~~
        - so maybe the analyzer should live in the music player after all
        - this way we could maybe have a bidirectional channel where the main thread can ask the music thread "hey, what's the average amplitude these days?"
        - then the music thread can send back an answer that should fall better in line with what the bike should be set to, without it being too time sensitive
    - it seems that blasting the bike with write packets makes it a little angry though
        - the bluetooth connection was dropped once out of the blue, and the bike refused reconnection for a while after
- ~~support for more bikes..~~
    - ~~i'm too dumb for `dyn` stuff man~~

also note, the analysis stuff isn't perfect yet.
i'm experimenting with various techniques, and i gotta kill my legs for a bit to figure out if i like one or the other.


## acknowledgements

these codebases have helped me a lot of the way!

- https://github.com/jetoneza/cycling_trainer
- https://github.com/BostonLeeK/iconsole-plus-client
