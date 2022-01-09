//! Build-time code generation facilities for Szmelc Player. Used during compilation to generate
//! various header files.

use std::fmt::Write;

/// Generates the `numberlut.h` file.
pub fn generate_numberlut() -> String {
   let mut buffer = String::from(
      r#"
   static const struct {
      size_t len;
      const char *str;
   } byte_to_decimal[] = {
   "#,
   );

   for i in 0..=255 {
      let _ = write!(buffer, r#"{{{}, "{}"}},"#, i, i);
   }

   buffer.push_str("};");
   buffer
}
