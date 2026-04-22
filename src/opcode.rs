pub mod opcode {
    //Special
    pub const HALT: u32 = 0; //Done
    //ALU - handled by execute stage and writeback
    // RR = register + register
    // RI = register + immediate
    pub const ADD_RR: u32 = 1; //Done
    pub const ADD_RI: u32 = 2; //Done
    pub const SUB_RR: u32 = 3;
    pub const SUB_RI: u32 = 4;
    pub const MUL_RR: u32 = 5;
    pub const MUL_RI: u32 = 6;
    pub const DIV_RR: u32 = 7;
    pub const DIV_RI: u32 = 8;
    pub const MOD_RR: u32 = 9;
    pub const MOD_RI: u32 = 10;
    pub const AND_RR: u32 = 11;
    pub const AND_RI: u32 = 12;
    pub const OR_RR: u32 = 13;
    pub const OR_RI: u32 = 14;
    pub const NOT: u32 = 15;
    pub const XOR_RR: u32 = 16;
    pub const XOR_RI: u32 = 17;
    pub const LS_RR: u32 = 18;
    pub const LS_RI: u32 = 19;
    pub const RS_RR: u32 = 20;
    pub const RS_RI: u32 = 21;
    pub const LSL_RR: u32 = 22;
    pub const LSL_RI: u32 = 23;
    pub const LSR_RR: u32 = 24;
    pub const LSR_RI: u32 = 25;

    //control flow - handled by excecute and writeback
    //D = register direct
    //I = register indirect 
    //PC = PC relative
    pub const JMP_D: u32 = 26; //Done
    pub const JMP_I: u32 = 27;  //Done
    pub const JMP_PC: u32 = 28;  //Done       //JMPS save lr for return
    pub const CMP_RR: u32 = 29; //Done
    pub const CMP_RI: u32 = 30;//Done
    pub const JE_D: u32 = 31; //Done
    pub const JE_I: u32 = 32; //Done
    pub const JE_PC: u32 = 33; //Done
    pub const JL_D: u32 = 34; //Done
    pub const JL_I: u32 = 35; //Done
    pub const JL_PC: u32 = 36; //Done
    pub const JG_D: u32 = 37; //Done
    pub const JG_I: u32 = 38; //Done
    pub const JG_PC: u32 = 39; //Done
    pub const RET: u32 = 40; //Done

    //memory - handled by memory and writeback stages

    pub const LDR_D: u32 = 41; //Done
    pub const LDR_I: u32 = 42; //Done
    pub const LDR_PC: u32 = 43; //Done
    pub const STR_D: u32 = 44; //Done
    pub const STR_I: u32 = 45; //Done
    pub const STR_PC: u32 = 46; //Done
    pub const GDR_D: u32 = 47; //Done
    pub const GDR_I: u32 = 48; //Done
    pub const GDR_PC: u32 = 49; //Done
    pub const GTR_D: u32 = 50; //Done
    pub const GTR_I: u32 = 51; //Done
    pub const GTR_PC: u32 = 52; //Done
    pub const PSH: u32 = 53; //Done
    pub const POP: u32 = 55; //Done
    pub const FDR: u32 = 58; //frame load, pulls graphics memory into frame buffer //Done
    pub const FTR: u32 = 59; //frame store, puts frame buffer into graphics memory //Done


}