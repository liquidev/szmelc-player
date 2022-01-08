# Szmelc Player

**Szmelc Player** is a program that converts any video you give it to a standalone executable that
plays the video in the terminal. It uses FFmpeg to decode the video, so you can feed it any format
you'd like - it's pretty much guaranteed to support it.

The end user does not need FFmpeg because the video frames are embedded as raw pixel data into the
final executable.

Szmelc Player requires a C98-compliant C compiler to function. Only Unix-based systems are supported
<del>because they're based and Windows is cringe</del> because of poor ANSI escape sequence support
on Windows. *Yes, I know Windows Terminal supports them, but there are other problems preventing*
*Szmelc Player from running on Windows such as the lack of `usleep`.*

## What does "szmelc" mean?

_Szmelc_ in Polish means _junk_, _trash_. It's pronounced like _shmeltz_. The program is called that
because it's an absolutely horrible way to watch videos.

## Alright shut up, how do I use it?

Well first you need to compile it:
```sh
$ cargo build --release
# To install the executable in ~/.cargo/bin:
$ cargo install --path .
```
Then the usage is as follows:
```sh
$ szmelc-player <input file> <output file>
# It is strongly recommended to scale the video down as the "pixels" output by szmelc-player are
# quite large (each pixel is 1x0.5 characters in your terminal).
# Thus, to make the video fill up an 80x24 terminal window, you need to double the height.
$ szmelc-player input.webm output --resize 80x48
```
It's possible to only generate the C code, without compiling, by using the `--generate-c` flag.
```
$ szmelc-player input.webm output.c --resize 20x12 --generate-c
$ head output.c -c1000
// “Dr. Szmelc is back, baby.”
// Generated by szmelc-player version 0.1.0

#define VIDEO_WIDTH (80)
#define VIDEO_HEIGHT (48)
#define SLEEP_INTERVAL (33366)
const unsigned char video_data[] = {103,103,103,103,103,103,103,103,103,103,103,103,104,104,104,107,107,107,109,109,109,110,110,110,111,111,111,112,112,112,113,113,113,113,113,113,113,115,120,115,116,122,115,116,122,115,116,122,115,116,122,115,116,122,115,116,122,115,116,122,116,117,123,116,117,123,116,117,123,116,117,123,116,117,123,116,117,123,116,117,123,116,117,123,117,117,122,117,117,122,126,118,115,127,119,116,131,119,113,131,119,113,131,119,113,131,119,113,131,119,113,131,119,113,131,119,113,131,119,113,131,119,113,131,119,113,130,118,112,130,118,112,130,118,112,130,118,112,130,118,112,130,118,112,130,118,112,129,117,111,129,117,111,129,117,111,127,116,110,127,116,110,127,116,110,126,115,109,126,115,109,125,113,108,125,113,108,125,113,108,124,112,107,123,111,105,123,111,105,122,110,104,120,109,103,120,109,103,119,108,10
```
By default, Szmelc Player will compile the C code using the default system compiler `cc` (usually symlinked to `gcc`). It is possible to specify a different compiler (eg. `clang`) by using the `$SZMELC_CC` or `$CC` environment variables (the former will take priority).

`gcc` and `clang` are quite resource hungry, but unfortunately it's not possible to use `tcc` quite yet, as it fails to compile [miniaudio](https://miniaud.io/) (which Szmelc Player uses for audio output). At some point a better solution involving compiling miniaudio separately might get added.
