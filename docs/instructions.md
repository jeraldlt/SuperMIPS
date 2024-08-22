# Instructions
This is a list of MIPS 32 bit instructions and their current support in the SuperMIPS emulator. If an instruction is not listed there is no current intention to support it.

|:-:|:-:|-
|**instruction**|**implemented**|**unit tested**|
|-:|:-:|:-:
| add     | no  | no  |
| addi    | yes | no  |
| addiu   | yes | no  |
| addu    | yes | no  |
| and     | no  | no  |
| andi    | yes | no  |
| b       | no  | no  |
| bal     | no  | no  |
| beq     | yes | no  |
| beql    | no  | no  |
| bgez    | no  | no  |
| bgezal  | no  | no  |
| bgezall | no  | no  |
| bgezl   | no  | no  |
| bgtz    | no  | no  |
| bgtzl   | no  | no  |
| blez    | no  | no  |
| blezl   | no  | no  |
| bltz    | no  | no  |
| bltzal  | no  | no  |
| bltzall | no  | no  |
| bltzl   | no  | no  |
| bne     | yes | no  |
| bnel    | no  | no  |
| break   | no  | no  |
| div     | no  | no  |
| divu    | no  | no  |
| j       | yes | no  |
| jal     | no  | no  |
| jalr    | no  | no  |
| jr      | no  | no  |
| lb      | no  | no  |
| lbu     | no  | no  |
| lh      | no  | no  |
| lhu     | no  | no  |
| ll      | no  | no  |
| lui     | yes | no  |
| lw      | no  | no  |
| madd    | no  | no  |
| maddu   | no  | no  |
| mfhi    | no  | no  |
| mflo    | no  | no  |
| msub    | no  | no  |
| msubu   | no  | no  |
| mthi    | no  | no  |
| mtlo    | no  | no  |
| mul     | no  | no  |
| mult    | yes | no  |
| nop     | no  | no  |
| nor     | no  | no  |
| or      | yes | no  |
| ori     | yes | no  |
| sb      | no  | no  |
| sh      | no  | no  |
| sll     | yes | no  |
| sllv    | no  | no  |
| slt     | yes | no  |
| slti    | no  | no  |
| sltiu   | no  | no  |
| sra     | no  | no  |
| srav    | no  | no  |
| sub     | no  | no  |
| subu    | no  | no  |
| sw      | no  | no  |
| swl     | no  | no  |
| swr     | no  | no  |
| syscall | no  | no  |
| xor     | yes | no  |
| xori    | yes | no  |
|-

## Pseudoinstructions

**li**

*If the immediate value is larger than 2^16:*

`li $t0, 0x1234ABCD`

gets expanded to

```
lui $at, 1234
ori $t0, $at, 0xABCD
```

*If the immediate value is less than or equal to 2^16:*

`li $t0, 0x1234`

gets expanded to

`addiu $t0, $zero, 0x1234`

**lw**

*Not yet implemented!*

**move**

`move $t0, $t1`

gets expanded to

`addu $t0, $zero, $t1`

**la**

*assuming label resolves to the address 0x1001ABCD*

`la $t0, label`

gets expanded to 

```
lui $at, 0x1001
ori $t0, $at, 0xABCD
```

