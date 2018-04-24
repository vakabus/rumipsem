
enum Instruction {
    ADD,    // Add Word
    ADDI,   //Add Immediate Word
    ADDIU,  //Add Immediate Unsigned Word
    ADDU,   //Add Unsigned Word
    CLO,    //Count Leading Ones in Word
    CLZ,    //Count Leading Zeros in Word
    DIV,    //Divide Word
    DIVU,   //Divide Unsigned Word
    MADD,   //Multiply and Add Word to Hi, Lo
    MADDU,  //Multiply and Add Unsigned Word to Hi, Lo
    MSUB,   //Multiply and Subtract Word to Hi, Lo
    MSUBU,  //Multiply and Subtract Unsigned Word to Hi, Lo
    MUL,    //Multiply Word to GPR
    MULT,   //Multiply Word
    MULTU,  //Multiply Unsigned Word
    SLT,    //Set on Less Than
    SLTI,   //Set on Less Than Immediate
    SLTIU,  //Set on Less Than Immediate Unsigned
    SLTU,   //Set on Less Than Unsigned
    SUB,    //Subtract Word
    SUBU,   //Subtract Unsigned Word
    B, //Unconditional Branch
BAL, //Branch and Link
BEQ, //Branch on Equal
BGEZ, //Branch on Greater Than or Equal to Zero
BGEZAL, //Branch on Greater Than or Equal to Zero and Link
BGTZ, //Branch on Greater Than Zero
BLEZ, //Branch on Less Than or Equal to Zero
BLTZ, //Branch on Less Than Zero
    BLTZAL, //Branch on Less Than Zero and Link
BNE, //Branch on Not Equal
J, //Jump
JAL, //Jump and Link
JALR, //Jump and Link Register
JR, //Jump Register
    NOP, //No Operation
SSNOP, //Superscalar No Operation
    LB, //Load Byte
LBU, //Load Byte Unsigned
LH, //Load Halfword
LHU, //Load Halfword Unsigned
LL, //Load Linked Word
LW, //Load Word
LWL, //Load Word Left
LWR, //Load Word Right
PREF, //Prefetch
SB, //Store Byte
SC, //Store Conditional Word
SD, //Store Doubleword
SH, //Store Halfword
SW, //Store Word
SWL, //Store Word Left
SWR, //Store Word Right
SYNC, //Synchronize Shared Memory
    AND, //And
ANDI, //And Immediate
LUI, //Load Upper Immediate
NOR, //Not Or
OR, //Or
ORI, //Or Immediate
XOR, //Exclusive Or
XORI, //Exclusive Or Immediate
    MFHI, //Move From HI Register
MFLO, //Move From LO Register
MOVF, //Move Conditional on Floating Point False
MOVN, //Move Conditional on Not Zero
MOVT, //Move Conditional on Floating Point True
MOVZ, //Move Conditional on Zero
MTHI, //Move To HI Register
MTLO, //Move To LO Register
    SLL, //Shift Word Left Logical
SLLV, //Shift Word Left Logical Variable
SRA, //Shift Word Right Arithmetic
SRAV, //Shift Word Right Arithmetic Variable
SRL, //Shift Word Right Logical
SRLV, //Shift Word Right Logical Variable
    BREAK, //Breakpoint
SYSCALL, //System Call
TEQ, //Trap if Equal
TEQI, //Trap if Equal Immediate
TGE, //Trap if Greater or Equal
TGEI, //Trap if Greater of Equal Immediate
TGEIU, //Trap if Greater or Equal Immediate Unsigned
TGEU, //Trap if Greater or Equal Unsigned
TLT, //Trap if Less Than
TLTI, //Trap if Less Than Immediate
TLTIU, //Trap if Less Than Immediate Unsigned
TLTU, //Trap if Less Than Unsigned
TNE, //Trap if Not Equal
TNEI, //Trap if Not Equal Immediate
    ABS_fmt, //Floating Point Absolute Value
ADD_fmt, //Floating Point Add
DIV_fmt, //Floating Point Divide
MADD_fmt, //Floating Point Multiply Add
MSUB_fmt, //Floating Point Multiply Subtract
MUL_fmt, //Floating Point Multiply
NEG_fmt, //Floating Point Negate
NMADD_fmt, //Floating Point Negative Multiply Add
NMSUB_fmt, //Floating Point Negative Multiply Subtract
RECIP_fmt, //Reciprocal Approximation
RSQRT_fmt, //Reciprocal Square Root Approximation
SQRT, //Floating Point Square Root
SUB_fmt, //Floating Point Subtract
BC1F, //Branch on FP False
BC1T, //Branch on FP True
C_cond_fmt, //Floating Point Compare
CEIL_W_fmt, //Floating Point Ceiling Convert to Word Fixed Point
CVT_D_fmt, //Floating Point Convert to Double Floating Point
CVT_S_fmt, //Floating Point Convert to Single Floating Point
CVT_W_fmt, //Floating Point Convert to Word Fixed Point
FLOOR_W_fmt, //Floating Point Floor Convert to Word Fixed Point
ROUND_W_fmt, //Floating Point Round to Word Fixed Point
TRUNC_W_fmt, //Floating Point Truncate to Word Fixed Point
LDC1, //Load Doubleword to Floating Point
LWC1, //Load Word to Floating Point
SDC1, //Store Doubleword from Floating Point
SWC1, //Store Word from Floating Point
CFC1, //Move Control Word from Floating Point
CTC1, //Move Control Word to Floating Point
MFC1, //Move Word from Floating Point
MOV_fmt, //Floating Point Move
MOVF_fmt, //Floating Point Move Conditional on Floating Point False
MOVN_fmt, //Floating Point Move Conditional on Not Zero
MOVT_fmt, //Floating Point Move Conditional on Floating Point True
MOVZ_fmt, //Floating Point Move Conditional on Zero
MTC1, //Move Word to Floating Point
CACHE, //Perform Cache Operation
ERET, //Exception Return
MFC0, //Move from Coprocessor 0
MTC0, //Move to Coprocessor 0
TLBP, //Probe TLB for Matching Entry
TLBR, //Read Indexed TLB Entry
TLBWI, //Write Indexed TLB Entry
TLBWR, //Write Random TLB Entry
WAIT, //Enter Standby Mode
DERET, //Debug Exception Return
SDBBP, //Software Debug Breakpoint
}

fn parse_instruction(instruction: u32) -> Instruction {

}