# music-rider

ride along to ~~the loudness~~ bass of your favorite album

the code is a mess rn, but the program

- connects to an iconsole+ bike
- loads up an album
- plays back songs in sequence
    - during playback of songs, it collects a sample into a few low frequency bins using fft, averages the bins to a number and scales it to a suitable(?) level on the bike

## acknowledgements

these codebases have helped me a lot of the way!

- https://github.com/jetoneza/cycling_trainer
- https://github.com/BostonLeeK/iconsole-plus-client
