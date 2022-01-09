#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#ifdef SZMELC_BUILD
#include "{SZMELC_AUDIO_H}"
#include "{SZMELC_NUMBERLUT_H}"
#else

// The following definitions are provided in case the code is _not_ being built
// by Szmelc Player, so that code analysis tools such as clangd don't get
// confused by the lack of symbols.

#include "audio.h"
#include "generated/numberlut.h"

#define VIDEO_WIDTH 80
#define VIDEO_HEIGHT 48
#define SLEEP_INTERVAL 16666
static const char video_data[] = {};
#define VIDEO_FRAME_COUNT 0

#define AUDIO_SAMPLE_RATE 48000
static const char audio_data[] = {};

#endif

/* Video runtime */

#define TEXT_BUFFER_SIZE 1024

char stdout_buffer[TEXT_BUFFER_SIZE + 1] = {0};
size_t stdout_buffer_len = 0;

static inline void video_flush() {
  fwrite(stdout_buffer, 1, stdout_buffer_len, stdout);
  fflush(stdout);
  stdout_buffer_len = 0;
}

static inline void video_rawprint(size_t len, const char *text) {
  if (stdout_buffer_len + len > TEXT_BUFFER_SIZE) {
    video_flush();
  }
  for (; *text != '\0'; ++text) {
    stdout_buffer[stdout_buffer_len] = *text;
    ++stdout_buffer_len;
  }
}

#define video_print(text) video_rawprint(strlen(text), text);

static inline void video_print_byte(unsigned char b) {
  video_rawprint(byte_to_decimal[b].len, byte_to_decimal[b].str);
}

static inline void video_setfg(unsigned char r, unsigned char g,
                               unsigned char b) {
  video_print("\e[38;2;");
  video_print_byte(r);
  video_print(";");
  video_print_byte(g);
  video_print(";");
  video_print_byte(b);
  video_print("m");
}

static inline void video_setbg(unsigned char r, unsigned char g,
                               unsigned char b) {
  video_print("\e[48;2;");
  video_print_byte(r);
  video_print(";");
  video_print_byte(g);
  video_print(";");
  video_print_byte(b);
  video_print("m");
}

/* Main */

int main(void) {
  audio_play(sizeof audio_data, audio_data, AUDIO_SAMPLE_RATE);

  video_print("\e[2J\e[0;0H");
  video_flush();
  for (size_t i = 0; i < VIDEO_FRAME_COUNT; ++i) {
    video_print("\e[0;0H");
    for (unsigned y = 0; y < VIDEO_HEIGHT; y += 2) {
      for (unsigned x = 0; x < VIDEO_WIDTH; ++x) {
        size_t top_index =
            (i * VIDEO_WIDTH * VIDEO_HEIGHT + y * VIDEO_WIDTH + x) * 3;
        size_t bottom_index =
            (i * VIDEO_WIDTH * VIDEO_HEIGHT + (y + 1) * VIDEO_WIDTH + x) * 3;
        unsigned char top_r = video_data[top_index],
                      top_g = video_data[top_index + 1],
                      top_b = video_data[top_index + 2],
                      bottom_r = video_data[bottom_index],
                      bottom_g = video_data[bottom_index + 1],
                      bottom_b = video_data[bottom_index + 2];
        video_setbg(top_r, top_g, top_b);
        video_setfg(bottom_r, bottom_g, bottom_b);
        video_print("▄");
      }
      video_print("\e[0m\n");
    }
    video_flush();
    usleep(SLEEP_INTERVAL);
  }

  audio_stop();
}
