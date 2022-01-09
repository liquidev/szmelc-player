#include "audio.h"

#include <stdlib.h>

#define MINIAUDIO_NO_ENCODING
#define MINIAUDIO_NO_MP3
#define MINIAUDIO_NO_WAV
#define MINIAUDIO_NO_GENERATION

#define MINIAUDIO_IMPLEMENTATION
#include "vendor/miniaudio.h"

struct audio_context {
  ma_decoder decoder;
};

static void audio_callback(ma_device *device, void *output, const void *input,
                           uint32_t frame_count) {
  struct audio_context *actx = device->pUserData;
  ma_decoder_read_pcm_frames(&actx->decoder, output, frame_count, NULL);

  (void)input;
}

static struct audio_context actx = {0};
static ma_device adevice;

void audio_play(size_t data_len, const char data[], unsigned sample_rate) {
  ma_decoder_config decoder_config =
      ma_decoder_config_init(ma_format_s16, 2, sample_rate);
  ma_decoder_init_memory((void *)data, data_len, &decoder_config,
                         &actx.decoder);

  ma_device_config config = ma_device_config_init(ma_device_type_playback);
  config.playback.format = ma_format_s16;
  config.playback.channels = 2;
  config.sampleRate = sample_rate;
  config.dataCallback = audio_callback;
  config.pUserData = &actx;

  if (ma_device_init(NULL, &config, &adevice) != MA_SUCCESS) {
    fprintf(stderr, "audio error: failed to initialize device\n");
    exit(1);
  }

  ma_device_start(&adevice);
}

void audio_stop() { ma_device_uninit(&adevice); }
