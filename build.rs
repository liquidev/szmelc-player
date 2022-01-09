use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
   println!("cargo:rerun-if-changed=scripts/generate-number-lut.py");
   let status = Command::new("python").arg("scripts/generate-number-lut.py").status()?;
   assert!(status.success(), "failed to run generate-number-lut script");

   Ok(())
}
