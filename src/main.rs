use std::path::PathBuf;
use std::str::FromStr;

use code::Generator;
use image::EncodableLayout;
use pbr::ProgressBar;
use structopt::StructOpt;
use video::{FfmpegOptions, VideoDecoder};

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
   generator.define("VIDEO_WIDTH", &video.width.to_string())?;
   generator.define("VIDEO_HEIGHT", &video.height.to_string())?;

   let sleep_interval = {
      let (num, den) = video.framerate;
      den as f64 / num as f64 * 1_000_000.0
   } as u64;
   generator.define("SLEEP_INTERVAL", &sleep_interval.to_string())?;

   progress::task("Transcoding frames");
   let mut progress_bar = ProgressBar::new(video.packet_count as u64);
   let mut frame_data = generator.const_byte_array("video_data")?;
   let mut frame_count: usize = 0;
   while let Some(event) = video.next()? {
      match event {
         video::VideoDecodeEvent::StartPacket => {
            progress_bar.inc();
         }
         video::VideoDecodeEvent::Frame(image) => {
            for &byte in image.as_bytes() {
               frame_data.byte(byte)?;
            }
            frame_count += 1;
         }
      }
   }
   progress_bar.finish();
   let mut generator = frame_data.finish()?;
   generator.define("VIDEO_FRAME_COUNT", &frame_count.to_string())?;
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
