//! Converting any arbitrary video into a sequence of frames.

use std::path::Path;

use ffmpeg_next::format::{self};
use ffmpeg_next::software::scaling;
use ffmpeg_next::util::frame::video::Video as VideoFrame;
use ffmpeg_next::{decoder, media, Packet};
use image::{Rgb, RgbImage};

/// A video decoder.
pub struct VideoDecoder {
   pub width: u32,
   pub height: u32,
   pub framerate: (u32, u32), // numerator, denominator
   pub packet_count: usize,

   ctx: format::context::Input,
   decoder: decoder::Video,
   scaler: scaling::Context,
   video_stream_index: usize,

   current_packet: Option<Packet>,
   decoded_frame: VideoFrame,
   scaled_frame: VideoFrame,
   frame_buffer: RgbImage,
}

pub enum VideoDecodeEvent<'dec> {
   StartPacket,
   Frame(&'dec RgbImage),
}

pub struct FfmpegOptions {
   /// If `Some`, the video will be scaled down to the provided size.
   pub resize: Option<(u32, u32)>,
}

impl VideoDecoder {
   /// Loads the video at the provided path.
   pub fn new(path: &Path, options: FfmpegOptions) -> anyhow::Result<Self> {
      let mut ctx = format::input(&path)?;
      let video_stream = ctx
         .streams()
         .best(media::Type::Video)
         .ok_or_else(|| anyhow::anyhow!("the provided input file does not have a video stream"))?;
      let video_stream_index = video_stream.index();
      let framerate = video_stream.rate();

      let decoder = video_stream.codec().decoder().video()?;
      let (width, height) = options.resize.unwrap_or((decoder.width(), decoder.height()));
      let packet_count = ctx.packets().count();
      ctx.seek(0, ..)?;

      let scaler = scaling::Context::get(
         decoder.format(),
         decoder.width(),
         decoder.height(),
         format::Pixel::RGB24,
         width,
         height,
         scaling::Flags::BILINEAR,
      )?;

      Ok(Self {
         width,
         height,
         framerate: (framerate.numerator() as u32, framerate.denominator() as u32),
         packet_count,

         ctx,
         decoder,
         scaler,
         video_stream_index,

         current_packet: None,
         decoded_frame: VideoFrame::empty(),
         scaled_frame: VideoFrame::empty(),
         frame_buffer: RgbImage::from_pixel(width, height, Rgb([0, 0, 0])),
      })
   }

   pub fn next(&mut self) -> anyhow::Result<Option<VideoDecodeEvent>> {
      if self.current_packet.is_none() {
         self.current_packet = self
            .ctx
            .packets()
            .find(|(stream, _)| stream.index() == self.video_stream_index)
            .map(|(_, packet)| packet);
         if let Some(packet) = &self.current_packet {
            self.decoder.send_packet(packet)?;
            return Ok(Some(VideoDecodeEvent::StartPacket));
         } else {
            return Ok(None);
         }
      }
      if let Some(_) = &self.current_packet {
         // Errors happen? Whoops, not my fault.
         if self.decoder.receive_frame(&mut self.decoded_frame).is_ok() {
            self.scaler.run(&self.decoded_frame, &mut self.scaled_frame)?;
            for y in 0..self.scaled_frame.height() {
               for x in 0..self.scaled_frame.width() {
                  let yy = y as usize * self.scaled_frame.stride(0);
                  let data = self.scaled_frame.data(0);
                  let index = yy + x as usize * 3;
                  self.frame_buffer[(x, y)] = Rgb([data[index], data[index + 1], data[index + 2]]);
               }
            }
            Ok(Some(VideoDecodeEvent::Frame(&self.frame_buffer)))
         } else {
            self.current_packet = None;
            self.next()
         }
      } else {
         self.decoder.send_eof()?;
         Ok(None)
      }
   }
}
