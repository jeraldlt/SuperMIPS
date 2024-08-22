# Memory Map

SuperMIPS follows the standard MIPS memory map:

0x00000000 .. 0x003fffff : Reserved
0x00400000 .. 0x0fffffff : .text
0x10000000 .. 0x1000ffff : Reserved
0x10010000 .. 0x7fffffff : .data (static data) / dynamic data\* / stack\*\*

\* dynamic data (data that is allocated at runtime) starts at the end of .data (which starts at 0x10010000) and grows "upward".
\*\* the stack pointer starts at 0x7fffffff, and the stack grows "downward"

