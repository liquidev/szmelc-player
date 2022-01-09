# This script generates the lookup table used for quickly converting from bytes to decimal numbers.

code = """
#include <stdlib.h>

static const struct {
   size_t len;
   const char *str;
} byte_to_decimal[] = {
"""
for i in range(0, 256):
   length = len(str(i))
   code += f'{{{length},"{i}"}},'

code += """
};
"""

with open("src/generated/numberlut.h", 'w') as f:
   f.write(code)
