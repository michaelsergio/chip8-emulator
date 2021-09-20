// Opcode table
// 35 ops 2 bytes long big endian

// NNN: address
//  NN: 8-bit K
//   N: 4 bit K
// X,Y: 4 bit register
//  PC: program counter
//   I: 16 bit register for memory address

// 00E0 CLS
// 00EE RET
// 1NNN JMP  NNN
// 2NNN CALL NNN *(0xNNN)()
// 3XNN SE skip if VX == NN (then JMP)
// 4XNN SNE skip if VX != NN (then JMP)
// 5XY0 SE skip if X == Y
// 6XNN LD VX to NN
// 7XNN ADD NN to VX
// 8XY0 LD VX to VY
// 8XY1 OR  VX |= VY
// 8XY2 AND VX &= VY
// 8XY3 XOR VX ^= VY
// 8XY4 ADD VX += VY  VF Carry
// 8XY5 SUB VX -= VY  VF Borrow
// 8XY6 SHR VX >>=1  lsb in VF
// 8XY7 SUBN VX=VY-VX  VF borrow
// 8XYE SHL VX<<=1    msb in VF
// 9XY0 SNE skip if vx != vy
// ANNN LD  I=NNN
// BNNN JMP to NNN+V0
// CXNN RND VX=rnd(0, 255) & NN
// DXYN DRW draw at coord(vx,vy, width=8, height=n) VF=pixels flipped
// EX9E SKP skip if key() == Vx
// EXA1 SKNP skip if key() !=Vy
// FX07 LD Vx=delay timer  
// FX0A LD VX=key
// FX15 LD delay=VX
// FX18 LD sound=VX
// FX1E ADD I+=VX
// FX29 LD  I=sprite_addr[Vx] set sprite char (4x5 sprite)
// FX33 LD set_BCD(Vx) => *(I+0)=3 *(I+1)=2 *(I+2)=1
// FX55 LD reg_dump(Vx, &I)
// FX65 LD reg_load(Vxm &I)

// SuperChip 48
// 00Cn - SCD nibble
// 00FB - SCR
// 00FC - SCL
// 00FD - EXIT
// 00FE - LOW
// 00FF - HIGH
// Dxy0 - DRW Vx, Vy, 0
// Fx30 - LD HF, Vx
// Fx75 - LD R, Vx
// Fx85 - LD Vx, R