//pub mod instruction;
//pub mod memory;
//use doomed_isa::instruction::*;
//use doomed_isa::memory::*;

use std::{collections::btree_map::Range, io, ptr::null};



use doomed_isa::{memory::memory::GRAPHICS_OFFSET, opcode::opcode::RS_RR};

use eframe::{App, wgpu::Color};
use egui::{Color32, ColorImage, Grid, ScrollArea, TextureHandle, util::id_type_map};
use egui_extras::{Column, TableBuilder};

use crate::{instruction::instruction::{Devices, Instruction, InstructionType}, 
memory::memory::{CACHE_SIZE, Cache, MEMORY_SIZE, Registers, ReturnVal}, 
opcode::opcode::{ADD_RI, ADD_RR, AND_RI, CMP_RI, CMP_RR, DIV_RI, FDR, FTR, GDR_D, GDR_I, GDR_PC, GTR_D, GTR_I, GTR_PC, HALT, JE_D, JE_I, JE_PC, JG_D, JG_I, JG_PC, JL_D, JL_I, JL_PC, JMP_D, JMP_I, JMP_PC, LDR_D, LDR_I, LDR_PC, LS_RI, LSL_RI, LSR_RI, MOD_RI, MUL_RI, OR_RI, POP, POP_LR, PSH, PSH_LR, RET, RS_RI, STR_D, STR_I, STR_PC, SUB_RI, XOR_RI}, pipeline::pipeline::Controler};
use crate::{pipeline::pipeline::{Decode, Memory, Fetch, Execute, Writeback}};

use std::cell::RefCell;
use std::rc::Rc;

use std::fs::File;
use std::io::{Write, Result};

pub mod instruction;
pub mod memory;
pub mod pipeline;
pub mod opcode;

const TYPE_SHIFT: u32 = 31;
const OPCODE_SHIFT: u32 = 25;
const REG1_SHIFT: u32 = 20;
const REG2_SHIFT: u32 = 15;
const REG3_SHIFT: u32 = 10;
const IMMEDIATE1_SHIFT: u32 = 13;
const IMMEDIATE2_SHIFT: u32 = 8;
const POST_REG_SHIFT: u32 = 3;
const IMMEDIATE3_SHIFT: u32 = 3;

const TYPE_MASK: u32 = 0b1;
const OPCODE_MASK: u32 = 0b11_1111;
const REG_MASK: u32 = 0b1_1111;
const IMMEDIATE_MASK: u32 = 0b1111_1111_1111;

// fn main() {

    // let type_field:u32 = 0;
    // let opcode = ADD_RI;
    // let reg1 = 2;
    // let reg2 = 1;
    // let reg3 = 2;
    // let imm2: u32 = 1;
    // let imm3: u32 = 0;

//      let instr_binary = (type_field << TYPE_SHIFT)
//                 | (opcode << OPCODE_SHIFT)
//                 | (reg1 << REG1_SHIFT)
//                 | (imm2 << IMMEDIATE2_SHIFT)
//                 | (reg3 << POST_REG_SHIFT);

//     println!("{}", instr_binary.to_string());

//     let type_field = ((instr_binary >> TYPE_SHIFT) & TYPE_MASK) as u8;
//     let opcode     = ((instr_binary >> OPCODE_SHIFT) & OPCODE_MASK) as u8;
//     let reg1       = ((instr_binary >> REG1_SHIFT) & REG_MASK) as u8;
//     let imm2       = ((instr_binary >> IMMEDIATE2_SHIFT) & IMMEDIATE_MASK) as u8;
//     let reg3       = ((instr_binary >> POST_REG_SHIFT) & REG_MASK) as u8;

//     println!("{}, {}, {}, {}, {}", type_field, opcode, reg1, imm2, reg3);

//     let index = 6 / 4;
//     let offset = 6 % 4;

//     println!("{}, {}", index, offset);

//     //test_memory();

//     //test_fetch();

//     //test_decode();

//     //test_excecute();
    
//     //test_mem_stage();

    //test_writeback();
    //test_improved_memory();
    //test_control_flow();

//     //test_cache_switch();

    //test_pipe_switch();

    //test_graphics_instructions();

    //test_push_pop();
    
    //test_jmp_ret();

//     test_pc_rel();
    
// }

pub fn test_pc_rel() {
    let i0 = instr_fields_to_decimal(0, ADD_RI, 2, 5, 2, InstructionType::ALU);
    let i1 = instr_fields_to_decimal(0, ADD_RI, 0, 2, 0, InstructionType::ALU);
    let i2 = instr_fields_to_decimal(0, STR_PC, 2, 0, 0, InstructionType::Memory); //should store in addr 2
    let i3 = instr_fields_to_decimal(0, LDR_PC, 0, 2, 0, InstructionType::Memory); // 0 second arg bc decode autofills sp index
    let i4 = instr_fields_to_decimal(0, CMP_RI, 2, 5, 0, InstructionType::ALU);
    let i5 = instr_fields_to_decimal(1, JG_PC, 2, 0, 0, InstructionType::Memory); ///2 in R0, so hops to 5 + 2 = 7
    let i6 = instr_fields_to_decimal(0, HALT, 0, 0, 0, InstructionType::Control);
    let i7 = instr_fields_to_decimal(0, ADD_RI, 0, 1, 0, InstructionType::Control);
    let i8 = instr_fields_to_decimal(0, HALT, 0, 0, 0, InstructionType::Control);

    create_binary_file("src/programs/graphics-test.bin", &[i0, i1, i2, i3, i4, i5, i6, i7, i8]);
    
    let mut reg = Registers::new();
    //reg.update_gp(1, 600);
    let mut cache = Cache::new([[-1; 7]; CACHE_SIZE as usize], [[-1; 4]; MEMORY_SIZE as usize]);
    cache.load_memory_from_file("src/programs/graphics-test.bin".to_string());

    let mut ctrl = Controler::new();
    //ctrl.switch(true);

    let cache_ref = &mut cache;
    let reg_ref = &mut reg;
    let ctrl_ref = &mut ctrl;
    let mut fetch = Fetch::new();
    let mut decode = Decode::new(fetch);
    let mut excecute = Execute::new(decode);
    let mut memory = Memory::new( excecute);
    let mut writeback = Writeback::new( memory);

    let mut wb_ret: Option<Instruction> = None;

    while wb_ret.is_none() || (wb_ret.is_some() && wb_ret.unwrap().opcode != HALT as i32) {
        println!("{}", "  ");
        println!("{}", "Writeback State: ");
        println!("{}", writeback.state());
        println!("{}", "  ");
        println!("{}", "Memory State: ");
        println!("{}", writeback.mem_stage.state());
        println!("{}", "  ");
        println!("{}", "Excecute State: ");
        println!("{}", writeback.mem_stage.exec_stage.state());
        println!("{}", "  ");
        println!("{}", "Decode State: ");
        let (cur, dec):(String, String) = writeback.mem_stage.exec_stage.dec_stage.state();
        println!("{} \n {}", cur, dec);
        println!("{}", "  ");
        println!("{}", "Fetch State: ");
        let (cur_i, pc) = writeback.mem_stage.exec_stage.dec_stage.fetch_stage.state();
        println!("{} \n {}", cur_i, pc);
        println!("{}", "  ");
        wb_ret = writeback.call(reg_ref, cache_ref, ctrl_ref);
    }

    println!("made it");
}

pub fn test_jmp_ret() {
    let i0 = instr_fields_to_decimal(0, ADD_RI, 2, 5, 2, InstructionType::ALU);
    let i1 = instr_fields_to_decimal(0, ADD_RI, 0, 1, 0, InstructionType::ALU);
    let i2 = instr_fields_to_decimal(0, JMP_D, 2, 0, 0, InstructionType::Control);
    let i3 = instr_fields_to_decimal(0, PSH, 0, 0, 0, InstructionType::Memory); // 0 second arg bc decode autofills sp index
    let i4 = instr_fields_to_decimal(0, ADD_RI, 2, 5, 0, InstructionType::ALU);
    let i5 = instr_fields_to_decimal(0, POP, 0, 0, 0, InstructionType::Memory); //similar here for first arg
    let i6 = instr_fields_to_decimal(0, RET, 0, 0, 0, InstructionType::Control);

    create_binary_file("src/programs/graphics-test.bin", &[i0, i1, i2, i3, i4, i5, i6]);
    
    let mut reg = Registers::new();
    //reg.update_gp(1, 600);
    let mut cache = Cache::new([[-1; 7]; CACHE_SIZE as usize], [[-1; 4]; MEMORY_SIZE as usize]);
    cache.load_memory_from_file("src/programs/graphics-test.bin".to_string());

    let mut ctrl = Controler::new();
    //ctrl.switch(true);

    let cache_ref = &mut cache;
    let reg_ref = &mut reg;
    let ctrl_ref = &mut ctrl;
    let mut fetch = Fetch::new();
    let mut decode = Decode::new(fetch);
    let mut excecute = Execute::new(decode);
    let mut memory = Memory::new( excecute);
    let mut writeback = Writeback::new( memory);

    let mut wb_ret: Option<Instruction> = None;

    while wb_ret.is_none() || (wb_ret.is_some() && wb_ret.unwrap().instr_type != InstructionType::NOOP) {
        println!("{}", "  ");
        println!("{}", "Writeback State: ");
        println!("{}", writeback.state());
        println!("{}", "  ");
        println!("{}", "Memory State: ");
        println!("{}", writeback.mem_stage.state());
        println!("{}", "  ");
        println!("{}", "Excecute State: ");
        println!("{}", writeback.mem_stage.exec_stage.state());
        println!("{}", "  ");
        println!("{}", "Decode State: ");
        let (cur, dec):(String, String) = writeback.mem_stage.exec_stage.dec_stage.state();
        println!("{} \n {}", cur, dec);
        println!("{}", "  ");
        println!("{}", "Fetch State: ");
        let (cur_i, pc) = writeback.mem_stage.exec_stage.dec_stage.fetch_stage.state();
        println!("{} \n {}", cur_i, pc);
        println!("{}", "  ");
        wb_ret = writeback.call(reg_ref, cache_ref, ctrl_ref);
    }

    println!("made it");
}

pub fn test_push_pop() {

    let i0 = instr_fields_to_decimal(0, ADD_RI, 2, 1, 2, InstructionType::ALU);
    let i1 = instr_fields_to_decimal(0, ADD_RI, 0, 1, 0, InstructionType::ALU);
    let i2 = instr_fields_to_decimal(0, PSH, 2, 0, 0, InstructionType::Memory);
    let i3 = instr_fields_to_decimal(0, PSH, 0, 0, 0, InstructionType::Memory); // 0 second arg bc decode autofills sp index
    let i4 = instr_fields_to_decimal(0, ADD_RI, 2, 5, 0, InstructionType::ALU);
    let i5 = instr_fields_to_decimal(0, POP, 0, 0, 0, InstructionType::Memory); //similar here for first arg
    let i6 = instr_fields_to_decimal(0, POP, 0, 2, 0, InstructionType::Memory);

    create_binary_file("src/programs/graphics-test.bin", &[i0, i1, i2, i3, i4, i5, i6]);
    
    let mut reg = Registers::new();
    //reg.update_gp(1, 600);
    let mut cache = Cache::new([[-1; 7]; CACHE_SIZE as usize], [[-1; 4]; MEMORY_SIZE as usize]);
    cache.load_memory_from_file("src/programs/graphics-test.bin".to_string());

    let mut ctrl = Controler::new();
    //ctrl.switch(true);

    let cache_ref = &mut cache;
    let reg_ref = &mut reg;
    let ctrl_ref = &mut ctrl;
    let mut fetch = Fetch::new();
    let mut decode = Decode::new(fetch);
    let mut excecute = Execute::new(decode);
    let mut memory = Memory::new( excecute);
    let mut writeback = Writeback::new( memory);

    let mut wb_ret: Option<Instruction> = None;

    while wb_ret.is_none() || (wb_ret.is_some() && wb_ret.unwrap().instr_type != InstructionType::NOOP) {
        println!("{}", "  ");
        println!("{}", "Writeback State: ");
        println!("{}", writeback.state());
        println!("{}", "  ");
        println!("{}", "Memory State: ");
        println!("{}", writeback.mem_stage.state());
        println!("{}", "  ");
        println!("{}", "Excecute State: ");
        println!("{}", writeback.mem_stage.exec_stage.state());
        println!("{}", "  ");
        println!("{}", "Decode State: ");
        let (cur, dec):(String, String) = writeback.mem_stage.exec_stage.dec_stage.state();
        println!("{} \n {}", cur, dec);
        println!("{}", "  ");
        println!("{}", "Fetch State: ");
        let (cur_i, pc) = writeback.mem_stage.exec_stage.dec_stage.fetch_stage.state();
        println!("{} \n {}", cur_i, pc);
        println!("{}", "  ");
        wb_ret = writeback.call(reg_ref, cache_ref, ctrl_ref);
    }

    println!("made it");
    //appears to work fine! remember to save your registers

}

pub fn test_graphics_instructions() {
    let i0 = instr_fields_to_decimal(0, ADD_RI, 2, 1, 2, InstructionType::ALU);
    let i1 = instr_fields_to_decimal(0, GTR_D, 0, 0, 0, InstructionType::Memory);
    let i2 = instr_fields_to_decimal(0, ADD_RI, 0, 1, 0, InstructionType::ALU);
    let i3 = instr_fields_to_decimal(0, CMP_RR, 0, 1, 0, InstructionType::Control);
    let i4 = instr_fields_to_decimal(0, JL_D, 2, 0, 0, InstructionType::Control);
    let i5 = Instruction {
        type_field: 0,
        instr_type: InstructionType::Memory,
        device: Devices::Memory,
        opcode: FDR as i32,
        arg1: 0,
        arg2: 0,
        arg3: 0,
        reg1: 0,
        reg2: 0,
        reg3: 0,
        result: None,
        pc: 5
    };

    create_binary_file("src/programs/graphics-test.bin", &[i0, i1, i2, i3, i4]);
    
    let mut reg = Registers::new();
    reg.update_gp(1, 600);
    let mut cache = Cache::new([[-1; 7]; CACHE_SIZE as usize], [[-1; 4]; MEMORY_SIZE as usize]);
    cache.load_memory_from_file("src/programs/graphics-test.bin".to_string());

    let mut ctrl = Controler::new();
    //ctrl.switch(true);

    let cache_ref = &mut cache;
    let reg_ref = &mut reg;
    let ctrl_ref = &mut ctrl;
    let mut fetch = Fetch::new();
    let mut decode = Decode::new(fetch);
    let mut excecute = Execute::new(decode);
    let mut memory = Memory::new( excecute);
    let mut writeback = Writeback::new( memory);

    let mut wb_ret: Option<Instruction> = None;

    while wb_ret.is_none() || (wb_ret.is_some() && wb_ret.unwrap().instr_type != InstructionType::NOOP) {
        println!("{}", "  ");
        println!("{}", "Writeback State: ");
        println!("{}", writeback.state());
        println!("{}", "  ");
        println!("{}", "Memory State: ");
        println!("{}", writeback.mem_stage.state());
        println!("{}", "  ");
        println!("{}", "Excecute State: ");
        println!("{}", writeback.mem_stage.exec_stage.state());
        println!("{}", "  ");
        println!("{}", "Decode State: ");
        let (cur, dec):(String, String) = writeback.mem_stage.exec_stage.dec_stage.state();
        println!("{} \n {}", cur, dec);
        println!("{}", "  ");
        println!("{}", "Fetch State: ");
        let (cur_i, pc) = writeback.mem_stage.exec_stage.dec_stage.fetch_stage.state();
        println!("{} \n {}", cur_i, pc);
        println!("{}", "  ");
        wb_ret = writeback.call(reg_ref, cache_ref, ctrl_ref);
    }

    println!("made it");

    let mut cache_ret = ReturnVal::Wait(true);
    let mut i = 0;
    while cache_ret == ReturnVal::Wait(true) {
        println!("cache delay = {}", i);
        cache_ret = cache_ref.call(i5);
        i = i + 1;
    }


    println!("made it 2");

    let i6 =  Instruction {
        type_field: 0,
        instr_type: InstructionType::Memory,
        device: Devices::Memory,
        opcode: FTR as i32,
        arg1: 0,
        arg2: 0,
        arg3: 0,
        reg1: 0,
        reg2: 0,
        reg3: 0,
        result: None,
        pc: 5
    };

    cache_ret = ReturnVal::Wait(true);
    i = 0;
    while cache_ret == ReturnVal::Wait(true) {
        println!("cache delay = {}", i);
        cache_ret = cache_ref.call(i6);
        i = i + 1;
    }

    println!("made it 3");

    let i7 = Instruction {
        type_field: 0,
        instr_type: InstructionType::Memory,
        device: Devices::Memory,
        opcode: GTR_D as i32,
        arg1: 4,
        arg2: 7,
        arg3: 0,
        reg1: 0,
        reg2: 0,
        reg3: 0,
        result: None,
        pc: 5
    };

    cache_ret = ReturnVal::Wait(true);
    i = 0;
    while cache_ret == ReturnVal::Wait(true) {
        //println!("cache delay = {}", i);
        cache_ret = cache_ref.call(i7);
        i = i + 1;
    }

    println!("made it 4");




}

pub fn test_pipe_switch() {
    //i1: LDR load from addr R3 into R1
    //i2: ADD R1 + 1 into R2
    //i3  STR store R2 in addr R3
    //i4  ADD R3 + 1 into R3
    //i5  ADD R0 + 1 into R0
    //i6  CMP R0, 4
    //i7  JL  to addr 0
    //i8 done, no halt implemented yet
    let i1 = instr_fields_to_decimal(0, LDR_D, 3, 1, 0, InstructionType::Memory);
    let i2 = instr_fields_to_decimal(0, ADD_RI, 0, 1, 2, InstructionType::ALU);
    let i3 = instr_fields_to_decimal(0, STR_D, 2, 3, 0, InstructionType::Memory);
    let i4 = instr_fields_to_decimal(0, ADD_RI, 3, 1, 3, InstructionType::ALU);
    let i5 = instr_fields_to_decimal(0, ADD_RI, 0, 1, 0, InstructionType::ALU);
    let i6 = instr_fields_to_decimal(0,CMP_RI,0,4,0,InstructionType::Control);
    let i7 = instr_fields_to_decimal(0,JL_D, 4,0,0,InstructionType::Control);
    let i8 = instr_fields_to_decimal(0,65,12,12,12,InstructionType::NOOP);

    create_binary_file("src/programs/fetch-test.bin", &[i1, i2, i3, i4,i5, i6, i7, i8, 1, 2,3]);
    let mut reg = Registers::new();
    reg.update_gp(0 as usize, 0);
    reg.update_gp(1 as usize, 0);
    reg.update_gp(2 as usize, 0);
    reg.update_gp(3 as usize, 8);
    reg.update_gp(4 as usize,0);
    let mut cache = Cache::new([[-1; 7]; CACHE_SIZE as usize], [[-1; 4]; MEMORY_SIZE as usize]);
    cache.load_memory_from_file("src/programs/fetch-test.bin".to_string());

    let mut ctrl = Controler::new();
    ctrl.switch(false);

    let cache_ref = &mut cache;
    let reg_ref = &mut reg;
    let ctrl_ref = &mut ctrl;
    let mut fetch = Fetch::new();
    let mut decode = Decode::new(fetch);
    let mut excecute = Execute::new(decode);
    let mut memory = Memory::new( excecute);
    let mut writeback = Writeback::new( memory);
    
    let mut wb_ret = writeback.call(reg_ref, cache_ref, ctrl_ref);
    
    while wb_ret.is_none() || (wb_ret.is_some() && wb_ret.unwrap().instr_type != InstructionType::NOOP) {
        println!("{}", "  ");
        println!("{}", "Writeback State: ");
        println!("{}", writeback.state());
        println!("{}", "  ");
        println!("{}", "Memory State: ");
        println!("{}", writeback.mem_stage.state());
        println!("{}", "  ");
        println!("{}", "Excecute State: ");
        println!("{}", writeback.mem_stage.exec_stage.state());
        println!("{}", "  ");
        println!("{}", "Decode State: ");
        let (cur, dec):(String, String) = writeback.mem_stage.exec_stage.dec_stage.state();
        println!("{} \n {}", cur, dec);
        println!("{}", "  ");
        println!("{}", "Fetch State: ");
        let (cur_i, pc) = writeback.mem_stage.exec_stage.dec_stage.fetch_stage.state();
        println!("{} \n {}", cur_i, pc);
        println!("{}", "  ");
        wb_ret = writeback.call(reg_ref, cache_ref, ctrl_ref);
    }

}

pub fn test_cache_switch() {
    let load_d = Instruction {
        instr_type: InstructionType::Memory,
        device: Devices::Memory,
        opcode: LDR_D as i32,
        type_field: 0,
        arg1: 4,
        arg2: 0,
        arg3: 0,
        reg1: 0,
        reg2: 0,
        reg3: 0,
        result: None,
        pc: 0
    };

    let load_i = Instruction {
        instr_type: InstructionType::Memory,
        device: Devices::Memory,
        opcode: LDR_I as i32,
        type_field: 0,
        arg1: 0,
        arg2: 0,
        arg3: 0,
        reg1: 0,
        reg2: 0,
        reg3: 0,
        result: None,
        pc: 0
    };

    let str_d = Instruction {
        instr_type: InstructionType::Memory,
        device: Devices::Memory,
        opcode: STR_D as i32,
        type_field: 0,
        arg1: 4,
        arg2: 0,
        arg3: 0,
        reg1: 0,
        reg2: 0,
        reg3: 0,
        result: None,
        pc: 0
    };

    let str_i = Instruction {
        instr_type: InstructionType::Memory,
        device: Devices::Memory,
        opcode: STR_I as i32,
        type_field: 0,
        arg1: 2,
        arg2: 0,
        arg3: 0,
        reg1: 0,
        reg2: 0,
        reg3: 0,
        result: None,
        pc: 0
    };

    let mut cache = Cache::new([[-1; 7]; CACHE_SIZE as usize], [[-1; 4]; MEMORY_SIZE as usize]);
    cache.switch(false);

    let mut ret = cache.call(str_d);
    ret = cache.call(str_d);
    ret = cache.call(str_d);
    ret = cache.call(str_d);

    println!("{}", cache.get_memory(0)[0].to_string());
    println!("{}", cache.get_memory(0)[1].to_string());
    println!("{}", cache.get_memory(0)[2].to_string());
    println!("{}", cache.get_memory(0)[3].to_string());

    ret = cache.call(str_i);
    ret = cache.call(str_i);
    ret = cache.call(str_i);
    ret = cache.call(str_i);
    ret = cache.call(str_i);
    ret = cache.call(str_i);
    ret = cache.call(str_i);

    println!("{}", cache.get_memory(4)[0].to_string());
    println!("{}", cache.get_memory(4)[1].to_string());
    println!("{}", cache.get_memory(4)[2].to_string());
    println!("{}", cache.get_memory(4)[3].to_string());

    ret = cache.call(load_d);
    ret = cache.call(load_d);
    ret = cache.call(load_d);
    ret = cache.call(load_d);

    assert_eq!(ret, ReturnVal::Data(2));

    ret = cache.call(load_i);
    ret = cache.call(load_i);
    ret = cache.call(load_i);
    ret = cache.call(load_i);
    ret = cache.call(load_i);
    ret = cache.call(load_i);
    ret = cache.call(load_i);
    assert_eq!(ret, ReturnVal::Data(2));

}


pub fn test_improved_memory() {
    let load = Instruction {
        instr_type: InstructionType::Memory,
        device: Devices::Fetch,
        opcode: LDR_D as i32,
        type_field: 0,
        arg1: 10, //line 2, word 2
        arg2: 0,
        arg3: 0,
        reg1: 0,
        reg2: 0,
        reg3: 0,
        result: None,
        pc: 0
    };

    let load2 = Instruction {
        instr_type: InstructionType::Memory,
        device: Devices::Fetch,
        opcode: LDR_D as i32,
        type_field: 0,
        arg1: 7, //line 1, word 3
        arg2: 0,
        arg3: 0,
        reg1: 0,
        reg2: 0,
        reg3: 0,
        result: None,
        pc: 0
    };

    let load3 = Instruction {
        instr_type: InstructionType::Memory,
        device: Devices::Fetch,
        opcode: LDR_D as i32,
        type_field: 0,
        arg1: 69, //line 17, word 1
        arg2: 0,
        arg3: 0,
        reg1: 0,
        reg2: 0,
        reg3: 0,
        result: None,
        pc: 0
    };
    let mut main_memory = [[-1; 4]; MEMORY_SIZE as usize];
    main_memory[2][0] = 1;
    main_memory[2][1] = 2;
    main_memory[2][2] = 3;
    main_memory[2][3] = 4;

    main_memory[1][0] = 6;
    main_memory[1][1] = 7;
    main_memory[1][2] = 8;
    main_memory[1][3] = 9;

    main_memory[17][0] = 69;
    main_memory[17][1] = 70;
    main_memory[17][2] = 71;
    main_memory[17][3] = 72;

    let mut cache = Cache::new([[-1; 7]; CACHE_SIZE as usize], main_memory);

    let mut ret = cache.call(load);
    ret = cache.call(load);
    ret = cache.call(load);
    ret = cache.call(load); //returns here with mem delay 3

    assert_eq!(ret, ReturnVal::Data(3));

    ret = cache.call(load2);
    ret = cache.call(load2);
    ret = cache.call(load2);
    ret = cache.call(load2);

    assert_eq!(ret, ReturnVal::Data(9));

    ret = cache.call(load3);
    ret = cache.call(load3);
    ret = cache.call(load3);
    ret = cache.call(load3);
    
    assert_eq!(ret, ReturnVal::Data(70));
    //basic load works!

    let mut prefilled:[[i32; 7]; CACHE_SIZE as usize] = [[-1; 7]; CACHE_SIZE as usize];
    prefilled[0][0] = 8;
    prefilled[0][1] = 1;
    prefilled[1][0] = 9;
    prefilled[1][1] = 1;
    prefilled[2][0] = 10;
    prefilled[2][1] = 1;
    prefilled[3][0] = 7;
    prefilled[3][1] = 1;

    for i in 0..3 {
        for j in 0..3 {
            prefilled[i][j + 3] = i as i32;
        }
    }

    let mut cache2 = Cache::new(prefilled, main_memory);
    
    let store = Instruction {
        instr_type: InstructionType::Memory,
        device: Devices::Fetch,
        type_field: 0,
        opcode: STR_D as i32,
        arg1: 0,
        arg2: 2,
        arg3: 0,
        reg1: 0,
        reg2: 0,
        reg3: 0,
        result: None,
        pc: 0
    };

    let store2 = Instruction {
        instr_type: InstructionType::Memory,
        device: Devices::Fetch,
        type_field: 0,
        opcode: STR_D as i32,
        arg1: 2,
        arg2: 10,
        arg3: 0,
        reg1: 0,
        reg2: 0,
        reg3: 0,
        result: None,
        pc: 0
    };

    ret = cache2.call(store);
    ret = cache2.call(store);
    ret = cache2.call(store);
    ret = cache2.call(store);

    ret = cache2.call(store2);
    ret = cache2.call(store2);
    ret = cache2.call(store2);
    ret = cache2.call(store2);
    //basic store works!

    

    



}

pub fn test_control_flow() {
    //i1: LDR load from addr R3 into R1
    //i2: ADD R1 + 1 into R2
    //i3  STR store R2 in addr R3
    //i4  ADD R3 + 1 into R3
    //i5  ADD R0 + 1 into R0
    //i6  CMP R0, 4
    //i7  JL  to addr 0
    //i8 done, no halt implemented yet
    let i1 = instr_fields_to_decimal(0, LDR_D, 3, 1, 0, InstructionType::Memory);
    let i2 = instr_fields_to_decimal(0, ADD_RI, 0, 1, 2, InstructionType::ALU);
    let i3 = instr_fields_to_decimal(0, STR_D, 2, 3, 0, InstructionType::Memory);
    let i4 = instr_fields_to_decimal(0, ADD_RI, 3, 1, 3, InstructionType::ALU);
    let i5 = instr_fields_to_decimal(0, ADD_RI, 0, 1, 0, InstructionType::ALU);
    let i6 = instr_fields_to_decimal(0,CMP_RI,0,4,0,InstructionType::Control);
    let i7 = instr_fields_to_decimal(0,JL_D, 4,0,0,InstructionType::Control);
    let i8 = instr_fields_to_decimal(0,65,12,12,12,InstructionType::NOOP);

    create_binary_file("src/programs/fetch-test.bin", &[i1, i2, i3, i4,i5, i6, i7, i8, 1, 2,3]);
    let mut reg = Registers::new();
    reg.update_gp(0 as usize, 0);
    reg.update_gp(1 as usize, 0);
    reg.update_gp(2 as usize, 0);
    reg.update_gp(3 as usize, 8);
    reg.update_gp(4 as usize,0);
    let mut cache = Cache::new([[-1; 7]; CACHE_SIZE as usize], [[-1; 4]; MEMORY_SIZE as usize]);
    cache.load_memory_from_file("src/programs/fetch-test.bin".to_string());

    let cache_ref = &mut cache;
    let reg_ref = &mut reg;
    let mut fetch = Fetch::new();
    let mut decode = Decode::new(fetch);
    let mut excecute = Execute::new(decode);
    let mut memory = Memory::new( excecute);
    let mut writeback = Writeback::new( memory);
    
    let mut wb_ret = writeback.call(reg_ref, cache_ref, &mut Controler::new());
    
    while wb_ret.is_none() || (wb_ret.is_some() && wb_ret.unwrap().instr_type != InstructionType::NOOP) {
        println!("{}", "  ");
        println!("{}", "Writeback State: ");
        println!("{}", writeback.state());
        println!("{}", "  ");
        println!("{}", "Memory State: ");
        println!("{}", writeback.mem_stage.state());
        println!("{}", "  ");
        println!("{}", "Excecute State: ");
        println!("{}", writeback.mem_stage.exec_stage.state());
        println!("{}", "  ");
        println!("{}", "Decode State: ");
        let (cur, dec):(String, String) = writeback.mem_stage.exec_stage.dec_stage.state();
        println!("{} \n {}", cur, dec);
        println!("{}", "  ");
        println!("{}", "Fetch State: ");
        let (cur_i, pc) = writeback.mem_stage.exec_stage.dec_stage.fetch_stage.state();
        println!("{} \n {}", cur_i, pc);
        println!("{}", "  ");
        wb_ret = writeback.call(reg_ref, cache_ref, &mut Controler::new());
    }



}

pub fn instr_fields_to_decimal(type_field: u32, opcode: u32, arg1: u32, arg2: u32, arg3: u32, instr_type: InstructionType) -> u32{
    if instr_type == InstructionType::ALU {
        if opcode == ADD_RI  || opcode == SUB_RI || opcode == MUL_RI || opcode == DIV_RI || opcode == AND_RI
        || opcode == OR_RI || opcode == XOR_RI || opcode == MOD_RI || opcode == XOR_RI || opcode == LSL_RI
        || opcode == LSR_RI || opcode == LS_RI || opcode == RS_RI {
            return (type_field << TYPE_SHIFT)
                | (opcode << OPCODE_SHIFT)
                | (arg1 << REG1_SHIFT)
                | (arg2 << IMMEDIATE2_SHIFT)
                | (arg3 << POST_REG_SHIFT);
        }

        else {
            return (type_field << TYPE_SHIFT)
                | (opcode << OPCODE_SHIFT)
                | (arg1 << REG1_SHIFT)
                | (arg2 << REG2_SHIFT)
                | (arg3 << REG3_SHIFT);
        }
        
    }
    else if instr_type == InstructionType::Control {
        if opcode == CMP_RI  { //if CMP specifically
            
                return (type_field << TYPE_SHIFT)
                | (opcode << OPCODE_SHIFT)
                | (arg1 << REG1_SHIFT)
                | (arg2 << IMMEDIATE2_SHIFT)
                | (arg3);
        }  
        else if opcode == CMP_RR {
                return (type_field << TYPE_SHIFT)
                | (opcode << OPCODE_SHIFT)
                | (arg1 << REG1_SHIFT)
                | (arg2 << REG2_SHIFT)
                | (arg3 << REG3_SHIFT);
            }
        else if opcode == JMP_D || opcode == JL_D || opcode == JE_D || opcode == JG_D { //expect immediate address for direct jump
            return (type_field << TYPE_SHIFT)
                | (opcode << OPCODE_SHIFT)
                | (arg1 << IMMEDIATE1_SHIFT)
                | (arg2 << IMMEDIATE2_SHIFT) //will be
                | (arg3 << POST_REG_SHIFT);
        }
        else { //some register jump
            return (type_field << TYPE_SHIFT)
                | (opcode << OPCODE_SHIFT)
                | (arg1 << REG1_SHIFT)
                | (arg2 << REG2_SHIFT)
                | (arg3 << REG3_SHIFT);

            }
        }
        else if instr_type == InstructionType::Memory {
        return (type_field << TYPE_SHIFT)
                | (opcode << OPCODE_SHIFT)
                | (arg1 << REG1_SHIFT)
                | (arg2 << REG2_SHIFT)
                | (arg3 << IMMEDIATE3_SHIFT);

        }
        else {
            return 0;
        }

    }
    

pub fn test_memory() {
        let mut instr = Instruction {
            instr_type: InstructionType::Memory,
            device: Devices::Clock,
            type_field: 0,
            opcode: 21,
            arg1: 5,
            arg2: 5,
            arg3: 0,
            reg1: 0,
            reg2: 0,
            reg3: 0,
            result: None,
            pc: 0
        };

        let interupting_instr = Instruction {
            instr_type: InstructionType::Memory,
            device: Devices::Fetch,
            type_field: 0,
            opcode: 20,
            arg1: 5,
            arg2: 0,
            arg3: 0,
            reg1: 0,
            reg2: 0,
            reg3: 0,
            result: None,
            pc: 0
        };

        let interupting_instr_2 =  Instruction {
            instr_type: InstructionType::Control,
            device: Devices::Clock,
            type_field: 0,
            opcode: 20,
            arg1: 5,
            arg2: 5,
            arg3: 0,
            reg1: 0,
            reg2: 0,
            reg3: 0,
            result: None,
            pc: 0
        };

        let mut data: ReturnVal = ReturnVal::Wait(true);
        let reg = Registers::new();

        let mut cache = Cache::new([[-1; 7]; CACHE_SIZE as usize], [[-1; 4]; MEMORY_SIZE as usize]);

        data = cache.call(instr);
        data = cache.call(instr); //should take 1, 2, 3 calls to store due to write through
        data = cache.call(instr);
        data = cache.call(instr);

        assert_eq!(data, ReturnVal::Data(5), "Value 5 was not succesfully stored in index 1!");

        //println!("{}", cache.get_cache(5).to_string());
        //println!("{}", cache.get_memory(5).to_string());

        instr.arg2 = 6;
        instr.arg1 = 6;

        data = cache.call(instr);
        
        data = cache.call(interupting_instr);
        assert_eq!(data, ReturnVal::Wait(true), "Interupting instruction intercepted return value!");

        data = cache.call(interupting_instr_2);
        assert_eq!(data, ReturnVal::Wait(true),  "Interupting instruction intercepted return value!");

        data = cache.call(instr);
        data = cache.call(instr);
        data = cache.call(instr);
        
        assert_eq!(data, ReturnVal::Data(6), "Value 6 was not successfuly stored in index 2!");

        //println!("{}", cache.get_cache(6).to_string());
        //println!("{}", cache.get_memory(6).to_string());
        
        instr.arg2 = 7;
        instr.arg1 = 7;
        
        data = cache.call(instr);
        data = cache.call(instr);
        data = cache.call(instr);
        data = cache.call(instr);
        
        assert_eq!(data, ReturnVal::Data(7), "Value 7 was not successfuly stored in index 2!");

       //println!("{}", cache.get_cache(7).to_string());
       // println!("{}", cache.get_memory(7).to_string());

        //STR Works!

        let mut ldr = Instruction {
            instr_type: InstructionType::Memory,
            device: Devices::Clock,
            type_field: 0,
            opcode: 20,
            arg1: 5,
            arg2: 0,
            arg3: 0,
            reg1: 0,
            reg2: 0,
            reg3: 0,
            result: None,
            pc: 0
        };

        data = cache.call(ldr);

        assert_eq!(data, ReturnVal::Wait(true), "Something went wrong and the data was returned a cycle too soon!");
        
        data = cache.call(ldr);
        assert_eq!(data, ReturnVal::Data(5), "The cache missed erroneously, or 5 was not stored correctly");

        ldr.arg1 = 8;

        data = cache.call(ldr); //dead call
        data = cache.call(ldr); //delay = 1
        assert_eq!(data, ReturnVal::Wait(true), "Something went wrong and the cache thought it had a hit!");

        data = cache.call(ldr); //delay = 2
        data = cache.call(ldr); //delay = 3

        assert_eq!(data, ReturnVal::Data(-1), "Cache returned wait too many times!");

        //LDR works!

        
        //test load from file

        cache.load_memory_from_file("src/programs/instructions.bin".to_string()); //note that it is necessary to properly generate the bin files via rust code or the command line

        //assert_eq!(cache.get_memory(0), 0);
        //assert_eq!(cache.get_memory(1), 1);
        //assert_eq!(cache.get_memory(2), 2);
        //assert_eq!(cache.get_memory(3), 3);


        //test load worked!







        return;
    }

    pub fn create_binary_file(path: &str, words: &[u32]) {
        let file: std::result::Result<File, io::Error> = File::create(path);
        match file {
            Result::Ok(mut f) => {
                for &word in words {
                    let _ = f.write_all(&word.to_le_bytes());
                }
            }

            Result::Err(_x) => {
                return;
            }
        }

        return;

    }

    pub fn test_fetch()  {
        create_binary_file("src/programs/fetch-test.bin", &[20,21,20,21,10,9]);
        let mut reg = Registers::new();
        let mut cache =Cache::new([[-1; 7]; CACHE_SIZE as usize], [[-1; 4]; MEMORY_SIZE as usize]); 
        cache.load_memory_from_file("src/programs/fetch-test.bin".to_string());

        let cache_ref = &mut cache;
        let mut fetch = Fetch::new();
        let mut fetch_return: (InstructionType, i32, i32);
        let reg_ref = &mut reg;
        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref,&mut Controler::new()); //makes first call to cache

        assert_eq!(fetch_return, (InstructionType::Stall, -1, -1), "Returned something wrong on first call");

        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref,&mut Controler::new()); //counter = 1
        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref,&mut Controler::new()); //counter = 2
        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref,&mut Controler::new()); //counter = 3, should return Data(20)

        assert_eq!(fetch_return, (InstructionType::NotBlocked, 20, 0), "Did not successfully fetch");

        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref,&mut Controler::new());
        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref,&mut Controler::new());
        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());

        assert_eq!(fetch_return, (InstructionType::NotBlocked, 21, 1), "Did not successfully fetch");
        

        return;
    }

   pub fn test_decode()  {
        let i1 = instr_fields_to_decimal(0, 20, 1, 2, 0, InstructionType::Memory);
        let i2 = instr_fields_to_decimal(0, 1, 0, 3, 3, InstructionType::ALU);
        let i3 = instr_fields_to_decimal(1, 16, 4, 5, 0, InstructionType::Control);
        create_binary_file("src/programs/fetch-test.bin", &[i1,i2,i3,21,10,9]);
        let mut reg = Registers::new();
        reg.update_gp(1 as usize, 1);
        reg.update_gp(0 as usize, 1);
        let mut cache = Cache::new([[-1; 7]; CACHE_SIZE as usize], [[-1; 4]; MEMORY_SIZE as usize]);
        cache.load_memory_from_file("src/programs/fetch-test.bin".to_string());

        let comp_instr = Instruction {
            type_field: 0,
            instr_type: InstructionType::Memory,
            device: Devices::Decode,
            opcode: 20,
            arg1: 1,
            arg2: 2,
            arg3: 0,
            reg1: 0,
            reg2: 0,
            reg3: 0,
            result: None,
            pc: 0

        };

        let default = Instruction {
            type_field: 0,
            instr_type: InstructionType::Stall,
            device: Devices::Clock,
            opcode:0,
            arg1:0,
            arg2:0,
            arg3:0,
            reg1: 0,
            reg2: 0,
            reg3: 0,
            result:None,
            pc:0
        };
        
        let cache_ref = &mut cache;
        let reg_ref = &mut reg;
        let mut fetch = Fetch::new();
        let mut decode = Decode::new(fetch);

        let mut dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref,&mut Controler::new()); //should start fetch chain
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());

        assert_eq!(dec_return.unwrap_or(default), comp_instr, "Decode did not return or properly decode the instruction");
        //decoding a memory instruction works!

        //test ALU decode
        let comp_instr = Instruction {
            type_field: 0,
            instr_type: InstructionType::ALU,
            device: Devices::Decode,
            opcode: 1,
            arg1: 1,
            arg2: 3,
            arg3: 3,
            reg1: 0,
        reg2: 0,
        reg3: 0,
            result: None,
            pc: 1

        };
        //one less because fetch retuned its first stall on the call that returned above
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());

        assert_eq!(dec_return.unwrap_or(default), comp_instr, "Decode did not return or properly decode the instruction");

        let comp_instr = Instruction {
            type_field: 1,
            instr_type: InstructionType::Control,
            device: Devices::Decode,
            opcode: 16,
            arg1: 0,
            arg2: 0,
            arg3: 0,
            reg1: 0,
        reg2: 0,
        reg3: 0,
            result: None,
            pc: 2

        };

        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());

        assert_eq!(dec_return.unwrap_or(default), comp_instr, "Decode did not return or properly decode the instruction");

        //decodes all three function types!

    }


    pub fn test_excecute() {
        let i1 = instr_fields_to_decimal(0, 20, 1, 2, 16, InstructionType::Memory);
        let i2 = instr_fields_to_decimal(0, 1, 0, 3, 3, InstructionType::ALU);
        let i3 = instr_fields_to_decimal(1, 16, 4, 5, 0, InstructionType::Control);
        create_binary_file("src/programs/fetch-test.bin", &[i1,i2,i3,21,10,9]);
        let mut reg = Registers::new();
        reg.update_gp(1 as usize, 1);
        reg.update_gp(0 as usize, 1);
        let mut cache = Cache::new([[-1; 7]; CACHE_SIZE as usize], [[-1; 4]; MEMORY_SIZE as usize]);
        cache.load_memory_from_file("src/programs/fetch-test.bin".to_string());

        let default = Instruction {
            type_field: 0,
            instr_type: InstructionType::Stall,
            device: Devices::Clock,
            opcode:0,
            arg1:0,
            arg2:0,
            arg3:0,
            reg1: 0,
        reg2: 0,
        reg3: 0,
            result:None,
            pc:0
        };

        let cache_ref = &mut cache;
        let reg_ref = &mut reg;
        let mut fetch = Fetch::new();
        let mut decode = Decode::new(fetch);
        let mut excecute = Execute::new(decode);

        let mut exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref,&mut Controler::new());
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new()); //decode returns here
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new()); //exec returns here, arg1 should be 16
        

        assert_eq!(exec_return.unwrap_or(default).arg1, 17);

        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new()); //decode returns here
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());

        assert_eq!(exec_return.unwrap_or(default).result.unwrap_or(0), 4);

        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new());
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref, &mut Controler::new()); //exec returns here, comparison flag should be 0

        //assert_eq!(cache_ref.registers.get_flags(), 0);

        //execute stage works!




    }

    pub fn test_mem_stage() {
        let i1 = instr_fields_to_decimal(0, 20, 1, 2, 16, InstructionType::Memory);
        let i2 = instr_fields_to_decimal(0, 1, 0, 3, 3, InstructionType::ALU);
        let i3 = instr_fields_to_decimal(1, 16, 4, 5, 0, InstructionType::Control);
        create_binary_file("src/programs/fetch-test.bin", &[i1,i2,i3,21,10,9]);
        let mut reg = Registers::new();
        reg.update_gp(1 as usize, 1);
        reg.update_gp(0 as usize, 1);
        let mut cache = Cache::new([[-1; 7]; CACHE_SIZE as usize], [[-1; 4]; MEMORY_SIZE as usize]);
        cache.load_memory_from_file("src/programs/fetch-test.bin".to_string());

        let default = Instruction {
            type_field: 0,
            instr_type: InstructionType::Stall,
            device: Devices::Clock,
            opcode:0,
            arg1:0,
            arg2:0,
            arg3:0,
            reg1: 0,
        reg2: 0,
        reg3: 0,
            result:None,
            pc:0
        };

        let cache_ref = &mut cache;
        let reg_ref = &mut reg;
        let mut fetch = Fetch::new();
        let mut decode = Decode::new(fetch);
        let mut excecute = Execute::new(decode);
        let mut memory = Memory::new( excecute);

        let mut mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked, &mut Controler::new());
        mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked, &mut Controler::new());
        mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked, &mut Controler::new());
         mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked, &mut Controler::new());
         mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked, &mut Controler::new());
         mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked,&mut Controler::new());
         mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked, &mut Controler::new());
         mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked, &mut Controler::new());
         mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked, &mut Controler::new());
         mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked, &mut Controler::new());
        mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked, &mut Controler::new());
        mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked, &mut Controler::new());
        mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked, &mut Controler::new()); //mem_stage returns value here on the same call that the cache returns a value

        assert_eq!(true, mem_return.unwrap_or(default).result.is_some())


    }

    pub fn test_writeback() {
        let i1 = instr_fields_to_decimal(0, 20, 1, 2, 16, InstructionType::Memory);
        let i2 = instr_fields_to_decimal(0, 1, 0, 3, 3, InstructionType::ALU);
        let i3 = instr_fields_to_decimal(1, 16, 4, 5, 0, InstructionType::Control);
        create_binary_file("src/programs/fetch-test.bin", &[i1,i2,i3,21,10,9]);
        let mut reg = Registers::new();
        reg.update_gp(1 as usize, 1);
        reg.update_gp(0 as usize, 1);
        let mut cache = Cache::new([[-1; 7]; CACHE_SIZE as usize], [[-1; 4]; MEMORY_SIZE as usize]);
        cache.load_memory_from_file("src/programs/fetch-test.bin".to_string());

        let default = Instruction {
            type_field: 0,
            instr_type: InstructionType::Stall,
            device: Devices::Clock,
            opcode:0,
            arg1:0,
            arg2:0,
            arg3:0,
            reg1: 0,
        reg2: 0,
        reg3: 0,
            result:None,
            pc:0
        };

        let cache_ref = &mut cache;
        let reg_ref = &mut reg;
        let mut fetch = Fetch::new();
        let mut decode = Decode::new(fetch);
        let mut excecute = Execute::new(decode);
        let mut memory = Memory::new( excecute);
        let mut writeback = Writeback::new( memory);

        let mut wb_return = writeback.call(reg_ref, cache_ref, &mut Controler::new());
        wb_return = writeback.call(reg_ref, cache_ref, &mut Controler::new());
        wb_return = writeback.call(reg_ref, cache_ref, &mut Controler::new());
        wb_return = writeback.call(reg_ref, cache_ref, &mut Controler::new());
        wb_return = writeback.call(reg_ref, cache_ref, &mut Controler::new());
        wb_return = writeback.call(reg_ref, cache_ref, &mut Controler::new());
        wb_return = writeback.call(reg_ref, cache_ref, &mut Controler::new());
        wb_return = writeback.call(reg_ref, cache_ref,&mut Controler::new());
        wb_return = writeback.call(reg_ref, cache_ref, &mut Controler::new());
        wb_return = writeback.call(reg_ref, cache_ref, &mut Controler::new());
        wb_return = writeback.call(reg_ref, cache_ref, &mut Controler::new());
        wb_return = writeback.call(reg_ref, cache_ref, &mut Controler::new()); //12th memory call returns
        wb_return = writeback.call(reg_ref, cache_ref, &mut Controler::new());
        wb_return = writeback.call(reg_ref, cache_ref, &mut Controler::new()); //should return for real here

        assert_eq!(reg_ref.get_gp(2), -1); //should have stored the -1 from memory in register 2

    }

struct Simulator {
    cache: Cache,
    registers: Registers,
    ctrl: Controler,
    cycles: u32,
    bp: String,
    breakpoints: Vec<u32>,
    filename: String,
    writeback: Writeback,
    texture: Option<egui::TextureHandle>
}

impl Default for Simulator {
    fn default() -> Self {
        let mut cache = Cache::new([[-1; 7]; CACHE_SIZE as usize], [[-1; 4]; MEMORY_SIZE as usize]);
        let mut registers = Registers::new();
        let mut ctrl = Controler::new();
        let filename = String::new();
        let mut bp: String = String::new();
        let mut breakpoints:Vec<u32> = Vec::new();
        let mut fetch_stage = Fetch::new();
        let mut dec_stage = Decode::new(fetch_stage);
        let mut exec_stage = Execute::new(dec_stage);
        let mut mem_stage = Memory::new(exec_stage);
        let mut writeback = Writeback::new(mem_stage);
        let mut cycles: u32 = 0;
        let texture = None;
        let i1 = instr_fields_to_decimal(0, ADD_RI, 1, 1, 1, InstructionType::ALU);
        let i2 = instr_fields_to_decimal(0, ADD_RR, 1, 1, 2, InstructionType::ALU);
        let i3 = instr_fields_to_decimal(0, ADD_RR, 2, 2, 3, InstructionType::ALU);
        let i4 = instr_fields_to_decimal(0, ADD_RI, 4, 20, 4, InstructionType::ALU); //STR addr
        let i5 = instr_fields_to_decimal(0, STR_D, 1, 4, 0, InstructionType::Memory);
        let i6 = instr_fields_to_decimal(0, ADD_RI, 4, 1, 4, InstructionType::ALU);
        let i7 = instr_fields_to_decimal(0, STR_D, 2, 4, 0, InstructionType::Memory);
        let i8 = instr_fields_to_decimal(0, ADD_RI, 4, 1, 4, InstructionType::ALU); 
        let i9 = instr_fields_to_decimal(0, STR_D, 3, 4, 0, InstructionType::Memory); 
        let i10 = instr_fields_to_decimal(0, PSH_LR, 0, 0, 0, InstructionType::Memory); //put addr 12 into R1
        let i11 = instr_fields_to_decimal(0, POP_LR, 0, 0, 0, InstructionType::Memory); //less than
        let i12 = instr_fields_to_decimal(0,STR_I,1,4,0, InstructionType::Memory); //should jump to immidate 4
        let i13 = instr_fields_to_decimal(0, HALT, 0, 0, 0, InstructionType::Control);
        create_binary_file("src/programs/timing-test.bin", &[i1,i2,i3,i4,i5,i6,i7,i8,i9,i10,i11,i12,i13,0,0,0,0,0,0,0]);
        
        
        Self {
            cache,
            registers,
            ctrl,
            cycles,
            bp,
            breakpoints,
            filename,
            writeback,
            texture
        }
    }
}


fn main() -> eframe::Result{
     let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1920.0, 1080.0]),
        ..Default::default()
    };
    eframe::run_native(
        "DOOMed Simulator",
        options,
        Box::new(|cc| {
            Ok(Box::<Simulator>::default())
        }),
    );
    Ok(())
}

impl eframe::App for Simulator {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("DOOMed ISA Simulator");
            ui.horizontal(|ui| {
                let path = ui.label("File path: ");
                let mut wb_ret: Option<Instruction> = None;
                ui.text_edit_singleline(&mut self.filename);
                ui.button("Load").clicked().then(|| {self.cache.load_memory_from_file(self.filename.clone());});
                ui.button("Run").clicked().then(|| {
                    while wb_ret.is_none() || (wb_ret.is_some() && wb_ret.unwrap().instr_type != InstructionType::NOOP) {
                        wb_ret = self.writeback.call(&mut self.registers, &mut self.cache, &mut self.ctrl);
                        let val = self.registers.reg[32] as u32;
                        if let Some(pos) = self.breakpoints.iter().position(|&x| x == val) {
                            self.breakpoints.remove(pos);
                            break;
                        }
                        self.cycles += 1;
                    }
                });
                ui.button("Step Cycle").clicked().then(|| {
                    wb_ret = self.writeback.call(&mut self.registers, &mut self.cache, &mut self.ctrl);
                    self.cycles += 1;
                });
                ui.button("Set Breakpoint").clicked().then(|| {self.breakpoints.push(self.bp.parse::<u32>().expect("Not a breakpoint!"));});
                ui.text_edit_singleline(&mut self.bp);
                ui.checkbox(&mut self.cache.on, "Cache");
                ui.checkbox(&mut self.ctrl.on, "Control");
                ui.label(format!("Cycle Count: {}", self.cycles));
            });

            ui.separator();
            
            ui.columns(3, |columns| { 
                columns[0].label("Registers");
                ScrollArea::vertical().id_salt("first area").show(&mut columns[0], |ui| {
                    TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .id_salt("first")
                    .column(Column::auto())
                    .column(Column::remainder())
                    .header(20.0, |mut header| {
                        header.col(|ui| { ui.label("Register"); });
                        header.col(|ui| { ui.label("Value"); });
                    })
                    .body(|mut body| {
                        for i in 0..35 {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    let label_text = if i == 32 {
                                        "PC".to_string()
                                    } else if i == 33 {
                                        "SP".to_string()
                                    } else if i == 34 {
                                        "LR".to_string()
                                    } else {
                                        format!("R{}", i)
                                    };
                                    ui.label(label_text);
                                });
                                row.col(|ui| { 
                                    ui.label(self.registers.reg[i].to_string()); 
                                });
                            });
                        }
                    });
                });
                columns[1].label("Memory");
                ScrollArea::vertical().id_salt("second area").show(&mut columns[1], |ui| {
                    TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .id_salt("second")
                    .column(Column::auto())
                    .column(Column::remainder())
                    .header(20.0, |mut header| {
                        header.col(|ui| { ui.label("Address"); });
                        header.col(|ui| { ui.label("Value"); });
                    })
                    .body(|mut body| {
                        for i in 0..1000 {
                            body.row(20.0, |mut row| {
                                row.col(|ui| { 
                                    ui.label(format!("0x{i:X}")); 
                                });
                                row.col(|ui| { ui.label(format!("{:?}", self.cache.main_memory[i])); });
                            });
                        }
                    });
                });

                columns[2].vertical(|ui| {
                    let available_height = ui.available_height();
                    let third_height = available_height / 3.0;

                    ui.label("Cache");
                    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), third_height),
                    egui::Layout::top_down(egui::Align::Min),
        |ui| {
                    ScrollArea::vertical().id_salt("third area").show(ui, |ui| {
                    TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .id_salt("third")
                    .column(Column::auto())
                    .column(Column::remainder())
                    .header(20.0, |mut header| {
                        header.col(|ui| { ui.label("Line"); });
                        header.col(|ui| { ui.label("Value"); });
                    })
                    .body(|mut body| {
                        for i in 0..250 {
                            body.row(20.0, |mut row| {
                                row.col(|ui| { ui.label(format!("0x{i:X}")); });
                                row.col(|ui| { ui.label(format!("{:?}", self.cache.data[i])); });
                            });
                        }
                    });
                    
                    }
                    
                );
                },
                );
                    ui.separator();
                    
                    
                    ui.label("Pipeline");

                    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), third_height),
                    egui::Layout::top_down(egui::Align::Min),
        |ui| {
                    ScrollArea::vertical().id_salt("fourth area").show(ui, |ui| {
                    TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .id_salt("fourth")
                    .column(Column::auto())
                    .column(Column::remainder())
                    .header(20.0, |mut header| {
                        header.col(|ui| { ui.label("Stage"); });
                        header.col(|ui| { ui.label("Value"); });
                    })
                    .body(|mut body| {
                        for i in 0..5 {
                            body.row(20.0, |mut row| {
                                row.col(|ui| { ui.label(vec!["Fetch", "Decode", "Execute", "Memory", "Writeback"].get(i).expect("Stage found").to_string()); });
                                row.col(|ui| {
                                    let text = match i {
                                        0 => format!("{:?}", self.writeback.mem_stage.exec_stage.dec_stage.fetch_stage.state()),
                                        1 => format!("{}", self.writeback.mem_stage.exec_stage.dec_stage.state().1),
                                        2 => format!("{}", self.writeback.mem_stage.exec_stage.state()),
                                        3 => format!("{}", self.writeback.mem_stage.state()),
                                        4 => format!("{}", self.writeback.state()),
                                        _ => String::new(),
                                    };

                                    ui.label(text);
                                });
                            });
                        }
                    });
                    });
                    },
                );

                ui.separator();

                ui.label("Framebuffer");
                // Framebuffer display
                ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), third_height),
                    egui::Layout::top_down(egui::Align::Min),
        |ui| {
                    let frame = self.cache.main_memory[(self.cache.main_memory.len() - (GRAPHICS_OFFSET / 4) as usize)..].into_iter().flatten().copied().collect::<Vec<_>>();
                    let mut pixels: Vec<Color32> = Vec::new();
                    for pixel in frame {
                        pixels.push(Color32::from_rgb(pixel.to_le_bytes()[0], pixel.to_le_bytes()[1], pixel.to_le_bytes()[2]));
                    }
                    let mut img = ColorImage::new([320, 240], pixels);
                    let texture: &egui::TextureHandle = self.texture.get_or_insert_with(|| {
                        ui.ctx().load_texture("framebuffer", img, Default::default())
                    });

                    ui.image((texture.id(), texture.size_vec2()));
                });
                });
            });
        });
    }
}