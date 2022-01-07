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
