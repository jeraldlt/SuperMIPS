# SuperMIPS
SuperMIPS is a MIPS32 based game console emulator, inspired by the Pico-8 project, and implemented with the purpose of making my Computer Architecture class more fun.





## Memory Map
SuperMIPS follows the standard MIPS memory map:

0x0000-0000 .. 0x003f-ffff : Reserved
0x0040-0000 .. 0x0fff-ffff : .text
0x1000-0000 .. 0x1000-ffff : Reserved
0x1001-0000 .. 0x7fff-ffff : .data (static data) / dynamic data\* / stack\*\*

\* dynamic data (data that is allocated at runtime) starts at the end of .data (which starts at 0x1001-0000) and grows "upward".
\*\* the stack pointer starts at 0x7fff-ffff, and the stack grows "downward"

## System Calls
System calls is largly how game code interacts with the SuperMIPS system. Like MARS, the contents of `$v0` to determine the purpose of the syscall.

|`$v0`|Description|Arguments|Returns|
====================================
|0x00|Exits the program|||
|0x01|Frame update|||
|0x02|Frame sync|||
|0x03|Prints a message to the system console|`$a0` contains the address of a null-terminated string to print||
|0x04|Prints an integer to the system console|`$a0` contains the 32-bit value|`$a1` contains how it should be interpreted (0=signed decimal; 1=unsigned decimal; 2=hexadecimal; 3=character (from bottom 8 bits))||
|0x05|Set window title|`$a0` contains the address of a null-terminated string to be the window title||
|0x06|Reserved for set framerate|||
|0x07|Reserved for set screen dimensions|||
|0x08|Seed the RNG with input|`$a0` contains a seed value||
|0x09|Seed the RNG with system time|||
|0x0a|Get a random u32 from the RNG||`$v0` contains the random number|
|0x0b|Get a ranged random u32 from the RNG|`$a0` contains the lower range extent (inclusive)|`$a1` contains the upper range extent (inclusive)|`$v0` contains the random number|


|0x10|Get buttons pressed this frame||`$v0` contains the button word|
|0x11|Get buttons un-pressed this frame||`$v0` contains the button word|
|0x12|Get buttons held this frame||`$v0` contains the button word|

|0x20|Fill the screen|`$a0` contains the color to fill the screen with||
|0x21|Draw a line|`$a0` contains the line color;`$a1` contains the start position;`$a2` contains the stop position||
|0x22|Fill a rectangle|`$a0` contains the color to fill the rectangle with; `$a1` contains the upper-left corner; `$a2` contains the lower-right corner||
|0x23|Fill a circle|`$a0` contains the color to fill the circle with; `$a1` contains the circle center position; `$a2` contains the circle radius||
|0x24|Blit a sprite|`$a0` contains the sprite address;`$a1` contains the width and height||

|0x30|Play a tone|`$a0` contains the tone to play in Hertz; `$a1` contains the number of frames to play the tone for||
|0x31|Play a melody|`$a0` contains the address to a melody; `$a1` contains the frames per beat||
|0x32|Loop a melody|`$a0` contains the address to a melody; `$a1` contains the frames per beat||
|0x33|Stop all audio|||
