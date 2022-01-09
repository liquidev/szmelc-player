use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use audio::AudioTranscoder;
use code::Generator;
use image::EncodableLayout;
use pbr::ProgressBar;
use structopt::StructOpt;
use tempfile::NamedTempFile;
use video::{FfmpegOptions, VideoDecoder};

mod audio;
mod code;
mod progress;
mod video;

#[derive(StructOpt)]
#[structopt(name = "szmelc-player")]
struct Args {
   /// The input video file.
   input_file: PathBuf,

   /// The output executable file.
   output_file: PathBuf,

   /// Generates C code and saves it to the given path without compiling it.
   #[structopt(long)]
   generate_c: bool,

   /// Defines the size of a frame.
   ///
   /// Setting this is strongly recommended unless you know what you're doing!
   #[structopt(long)]
   resize: Option<FrameSize>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
   let args = Args::from_args();
   ffmpeg_next::init()?;
   ffmpeg_next::format::register_all();

   let c_file = tempfile::Builder::new().prefix("szmelc-player").suffix(".c").tempfile()?;
   let mut generator = Generator::new(&c_file);
   generator.prelude()?;

   progress::task("Reading video file");
   let mut video = VideoDecoder::new(
      &args.input_file,
      FfmpegOptions {
         resize: args.resize.map(|FrameSize(width, height)| (width, height)),
      },
   )?;
   generator.define_value("VIDEO_WIDTH", &video.width.to_string())?;
   generator.define_value("VIDEO_HEIGHT", &video.height.to_string())?;

   let sleep_interval = {
      let (num, den) = video.framerate;
      den as f64 / num as f64 * 1_000_000.0
   } as u64;
   generator.define_value("SLEEP_INTERVAL", &sleep_interval.to_string())?;

   progress::task("Transcoding video");
   let mut progress_bar = ProgressBar::new(video.packet_count as u64);
   progress_bar.set_max_refresh_rate(Some(Duration::from_millis(100)));
   let mut video_data = generator.const_byte_array("video_data")?;
   let mut frame_count: usize = 0;
   while let Some(event) = video.next()? {
      match event {
         video::VideoDecodeEvent::StartPacket => {
            progress_bar.inc();
         }
         video::VideoDecodeEvent::Frame(image) => {
            for &byte in image.as_bytes() {
               video_data.byte(byte)?;
            }
            frame_count += 1;
         }
      }
   }
   progress_bar.finish();
   let mut generator = video_data.finish()?;
   generator.define_value("VIDEO_FRAME_COUNT", &frame_count.to_string())?;

   progress::task("Reading audio file");
   let flac_file =
      tempfile::Builder::new().prefix("szmelc-audio").suffix(".flac").tempfile()?.into_temp_path();
   let transcoder = AudioTranscoder::new(&args.input_file, &flac_file)?;
   generator.define_value("AUDIO_SAMPLE_RATE", &transcoder.sample_rate.to_string())?;

   progress::task("Transcoding audio");
   let mut progress_bar = ProgressBar::new(transcoder.packet_count as u64);
   progress_bar.set_max_refresh_rate(Some(Duration::from_millis(100)));
   transcoder.transcode_to_flac(|| {
      progress_bar.inc();
   })?;
   progress_bar.finish();

   progress::task("Embedding audio into executable");
   let mut audio_data = generator.const_byte_array("audio_data")?;
   let mut flac_file = File::open(&flac_file)?;
   let mut progress_bar = ProgressBar::new(file_size(&mut flac_file)?);
   progress_bar.set_max_refresh_rate(Some(Duration::from_millis(100)));
   for byte in flac_file.bytes() {
      progress_bar.inc();
      audio_data.byte(byte?)?;
   }

   progress::task("Finishing C code generation");
   let numberlut = save_header(include_bytes!("generated/numberlut.h"))?;
   let miniaudio = save_header(include_bytes!("vendor/miniaudio.h"))?;

   let mut generator = audio_data.finish()?;
   generator.define("SZMELC_BUILD")?;
   // This is a horrible way to define strings and I acknowledge that fully.
   // However at this moment in time I am too lazy to do this properly, so forgive my sins and
   // send a PR fixing this.
   generator.define_value("SZMELC_MINIAUDIO_H", &format!("{:?}", miniaudio.path()))?;
   generator.define_value("SZMELC_NUMBERLUT_H", &format!("{:?}", numberlut.path()))?;
   generator.main()?;

   if !args.generate_c {
      progress::task("Compiling executable");
      let compiler =
         std::env::var("SZMELC_CC").or_else(|_| std::env::var("CC")).unwrap_or("cc".to_owned());
      println!("Using C compiler: {}", compiler);
      code::compile_c(compiler, c_file.path(), &args.output_file)?;
   } else {
      std::fs::copy(c_file.path(), &args.output_file)?;
   }

   Ok(())
}

struct FrameSize(u32, u32);

impl FromStr for FrameSize {
   type Err = anyhow::Error;

   fn from_str(s: &str) -> Result<Self, Self::Err> {
      let mut split = s.split('x');
      let width = split.next().ok_or_else(|| anyhow::anyhow!("missing frame width"))?.parse()?;
      let height = split.next().ok_or_else(|| anyhow::anyhow!("missing frame height"))?.parse()?;
      Ok(Self(width, height))
   }
}

/// Returns the size of a file, in bytes.
fn file_size(file: &mut File) -> anyhow::Result<u64> {
   let size = file.seek(SeekFrom::End(0))?;
   file.seek(SeekFrom::Start(0))?;
   Ok(size)
}

fn save_header(data: &[u8]) -> anyhow::Result<NamedTempFile> {
   let mut file = tempfile::Builder::new().prefix("szmelc-runtime").suffix(".h").tempfile()?;
   file.write_all(data)?;
   Ok(file)
}
