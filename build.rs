use std::path::Path;

use szmelc_player_codegen::generate_numberlut;

fn main() -> Result<(), Box<dyn std::error::Error>> {
   println!("cargo:rerun-if-changed=build.rs");

   std::fs::write("src/c/generated/numberlut.h", generate_numberlut())?;

   cc::Build::new()
      .file("src/c/audio.c")
      .out_dir(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/c/generated/libs"))
      .compile("audio");

   Ok(())
}
