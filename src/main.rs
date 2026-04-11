//pub mod instruction;
//pub mod memory;
//use doomed_isa::instruction::*;
//use doomed_isa::memory::*;

use std::{collections::btree_map::Range, io, ptr::null};



use doomed_isa::opcode::opcode::RS_RR;

use crate::{instruction::instruction::{Devices, Instruction, InstructionType}, 
memory::memory::{Cache, Registers, ReturnVal}, 
opcode::opcode::{ADD_RI, AND_RI, CMP_RI, CMP_RR, DIV_RI, JL_D, LDR_D, LS_RI, LSL_RI, LSR_RI, MOD_RI, MUL_RI, OR_RI, RS_RI, STR_D, SUB_RI, XOR_RI}};
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
const IMMEDIATE2_SHIFT: u32 = 8;
const POST_REG_SHIFT: u32 = 3;
const IMMEDIATE3_SHIFT: u32 = 3;

const TYPE_MASK: u32 = 0b1;
const OPCODE_MASK: u32 = 0b11_1111;
const REG_MASK: u32 = 0b1_1111;
const IMMEDIATE_MASK: u32 = 0b1111_1111_1111;

fn main() {

    let type_field:u32 = 0;
    let opcode = ADD_RI;
    let reg1 = 3;
    let reg2 = 0;
    let reg3 = 3;
    let imm2: u32 = 1;
    let imm3: u32 = 0;

     let instr_binary = (type_field << TYPE_SHIFT)
                | (opcode << OPCODE_SHIFT)
                | (reg1 << REG1_SHIFT)
                | (imm2 << IMMEDIATE2_SHIFT)
                | (reg3 << POST_REG_SHIFT);

    println!("{}", instr_binary.to_string());

    let type_field = ((instr_binary >> TYPE_SHIFT) & TYPE_MASK) as u8;
    let opcode     = ((instr_binary >> OPCODE_SHIFT) & OPCODE_MASK) as u8;
    let reg1       = ((instr_binary >> REG1_SHIFT) & REG_MASK) as u8;
    let imm2       = ((instr_binary >> IMMEDIATE2_SHIFT) & IMMEDIATE_MASK) as u8;
    let reg3       = ((instr_binary >> POST_REG_SHIFT) & REG_MASK) as u8;

    println!("{}, {}, {}, {}, {}", type_field, opcode, reg1, imm2, reg3);

    //test_memory();

    //test_fetch();

    //test_decode();

    //test_excecute();
    
    //test_mem_stage();

    //test_writeback();

    test_control_flow();
    
    
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
    let mut cache = Cache::new([[-1; 4]; 4], [-1; 64]);
    cache.load_memory_from_file("src/programs/fetch-test.bin".to_string());

    let cache_ref = &mut cache;
    let reg_ref = &mut reg;
    let mut fetch = Fetch::new();
    let mut decode = Decode::new(fetch);
    let mut excecute = Execute::new(decode);
    let mut memory = Memory::new( excecute);
    let mut writeback = Writeback::new( memory);
    
    let mut wb_ret = writeback.call(reg_ref, cache_ref);
    
    while wb_ret.is_none() || (wb_ret.is_some() && wb_ret.unwrap().instr_type != InstructionType::NOOP) {
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
        wb_ret = writeback.call(reg_ref, cache_ref);
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
        else { //some jump
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
            result: None,
            pc: 0
        };

        let mut data: ReturnVal = ReturnVal::Wait(true);
        let reg = Registers::new();

        let mut cache = Cache::new([[-1; 4]; 4], [-1; 64]);

        data = cache.call(instr);
        data = cache.call(instr); //should take 1, 2, 3 calls to store due to write through
        data = cache.call(instr);
        data = cache.call(instr);

        assert_eq!(data, ReturnVal::Data(5), "Value 5 was not succesfully stored in index 1!");

        println!("{}", cache.get_cache(5).to_string());
        println!("{}", cache.get_memory(5).to_string());

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

        println!("{}", cache.get_cache(6).to_string());
        println!("{}", cache.get_memory(6).to_string());
        
        instr.arg2 = 7;
        instr.arg1 = 7;
        
        data = cache.call(instr);
        data = cache.call(instr);
        data = cache.call(instr);
        data = cache.call(instr);
        
        assert_eq!(data, ReturnVal::Data(7), "Value 7 was not successfuly stored in index 2!");

        println!("{}", cache.get_cache(7).to_string());
        println!("{}", cache.get_memory(7).to_string());

        //STR Works!

        let mut ldr = Instruction {
            instr_type: InstructionType::Memory,
            device: Devices::Clock,
            type_field: 0,
            opcode: 20,
            arg1: 5,
            arg2: 0,
            arg3: 0,
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

        assert_eq!(cache.get_memory(0), 0);
        assert_eq!(cache.get_memory(1), 1);
        assert_eq!(cache.get_memory(2), 2);
        assert_eq!(cache.get_memory(3), 3);


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
        let mut cache = Cache::new([[-1; 4]; 4], [-1; 64]);
        cache.load_memory_from_file("src/programs/fetch-test.bin".to_string());

        let cache_ref = &mut cache;
        let mut fetch = Fetch::new();
        let mut fetch_return: (InstructionType, i32, i32);
        let reg_ref = &mut reg;
        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref); //makes first call to cache

        assert_eq!(fetch_return, (InstructionType::Stall, -1, -1), "Returned something wrong on first call");

        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref); //counter = 1
        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref); //counter = 2
        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref); //counter = 3, should return Data(20)

        assert_eq!(fetch_return, (InstructionType::NotBlocked, 20, 0), "Did not successfully fetch");

        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        fetch_return = fetch.call(InstructionType::NotBlocked, reg_ref, cache_ref);

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
        let mut cache = Cache::new([[-1; 4]; 4], [-1; 64]);
        cache.load_memory_from_file("src/programs/fetch-test.bin".to_string());

        let comp_instr = Instruction {
            type_field: 0,
            instr_type: InstructionType::Memory,
            device: Devices::Decode,
            opcode: 20,
            arg1: 1,
            arg2: 2,
            arg3: 0,
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
            result:None,
            pc:0
        };
        
        let cache_ref = &mut cache;
        let reg_ref = &mut reg;
        let mut fetch = Fetch::new();
        let mut decode = Decode::new(fetch);

        let mut dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref); //should start fetch chain
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref);

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
            result: None,
            pc: 1

        };
        //one less because fetch retuned its first stall on the call that returned above
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref);

        assert_eq!(dec_return.unwrap_or(default), comp_instr, "Decode did not return or properly decode the instruction");

        let comp_instr = Instruction {
            type_field: 1,
            instr_type: InstructionType::Control,
            device: Devices::Decode,
            opcode: 16,
            arg1: 0,
            arg2: 0,
            arg3: 0,
            result: None,
            pc: 2

        };

        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        dec_return = decode.call(InstructionType::NotBlocked, reg_ref, cache_ref);

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
        let mut cache = Cache::new([[-1; 4]; 4], [-1; 64]);
        cache.load_memory_from_file("src/programs/fetch-test.bin".to_string());

        let default = Instruction {
            type_field: 0,
            instr_type: InstructionType::Stall,
            device: Devices::Clock,
            opcode:0,
            arg1:0,
            arg2:0,
            arg3:0,
            result:None,
            pc:0
        };

        let cache_ref = &mut cache;
        let reg_ref = &mut reg;
        let mut fetch = Fetch::new();
        let mut decode = Decode::new(fetch);
        let mut excecute = Execute::new(decode);

        let mut exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref); //decode returns here
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref); //exec returns here, arg1 should be 16
        

        assert_eq!(exec_return.unwrap_or(default).arg1, 17);

        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref); //decode returns here
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref);

        assert_eq!(exec_return.unwrap_or(default).result.unwrap_or(0), 4);

        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref);
        exec_return = excecute.call(InstructionType::NotBlocked, reg_ref, cache_ref); //exec returns here, comparison flag should be 0

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
        let mut cache = Cache::new([[-1; 4]; 4], [-1; 64]);
        cache.load_memory_from_file("src/programs/fetch-test.bin".to_string());

        let default = Instruction {
            type_field: 0,
            instr_type: InstructionType::Stall,
            device: Devices::Clock,
            opcode:0,
            arg1:0,
            arg2:0,
            arg3:0,
            result:None,
            pc:0
        };

        let cache_ref = &mut cache;
        let reg_ref = &mut reg;
        let mut fetch = Fetch::new();
        let mut decode = Decode::new(fetch);
        let mut excecute = Execute::new(decode);
        let mut memory = Memory::new( excecute);

        let mut mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked);
        mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked);
        mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked);
         mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked);
         mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked);
         mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked);
         mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked);
         mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked);
         mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked);
         mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked);
        mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked);
        mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked);
        mem_return = memory.call(reg_ref, cache_ref, InstructionType::NotBlocked); //mem_stage returns value here on the same call that the cache returns a value

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
        let mut cache = Cache::new([[-1; 4]; 4], [-1; 64]);
        cache.load_memory_from_file("src/programs/fetch-test.bin".to_string());

        let default = Instruction {
            type_field: 0,
            instr_type: InstructionType::Stall,
            device: Devices::Clock,
            opcode:0,
            arg1:0,
            arg2:0,
            arg3:0,
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

        let mut wb_return = writeback.call(reg_ref, cache_ref);
        wb_return = writeback.call(reg_ref, cache_ref);
        wb_return = writeback.call(reg_ref, cache_ref);
        wb_return = writeback.call(reg_ref, cache_ref);
        wb_return = writeback.call(reg_ref, cache_ref);
        wb_return = writeback.call(reg_ref, cache_ref);
        wb_return = writeback.call(reg_ref, cache_ref);
        wb_return = writeback.call(reg_ref, cache_ref);
        wb_return = writeback.call(reg_ref, cache_ref);
        wb_return = writeback.call(reg_ref, cache_ref);
        wb_return = writeback.call(reg_ref, cache_ref);
        wb_return = writeback.call(reg_ref, cache_ref); //12th memory call returns
        wb_return = writeback.call(reg_ref, cache_ref);
        wb_return = writeback.call(reg_ref, cache_ref); //should return for real here

        assert_eq!(reg_ref.get_gp(2), -1); //should have stored the -1 from memory in register 2

    }