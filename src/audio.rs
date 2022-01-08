//! Audio transcoding (whatever format you input into FLAC).

use std::path::Path;

use ffmpeg_next::format::{self, sample, Sample};
use ffmpeg_next::frame::Audio;
use ffmpeg_next::software::resampling;
use ffmpeg_next::{decoder, encoder, media, ChannelLayout, Packet};

/// A transcoder for audio.
pub struct AudioTranscoder {
   pub sample_rate: u32,
   pub packet_count: usize,

   input_ctx: format::context::Input,
   output_ctx: format::context::Output,
   input_stream_index: usize,
   output_stream_index: usize,
   decoder: decoder::Audio,
   encoder: encoder::Audio,
   resampler: resampling::Context,

   decoded: Audio,
   resampled: Audio,
   encoded: Packet,
}

impl AudioTranscoder {
   /// Reads the audio information and returns a transcoder instance.
   pub fn new(input_path: &Path, output_path: &Path) -> anyhow::Result<Self> {
      /* Set up input */

      let mut input_ctx = format::input(&input_path)?;
      let input_stream = input_ctx
         .streams()
         .best(media::Type::Audio)
         .ok_or_else(|| anyhow::anyhow!("the provided input file does not have an audio stream"))?;
      let input_stream_index = input_stream.index();

      let mut decoder = input_stream.codec().decoder().audio()?;
      decoder.set_parameters(input_stream.parameters())?;

      let sample_rate = decoder.rate();
      let resampler = resampling::Context::get(
         decoder.format(),
         decoder.channel_layout(),
         sample_rate,
         Sample::I16(sample::Type::Packed),
         ChannelLayout::STEREO,
         sample_rate,
      )?;

      let packet_count = input_ctx.packets().count();
      input_ctx.seek(0, ..)?;

      /* Set up output */

      let mut output_ctx = format::output(&output_path)?;
      let codec = encoder::find(output_ctx.format().codec(&output_path, media::Type::Audio))
         .ok_or_else(|| {
            anyhow::anyhow!("your build of FFmpeg does not seem to have FLAC encoding support")
         })?;

      let mut output_stream = output_ctx.add_stream(codec)?;
      let output_stream_index = output_stream.index();
      let mut encoder = output_stream.codec().encoder().audio()?;
      encoder.set_format(Sample::I16(sample::Type::Packed));
      encoder.set_channel_layout(ChannelLayout::STEREO);
      encoder.set_rate(sample_rate as i32); // uh oh, signed integers!!
      encoder.set_channels(encoder.channel_layout().channels());

      encoder.set_time_base((1, decoder.rate() as i32));
      output_stream.set_time_base((1, decoder.rate() as i32));

      let encoder = encoder.open_as(codec)?;
      output_stream.set_parameters(&encoder);

      Ok(Self {
         sample_rate,
         packet_count,

         input_ctx,
         output_ctx,
         input_stream_index,
         output_stream_index,
         decoder,
         encoder,
         resampler,
         decoded: Audio::empty(),
         resampled: Audio::empty(),
         encoded: Packet::empty(),
      })
   }

   /// Transcodes the audio to FLAC and saves it under the given path.
   ///
   /// The `progress` callback is called on each packet in the input stream.
   pub fn transcode_to_flac(mut self, mut progress: impl FnMut()) -> anyhow::Result<()> {
      self.output_ctx.set_metadata(self.input_ctx.metadata().to_owned());
      self.output_ctx.write_header()?;

      let mut encode_resampled_frames = |encoder: &mut encoder::Audio| -> anyhow::Result<()> {
         while encoder.receive_packet(&mut self.encoded).is_ok() {
            self.encoded.set_stream(self.output_stream_index);
            self.encoded.write_interleaved(&mut self.output_ctx)?;
         }
         Ok(())
      };

      let mut resample_decoded_frames = |decoder: &mut decoder::Audio| -> anyhow::Result<()> {
         while decoder.receive_frame(&mut self.decoded).is_ok() {
            let buffer_size = self.resampler.get_out_samples(self.decoded.samples() as u32)?;
            if (self.resampled.samples() as u32) < buffer_size {
               self.resampled = Audio::new(
                  format::Sample::I16(sample::Type::Packed),
                  buffer_size as usize,
                  ChannelLayout::STEREO,
               );
            }
            self.resampler.run(&self.decoded, &mut self.resampled)?;
            self.resampled.set_pts(self.decoded.timestamp());
            self.encoder.send_frame(&self.resampled)?;
            encode_resampled_frames(&mut self.encoder)?;
         }
         Ok(())
      };

      for (stream, packet) in self.input_ctx.packets() {
         progress();
         if stream.index() == self.input_stream_index {
            self.decoder.send_packet(&packet)?;
            resample_decoded_frames(&mut self.decoder)?;
         }
      }
      self.decoder.send_eof()?;
      self.decoder.flush();

      while let Ok(delay) = self.resampler.flush(&mut self.resampled) {
         self.encoder.send_frame(&self.resampled)?;
         encode_resampled_frames(&mut self.encoder)?;
         // self.resampled = Audio::empty();
         if let None = delay {
            break;
         }
      }
      self.encoder.send_eof()?;

      self.output_ctx.write_trailer()?;

      Ok(())
   }
}
