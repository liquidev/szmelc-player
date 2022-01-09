// Wrapper for miniaudio.

#ifndef SZMELC_AUDIO_H
#define SZMELC_AUDIO_H

#include <stdlib.h>

void audio_play(size_t data_len, const char data[], unsigned sample_rate);

void audio_stop();

#endif
