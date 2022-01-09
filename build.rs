use szmelc_player_codegen::generate_numberlut;

fn main() -> Result<(), Box<dyn std::error::Error>> {
   std::fs::write("src/generated/numberlut.h", generate_numberlut())?;

   Ok(())
}
