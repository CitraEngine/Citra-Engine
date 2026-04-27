# Ilama ISA

The Ilama ISA is the ISA that the C-Machine uses. It has some standard general processor opcodes, but the bulk of the optimization comes from a specialized opcode called GAME_CALL. Otherwise this VM is a limited but functional register based general purpose machine.

## Physical Registers

- r0 - r10:  general purpose integer registers
- flg (r11): flags register
- stk (r12): stack pointer
- pc (r13): program counter
- fp (r14): frame pointer
- obj (r15): object pointer (Points to the game object that called the C-Machine function) \[READ ONLY\]

## Abstraction Registers
- f0 - f15: registers r0 - r15 but represented as floating point data
  - `mov r0, 2   -> MOV  0 2`
  - `mov f0, 2.0 -> MOVF 0 2.0`

## Instruction format

```
32-bit fixed bit
2 configs

Short:
00000000 | 0000 | 0000 | 00000000 00000000

1. 8-bits:  op code
2. 4-bits:  destination reg
3. 4-bits:  source reg
4. 16-bits: extra

Long:
00000000 | 0000 | 0000 00000000 00000000

1. 8-bits:  op code
2. 4-bits:  destination reg
3. 20-bits: immediate pool index

```
The Ilama ISA is fixed length and the register length is also the same size as an instruction. This makes it impossible for functions that use immediates to work on their own. To fix this, the C-Machine provides a specialized memory space called the pool. The pool is a fixed length array that can store just over a million unique numbers that are exclusively used in place of immediates. Any instruction that uses an immediate will be of the long instruction config and index into the immediate pool with a 20-bit index.

## Instructions
This section describes every instruction that the C-Machine will recognize. Not all of them will be available through the Asimina Assembler (mostly because they are abstracted away to keep the "apparent" ISA simple). General rule of thumb is, the instruction is only directly available in Asimina Assembly if the abstraction is "None". If you want to use an instruction not directly available in Asimina Assembly, refer to the Abstraction column.

### Global Numonics
- REG = Register / any r0 - r15 register
- FREG = Any f0 - f15 register
- AREG = Any r0 - r15 or f0 - f15 register
- IDX = Immediate pool idx
- INT = Any integer number
- FLT = Any floating point number
- MEM = Any 32-bit memory address
- Guest code / function = any code written to run in the machine
- Host code / function = any code written as an intrinsic of the machine

### Short Format Instructions (OpCode <= 127)
| OpCode | Numonic | Description                                                                                      | Usage         | Abstraction    |
| :----: | :-----: | :----------------------------------------------------------------------------------------------: | :----------:  | :------------: |
| 0x00   | NOP     | Do nothing for one cycle.                                                                        | NOP           | None           |
| 0x01   | HALT    | Stop program execution until an interrupt happens.                                               | HALT          | None           |
| 0x02   | MOVR    | Set the value of a register with another register.                                               | MOVR REG REG  | MOV AREG AREG  |
| 0x03   | CFIT    | Convert floating point to integer with truncation.                                               | CFIT REG REG  | MOV REG FREG   |
| 0x04   | CIFT    | Convert integer to floating point.                                                               | CIFT REG REG  | MOV FREG REG   |
| 0x05   | PUSH    | Push the value of a register to the stack.                                                       | PUSH REG      | None           |
| 0x06   | POP     | Set the value of a register with the value at the top of the stack, then pop the value.          | POP REG       | None           |
| 0x07   | LDRR    | Set the value of a register with data in memory with address from register.                      | LDRR REG REG  | LDR AREG REG   |
| 0x08   | STRR    | Store the value of a register to memory with address from register.                              | STRR REG REG  | STR REG AREG   |
| 0x09   | BCMPR   | Bitwise compare the values of two registers.                                                     | BCMPR REG REG | BCMP AREG AREG |
| 0x0A   | ICMPR   | Integer compare the values of two registers.                                                     | ICMPR REG REG | CMP REG REG    |
| 0x0B   | FCMPR   | Floating point compare the values of two registers.                                              | FCMPR REG REG | CMP FREG FREG  |
| 0x0C   | JMPR    | Jump to a memory address in a register.                                                          | JMPR REG      | JMP REG        |
| 0x0D   | JZR     | Jump to a memory address in a register if zero.                                                  | JZR REG       | JZ REG         |
| 0x0E   | JNZR    | Jump to a memory address in a register if not zero.                                              | JNZR REG      | JNZ REG        |
| 0x0F   | JLR     | Jump to a memory address in a register if signed less than.                                      | JLR REG       | JL REG         |
| 0x10   | JLER    | Jump to a memory address in a register if signed less than or equal to.                          | JLER REG      | JLE REG        |
| 0x11   | JAR     | Jump to a memory address in a register if unsigned greater than.                                 | JAR REG       | JA REG         |
| 0x12   | JAER    | Jump to a memory address in a register if unsigned greater than or equal.                        | JAER REG      | JAE REG        |
| 0x13   | SFTR    | Shift the bits in a register a number of times as specified in another register.                 | SFTR REG REG  | SFT REG REG    |
| 0x14   | ROTR    | Rotate the bits in a register a number of times as specified in another register.                | ROTR REG REG  | ROT REG REG    |
| 0x15   | CLGR    | Jump to guest function pointed to by an address in a register and push current address to stack. | CLGR REG      | CALL REG       |
| 0x16   | CLHR    | Execute a host function pointed to by an index in a register.                                    | CLHR REG      | CALH REG       |
| 0x17   | RET     | Pop the top value on the stack and use it as a memory address to jump to.                        | RET           | None           |

### Long Format Instructions (OpCode >= 128)
| OpCode | Numonic | Description                                                                                            | Usage         | Abstraction    |
| :----: | :-----: | :----------------------------------------------------------------------------------------------------: | :-----------: | :------------: |
| 0x80   | MOVII   | Set the value of a register with integer immediate from pool.                                          | MOVII REG IDX | MOV REG INT    |
| 0x81   | MOVIF   | Set the value of a register with floating point immediate from pool.                                   | MOVIF REG IDX | MOV FREG FLT   |
| 0x82   | LDRI    | Set the value of a register with data in memory with address from pool.                                | LDRI REG IDX  | LDR AREG [MEM] |
| 0x83   | STRI    | Store the value of a register to memory with address from pool.                                        | STRI REG IDX  | STR [MEM] AREG |
| 0x84   | BCMPI   | Bitwise compare the values of a register and an intermediate from pool.                                | BCMPI REG IDX | BCMP REG INT   |
| 0x85   | ICMPI   | Integer compare the values of a register and an intermediate from pool.                                | ICMPI REG IDX | CMP REG INT    |
| 0x86   | FCMPI   | Floating point compare the values of a register and an intermediate from pool.                         | FCMPI REG IDX | CMP FREG FLT   |
| 0x87   | BCMPM   | Bitwise compare the values of a register and the value stored at a memory address.                     | BCMPM REG IDX | BCMP REG [MEM] |
| 0x88   | ICMPM   | Integer compare the values of a register and the value stored at a memory address.                     | ICMPM REG IDX | CMP REG [MEM]  |
| 0x89   | FCMPM   | Floating point compare the values of a register and the value stored at a memory address.              | FCMPM REG IDX | CMP FREG [MEM] |
| 0x8A   | JMPI    | Jump to a memory address in immediate pool.                                                            | JMPI IDX      | JMP INT        |
| 0x8B   | JZI     | Jump to a memory address in immediate pool if zero.                                                    | JZI IDX       | JZ INT         |
| 0x8C   | JNZI    | Jump to a memory address in immediate pool if not zero.                                                | JNZI IDX      | JNZ INT        |
| 0x8D   | JLI     | Jump to a memory address in immediate pool if signed less than.                                        | JLI IDX       | JL INT         |
| 0x8E   | JLEI    | Jump to a memory address in immediate pool if signed less than or equal to.                            | JLEI IDX      | JLE INT        |
| 0x8F   | JAI     | Jump to a memory address in immediate pool if unsigned greater than.                                   | JAI IDX       | JA INT         |
| 0x90   | JAEI    | Jump to a memory address in immediate pool if unsigned greater than or equal to.                       | JAEI IDX      | JAE INT        |
| 0x91   | JMPM    | Jump to a memory address in memory.                                                                    | JMPM IDX      | JMP [MEM]      |
| 0x92   | JZM     | Jump to a memory address in memory if zero.                                                            | JZM IDX       | JZ [MEM]       |
| 0x93   | JNZM    | Jump to a memory address in memory if not zero.                                                        | JNZM IDX      | JNZ [MEM]      |
| 0x94   | JLM     | Jump to a memory address in memory if signed less than.                                                | JLM IDX       | JL [MEM]       |
| 0x95   | JLEM    | Jump to a memory address in memory if signed less than or equal to.                                    | JLEM IDX      | JLE [MEM]      |
| 0x96   | JAM     | Jump to a memory address in memory if unsigned greater than.                                           | JAM IDX       | JA [MEM]       |
| 0x97   | JAEM    | Jump to a memory address in memory if unsigned greater than or equal to.                               | JAEM IDX      | JAE [MEM]      |
| 0x98   | SFTI    | Shift the bits in a register a number of times as specified by an immediate in the pool.               | SFTI REG IDX  | SFT REG INT    |
| 0x99   | SFTM    | Shift the bits in a register a number of times as specified by a value in memory.                      | SFTM REG IDX  | SFT REG [MEM]  |
| 0xA0   | ROTI    | Rotate the bits in a register a number of times as specified by an immediate in the pool.              | ROTI REG IDX  | ROT REG INT    |
| 0xA1   | ROTM    | Rotate the bits in a register a number of times as specified by a value in memory.                     | ROTM REG IDX  | ROT REG [MEM]  |
| 0xA2   | CLGI    | Jump to a guest function pointed to by an address in immediate pool and push current address to stack. | CLGI IDX      | CALL INT       |
| 0xA3   | CLGM    | Jump to a guest function pointed to by an address in memory and push current address to stack.         | CLGM IDX      | CALL [MEM]     |
| 0xA4   | CLHI    | Execute a host function pointed to by an index in immediate pool.                                      | CLHI IDX      | CALH INT       |
| 0xA5   | CLHM    | Execute a host function pointed to by an index in memory.                                              | CLHM IDX      | CALH [MEM]     |

## Host calling convention
This calling convention is for HOST FUNCTIONS ONLY, guest functions are all written by the game dev so they don't need a set in stone calling convention. Static calling convention, all registers EXCEPT r0 are protected and do not to be saved to the stack or memory (this is due to the fact that host functions do not even use the VM to run). r0 is arg 1, r1 is arg 2 and so forth. If 16 registers is not enough to pass all arguments they are pushed in order to the stack.
```asimina_asm
push <ARG17>
push <ARG18>
...
CALH <FUNCTION>
```
The return value of a host function will be stored to r0.
