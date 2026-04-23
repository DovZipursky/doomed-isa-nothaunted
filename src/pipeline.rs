pub mod pipeline {
    use doomed_isa::Memory::memory::GRAPHICS_OFFSET;

    use crate::{instruction::{self, instruction::{Devices, Instruction, InstructionType}}, memory::{self, memory::{Cache, MEMORY_SIZE, Registers, ReturnVal}}, opcode::opcode::{ADD_RI, ADD_RR, CMP_RI, CMP_RR, FDR, FTR, GDR_D, GDR_I, GDR_PC, GTR_D, GTR_I, GTR_PC, HALT, JE_D, JE_I, JE_PC, JG_D, JG_I, JG_PC, JL_D, JL_I, JL_PC, JMP_D, JMP_I, JMP_PC, LDR_D, LDR_I, LDR_PC, POP, PSH, RET, STR_D, STR_I, STR_PC}};
    use std::cell::RefCell;
    use crate::opcode::opcode;

//constants that define the range of opcodes that refer to different instruction types
const ALU_RANGE: [i32; 2] = [1, 25];
const CONTROL_RANGE: [i32; 2] = [26, 40];
const MEMORY_RANGE: [i32; 2] = [41, 59];

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


    pub struct Fetch {
        load_instruction: Option<Instruction>,
        cur_instruction: Option<i32>,
        cur_pc: i32
    }
    pub struct Decode  {
        instruction: Option<(InstructionType, i32, i32)>,
        dec_instruction: Option<Instruction>,
        next_instruction: Option<(i32, i32)>,
        pub fetch_stage: Fetch,
    }
    pub struct Execute  {
        instruction: Option<Instruction>,
        pub dec_stage: Decode,
    }
    pub struct Memory  {
        instruction: Option<Instruction>,
        call_instr: Option<Instruction>,
        pub exec_stage: Execute
    }
    pub struct Writeback  {
        instruction: Option<Instruction>,
        pub mem_stage: Memory
    }

    pub struct Controler {
        pub on: bool,
        processing: i32
    }

    impl Controler {
        pub fn new() -> Self {
            Controler {
                on: true,
                processing: 0
            }
        }

        pub fn switch(&mut self, val: bool) {
            self.on = val;
        }

        pub fn inc(&mut self) {
            self.processing = self.processing + 1;
        }

        pub fn dec(&mut self) {
            self.processing = self.processing - 1;
        }

        pub fn status(&self) -> bool {
            return self.on;
        }

        pub fn in_process(&self) -> i32 {
            return self.processing;
        }


    }

    impl Writeback {
        pub fn new(mem_stage: Memory) -> Self {
            Writeback {
                instruction: None,
                mem_stage: mem_stage
            }
        }

        pub fn call(&mut self, reg: &mut Registers, cache: &mut Cache, ctrl: &mut Controler) -> Option<Instruction>{
            let mut wb_status = InstructionType::NotBlocked;
            //if there is some instruction that is not NOOP, Stalled, or Squashed:
            if self.instruction.is_some()  && (self.instruction.unwrap().instr_type !=  InstructionType::NOOP ||self.instruction.unwrap().instr_type !=  InstructionType::Squashed || self.instruction.unwrap().instr_type !=  InstructionType::Stall) {
                
                let instr = self.instruction.unwrap();
                //if there is a result and it is not a control flow instruction
                if instr.result.is_some() && instr.instr_type == InstructionType::Memory {
                    //assume arg2 is a destination if LDR
                    if instr.opcode == LDR_D as i32 || instr.opcode == LDR_I as i32 || instr.opcode == LDR_PC as i32 || instr.opcode == GDR_D as i32 || instr.opcode == GDR_I as i32 { 
                        reg.update_gp(instr.reg2 as usize, instr.result.unwrap());
                        reg.update_pending(instr.reg2 as usize, false); 
                    }
                    else if instr.opcode == STR_D as i32 || instr.opcode == STR_I as i32 || instr.opcode == STR_PC as i32 || instr.opcode == GTR_D as i32 || instr.opcode == GTR_I as i32 {
                        //with STR registers are not written to, but update src as not pending
                        reg.update_pending(instr.reg1 as usize, false);
                    }
                    else if instr.opcode == PSH as i32 {
                        let new_stack = reg.get_gp(33) - 1; //psh stores at sp, so now decrement to new empty spot
                        reg.update_gp(33, new_stack);
                        reg.update_pending(instr.reg1 as usize, false);

                    }
                    else if instr.opcode == POP as i32 {
                        let new_stack = reg.get_gp(33) + 1; //pop loads from sp, raise stack back up
                        reg.update_gp(33, new_stack);
                        reg.update_gp(instr.reg2 as usize, instr.result.unwrap());
                        reg.update_pending(instr.reg2 as usize, false); 
                    }

                   
                    //write to register and update to no longer pending

                    if !ctrl.status() {
                        ctrl.dec();
                    }
                    
                    
                }
                else if instr.result.is_some() && instr.instr_type == InstructionType::ALU {
                    //assume arg3 is a destination
                    
                    reg.update_gp(instr.reg3 as usize, instr.result.unwrap());
                    
                    reg.update_pending(instr.reg3 as usize, false); 

                    if !ctrl.status() {
                        ctrl.dec();
                    }
                }   

                else if instr.instr_type == InstructionType::Control { //if it is a control flow instruction
                    //assume result is the update to PC
                    if instr.opcode == JMP_D as i32  || instr.opcode == JMP_I as i32 || instr.opcode == JMP_PC as i32  { //JMP
                        reg.update_pending(34, false); //unpend linked register
                        reg.update_gp(34,instr.pc + 1); //update link register before jump
                        reg.update_gp(32, instr.result.unwrap()); //update pc to destination
                        wb_status = InstructionType::Squashed;
                    }
                    else if instr.opcode == JL_D as i32 || instr.opcode == JL_I as i32 || instr.opcode == JL_PC as i32 || instr.opcode == RET as i32 
                    || instr.opcode == JE_D as i32 || instr.opcode == JE_I as i32 || instr.opcode == JE_PC as i32 
                    || instr.opcode == JG_D as i32 || instr.opcode == JG_I as i32 || instr.opcode == JG_PC as i32 { //conditionals and RET
                          
                        if instr.result.unwrap() != instr.pc { //if jump acutally jumped 
                            reg.update_gp(32, instr.result.unwrap());
                            wb_status = InstructionType::Squashed;
                        }
                        
                    }
                    else if instr.opcode == HALT as i32 {
                        wb_status = InstructionType::Blocked;
                    }
                    if !ctrl.status() {
                        ctrl.dec();
                    }
 
                }

            }
            println!("Writeback status: {}", wb_status.to_string());
            let next_instr = self.mem_stage.call(reg, cache, wb_status, ctrl);

            let ret_instr = self.instruction;
            self.instruction = next_instr;

            return ret_instr;


        }

        pub fn state(&self) -> String {
             let instr: String;
            if self.instruction.is_some() {
                instr = self.instruction.unwrap().formatted_to_string();
            }
            else {
                instr = "None".to_owned();
            }

            return instr;
        }
    }

    impl Memory {
        pub fn new(exec_stage: Execute) -> Self {
            Memory {
                instruction: None,
                call_instr: None,
                exec_stage: exec_stage
            }
        }

        pub fn call(&mut self, reg: &mut Registers, cache: &mut Cache, wb_status: InstructionType, ctrl: &mut Controler) -> Option<Instruction>{
            let mut mem_status: InstructionType = InstructionType::Blocked;
            let mut result: ReturnVal = ReturnVal::Wait(true);
            
            if wb_status == InstructionType::Squashed { //if squashing, set own instruction to squashed and pass on squashed to exec
                self.instruction = Some(Instruction {
                                        instr_type: InstructionType::Squashed,
                                        device: Devices::Memory,
                                        type_field: -1,
                                        opcode: -1,
                                        arg1: -1,
                                        arg2: -1,
                                        arg3: -1,
                                        reg1: 0,
                                        reg2: 0,
                                        reg3: 0,
                                        result: None,
                                        pc: -1
                    });
                mem_status = InstructionType::Squashed;
                let maybe_next_instr = self.exec_stage.call(mem_status, reg, cache, ctrl);
                return self.instruction;

            }

            if let Some(instr) = self.instruction {
               let mod_instr = self.instruction.as_mut();
                if instr.instr_type == InstructionType::Memory {
                    result = cache.call(instr);
                    match result {
                        ReturnVal::Data(i) => {
                            
                            mod_instr.unwrap().result.replace(i);
                            mem_status = InstructionType::NotBlocked; //with not blocked
                        }

                        ReturnVal::Wait(j) => {
                           mem_status = InstructionType::Blocked;
                        }
                    }

                }
                else {
                    mem_status = InstructionType::NotBlocked;
                }
            }
            else {
                mem_status = InstructionType::NotBlocked;
            }
            println!("Memory status: {}", mem_status.to_string());
            let maybe_next_instr = self.exec_stage.call(mem_status, reg, cache, ctrl);

            
            let ret_instr = self.instruction;
            if ret_instr.is_none() || (ret_instr.is_some() && (ret_instr.unwrap().instr_type != InstructionType::Memory || ret_instr.unwrap().result.is_some())) { 
                    //if not a memeory instruction, or result has been gotten for a memory instruction:
                    self.instruction = maybe_next_instr; 
                    if self.instruction.is_some() {
                        println!("Memory recieved an instruction from decode with pc value {}", self.instruction.unwrap().pc.to_string());
                    }
                    return ret_instr;
            }

            else  { //if a memory instruction without a result:
                //exec will not have returned any instruction bc it was called with blocked
                //return stall without doing anything to self.instruction

                        return Some(Instruction {
                                        instr_type: InstructionType::Stall,
                                        device: Devices::Decode,
                                        type_field: -1,
                                        opcode: -1,
                                        arg1: -1,
                                        arg2: -1,
                                        arg3: -1,
                                        reg1: 0,
                                        reg2: 0,
                                        reg3: 0,
                                        result: None,
                                        pc: -1
                    });
            }




        }

        

        pub fn state(&self) -> String {
            let instr: String;
            if self.instruction.is_some() {
                instr = self.instruction.unwrap().formatted_to_string();
            }
            else {
                instr = "None".to_owned();
            }

            return instr;
        }

    }

    impl Execute {
        pub fn new(dec_stage: Decode) -> Self {
            Execute {
                instruction: None,
                dec_stage: dec_stage,
            }
        }

        pub fn call(&mut self, mem_status: InstructionType, reg: &mut Registers, cache: &mut Cache, ctrl: &mut Controler) -> Option<Instruction> {
            if mem_status == InstructionType::Squashed {
                self.instruction = Some(Instruction {
                                        instr_type: InstructionType::Squashed,
                                        device: Devices::Memory,
                                        type_field: -1,
                                        opcode: -1,
                                        arg1: -1,
                                        arg2: -1,
                                        arg3: -1,
                                        reg1: 0,
                                        reg2: 0,
                                        reg3: 0,
                                        result: None,
                                        pc: -1
                    });
                let dec_ret = self.dec_stage.call(mem_status, reg, cache, ctrl);
                return self.instruction;
            }
            if let Some(instr) = self.instruction.as_mut() {
                if instr.instr_type == InstructionType::ALU {
                    if instr.opcode == ADD_RI as i32  { //if ADD, use values provided by decode
                        instr.result.replace( instr.arg1 +  instr.arg2);
                        
                    }
                    else if instr.opcode == ADD_RR as i32 {
                        instr.result.replace( instr.arg1 +  instr.arg2);
                    }
                }
                else if  instr.instr_type == InstructionType::Control {
                    if  instr.opcode == JMP_D as i32 || instr.opcode == JMP_I as i32 || instr.opcode == JMP_PC as i32 { //JMP
                        instr.result.replace( instr.arg1 + instr.arg2); //src + offset
                    }
                    else if instr.opcode == JL_D as i32 || instr.opcode == JL_I as i32 || instr.opcode == JL_PC as i32 {//JL
                        if reg.get_flags() == -1 {
                            if instr.type_field == 1 {
                                instr.result.replace(instr.arg1 + reg.get_gp(instr.reg2 as usize));
                            }
                            else {
                                instr.result.replace(instr.arg1);
                            }
                        
                        }
                        else {
                            instr.result.replace(instr.pc);
                        }

                    }
                    else if instr.opcode == JE_D as i32 || instr.opcode == JE_I as i32 || instr.opcode == JE_PC as i32 {//JL
                        if reg.get_flags() == 0 {
                            if instr.type_field == 1 {
                                instr.result.replace(instr.arg1 + reg.get_gp(instr.reg2 as usize));
                            }
                            else {
                                instr.result.replace(instr.arg1);
                            }
                        }
                        else {
                            instr.result.replace(instr.pc);
                        }

                    }
                    else if instr.opcode == JG_D as i32 || instr.opcode == JG_I as i32 || instr.opcode == JG_PC as i32 {//JL
                        if reg.get_flags() == 1 {
                            if instr.type_field == 1 {
                                instr.result.replace(instr.arg1 + reg.get_gp(instr.reg2 as usize));
                            }
                            else {
                                instr.result.replace(instr.arg1);
                            }
                        }
                        else {
                            instr.result.replace(instr.pc);
                        }

                    }
                    
                    else if instr.opcode == RET as i32 {
                        instr.result.replace(reg.get_gp(34)); //return address
                    }

                    else if  instr.opcode == CMP_RI as i32 || instr.opcode == CMP_RR as i32 { //CMP
                        if  instr.arg1 >  instr.arg2 {
                            reg.update_flags(1);
                        }
                        else if instr.arg1 < instr.arg2 {
                            reg.update_flags(-1);
                        }
                        else if instr.arg1 == instr.arg2 {
                            reg.update_flags(0);
                        }
                        else {
                            reg.update_flags(-5);
                        }

                    }
                }

                else if  instr.instr_type == InstructionType::Memory {
                    //calculate address, if LDR arg1 = arg1 + arg3, if str arg2 = arg2 + arg3
                    if  instr.opcode == LDR_D as i32 || instr.opcode == LDR_I as i32 || instr.opcode == LDR_PC as i32 { //if LDR
                       instr.arg1 = instr.arg1 + instr.arg3;
                    }
                    else if instr.opcode == STR_D as i32 || instr.opcode == STR_I as i32 || instr.opcode == STR_PC as i32 {
                        instr.arg2 = instr.arg2 + instr.arg3;
                    }
                    else if instr.opcode == GDR_D as i32 || instr.opcode == GDR_I as i32 || instr.opcode == GDR_PC as i32 {
                        let graphics_addr = (instr.arg1 + GRAPHICS_OFFSET) % (MEMORY_SIZE * 4); //mod memory size to prevent overflow
                        instr.arg1 = graphics_addr + instr.arg3;

                    }

                    else if instr.opcode == GTR_D as i32 || instr.opcode == GTR_I as i32 || instr.opcode == GTR_PC as i32 {
                        let graphics_addr = (instr.arg2 + GRAPHICS_OFFSET) % (MEMORY_SIZE * 4);
                        instr.arg2 = graphics_addr + instr.arg3;
                    }
                    else if instr.opcode == PSH as i32 {
                        instr.arg2 = reg.get_gp(instr.reg2 as usize) - 1; //for psh, stack is dst so store there + 1, WB makes new pointer official later
                    }
                    else if instr.opcode == POP as i32 {
                        instr.arg1 = reg.get_gp(instr.reg1 as usize); //for pop, stack is src so load from there
                    }
                    else {
                        //whatever
                    }

                }




            }
            println!("Execute status: {}", mem_status.to_string());
            let next = self.dec_stage.call(mem_status, reg, cache, ctrl);
            let cur = self.instruction;
            if next.is_some() {
                println!("Execute recieved an instruction from Decode with pc value {}", next.unwrap().pc.to_string());
                self.instruction = next;
            }
            if mem_status != InstructionType::Blocked && cur.is_some() {
                self.instruction = next;
                return cur;
            }
            else if mem_status != InstructionType::Blocked && cur.is_none() {
                return Some(Instruction {
                                    instr_type: InstructionType::Stall,
                                    device: Devices::Decode,
                                    type_field: -1,
                                    opcode: -1,
                                    arg1: -1,
                                    arg2: -1,
                                    arg3: -1,
                                    reg1: 0,
                                    reg2: 0,
                                    reg3: 0,
                                    result: None,
                                    pc: -1
                });
            }

            else {
                return Some(Instruction {
                                    instr_type: InstructionType::Stall,
                                    device: Devices::Decode,
                                    type_field: -1,
                                    opcode: -1,
                                    arg1: -1,
                                    arg2: -1,
                                    arg3: -1,
                                    reg1: 0,
                                    reg2: 0,
                                    reg3: 0,
                                    result: None,
                                    pc: -1
                });
            }



            //return None;
        }

        pub fn state(&self) -> String {
            let instr: String;
            if self.instruction.is_some() {
                instr = self.instruction.unwrap().formatted_to_string();
            }
            else {
                instr = "None".to_owned();
            }

            return instr;
        }
    }

    impl Decode {
        pub fn new(fetch_stage: Fetch) -> Self {
            Decode {
                instruction: None,
                fetch_stage: fetch_stage,
                next_instruction: None,
                dec_instruction: None
            }
        }

        pub fn call(&mut self, exec_status: InstructionType, reg: &mut Registers, cache: &mut Cache, ctrl: &mut Controler) -> Option<Instruction> {
            let mut dec_status = exec_status;
            if dec_status == InstructionType::Squashed {
                self.dec_instruction = Some(Instruction {
                                        instr_type: InstructionType::Squashed,
                                        device: Devices::Memory,
                                        type_field: -1,
                                        opcode: -1,
                                        arg1: -1,
                                        arg2: -1,
                                        arg3: -1,
                                        reg1: 0,
                                        reg2: 0,
                                        reg3: 0,
                                        result: None,
                                        pc: -1
                    });
                self.instruction = None;
                let _fetch_ret = self.fetch_stage.call(dec_status, reg, cache, ctrl);
                return self.dec_instruction;
            }
            
            if self.instruction.is_some() && self.dec_instruction.is_none() { 
                let (fetch_type, instr_binary, pc) = self.instruction.unwrap();
                //self.next_instruction = None;
                let type_field = ((instr_binary as u32) >> TYPE_SHIFT) & TYPE_MASK; //most sig bit
                let opcode = (((instr_binary as u32) >> OPCODE_SHIFT)) & OPCODE_MASK; //next 5 bits

                let arg1: i32;
                let arg2: i32;
                let arg3: i32;
                let instr_type: InstructionType;
                let result: Option<i32> = None; 
                let instruction: Instruction;
                
                

                if fetch_type == InstructionType::Stall {
                    instruction = Instruction {
                        instr_type: InstructionType::Stall,
                        device: Devices::Decode,
                        type_field: -1,
                        opcode: -1,
                        arg1: -1,
                        arg2: -1,
                        arg3: -1,
                        reg1: 0,
                        reg2: 0,
                        reg3: 0,
                        result: None,
                        pc: -1
                    };
                    self.instruction = None;
                    self.dec_instruction = Some(instruction);
                }
                else if opcode == 0 { //HALT
                    instruction = Instruction {
                        instr_type: InstructionType::Control,
                        device: Devices::Decode,
                        type_field: -1,
                        opcode: opcode as i32,
                        arg1: -1,
                        arg2: -1,
                        arg3: -1,
                        reg1: 0,
                        reg2: 0,
                        reg3: 0,
                        result: None,
                        pc: pc
                    };
                    self.instruction = None;
                    self.dec_instruction = Some(instruction);
                }

                else if opcode >= ALU_RANGE[0] as u32 && opcode <= ALU_RANGE[1] as u32 { //ALU section
                    instr_type = InstructionType::ALU;
                    if opcode == ADD_RI { //assume reg + immediate to dst arg3
                        arg1 = (((instr_binary as u32) >> REG1_SHIFT) & REG_MASK) as i32;
                        arg2 = (((instr_binary as u32) >> IMMEDIATE2_SHIFT) & IMMEDIATE_MASK) as i32;
                        arg3 = (((instr_binary as u32) >> POST_REG_SHIFT) & REG_MASK) as i32; //no shift since whole structure is taken up

                        if !reg.is_pending(arg1 as usize) {
                            instruction = Instruction {
                                instr_type: instr_type,
                                device: Devices::Decode,
                                type_field: type_field as i32,
                                opcode: opcode as i32,
                                arg1: arg1,
                                arg2: arg2,
                                arg3: arg3,
                                reg1: 0,
                                reg2: 0,
                                reg3: 0,
                                result: result,
                                pc: pc
                            };
                            self.instruction = None;
                            self.dec_instruction = Some(instruction);

                        }
                        else {
                            dec_status = InstructionType::Blocked;
                        }


                    }

                    else if opcode == ADD_RR { //register + register 
                        arg1 = (((instr_binary as u32) >> REG1_SHIFT) & REG_MASK) as i32;
                        arg2 = (((instr_binary as u32) >> REG2_SHIFT) & REG_MASK) as i32;
                        arg3 = (((instr_binary as u32) >> REG3_SHIFT) & REG_MASK) as i32;

                        if !reg.is_pending(arg1 as usize) && !reg.is_pending(arg2 as usize) {
                            instruction = Instruction {
                                instr_type: instr_type,
                                device: Devices::Decode,
                                type_field: type_field as i32,
                                opcode: opcode as i32,
                                arg1: arg1,
                                arg2: arg2,
                                arg3: arg3,
                                reg1: 0,
                                reg2: 0,
                                reg3: 0,
                                result: result,
                                pc: pc
                            };
                             self.instruction = None;
                             self.dec_instruction = Some(instruction);

                        }

                        else {
                            dec_status = InstructionType::Blocked;
                        }

                    }
                }

                else if opcode >= CONTROL_RANGE[0] as u32 && opcode <= CONTROL_RANGE[1] as u32 {
                    instr_type = InstructionType::Control;

                    if opcode == JMP_D || opcode == JMP_I  
                    || opcode == JL_D || opcode == JL_I 
                    || opcode == JE_D || opcode == JE_I 
                    || opcode == JG_D || opcode == JG_I  {  //JMP, JE, JL, JG 
                        //assume arg1 register src, arg2 register offset
                        arg1 = (((instr_binary as u32) >> REG1_SHIFT) & REG_MASK) as i32;
                        arg2 = (((instr_binary as u32) >> REG2_SHIFT) & REG_MASK) as i32;

                        if !reg.is_pending(arg1 as usize) && !reg.is_pending(arg2 as usize) {
                            instruction = Instruction {
                                instr_type: instr_type,
                                device: Devices::Decode,
                                type_field: type_field as i32,
                                opcode: opcode as i32,
                                arg1: arg1,
                                arg2: arg2,
                                arg3: 0,
                                reg1: 0,
                                reg2: 0,
                                reg3: 0,
                                result: result,
                                pc: pc
                            };
                            self.instruction = None;
                            self.dec_instruction = Some(instruction);

                        }

                        else {
                            dec_status = InstructionType::Blocked;
                        }


                    }
                    else if opcode == JMP_PC || opcode == JL_PC || opcode == JE_PC || opcode == JG_PC {
                        arg1 = pc;
                        arg2 = (((instr_binary as u32) >> REG2_SHIFT) & REG_MASK) as i32;

                        if !reg.is_pending(arg2 as usize) {
                            instruction = Instruction {
                                instr_type: instr_type,
                                device: Devices::Decode,
                                type_field: type_field as i32,
                                opcode: opcode as i32,
                                arg1: arg1,
                                arg2: arg2,
                                arg3: 0,
                                reg1: 0,
                                reg2: 0,
                                reg3: 0,
                                result: result,
                                pc: pc
                            };
                            self.instruction = None;
                            self.dec_instruction = Some(instruction);

                        }

                        else {
                            dec_status = InstructionType::Blocked;
                        }
                    }
                    else if opcode == CMP_RI { //CMP
                       
                        arg1 = (((instr_binary as u32) >> REG1_SHIFT) & REG_MASK) as i32;
                        arg2 = (((instr_binary as u32) >> IMMEDIATE2_SHIFT) & IMMEDIATE_MASK) as i32;


                        if !reg.is_pending(arg1 as usize) {
                            instruction = Instruction {
                                instr_type: instr_type,
                                device: Devices::Decode,
                                type_field: type_field as i32,
                                opcode: opcode as i32,
                                arg1: arg1,
                                arg2: arg2,
                                arg3: 0,
                                reg1: 0,
                                reg2: 0,
                                reg3: 0,
                                result: result,
                                pc: pc
                            };
                             self.instruction = None;
                             self.dec_instruction = Some(instruction);

                        }
                        else {
                            dec_status = InstructionType::Blocked;
                        }


                    }

                    else if opcode == CMP_RR { //register cmp register 
                        arg1 = (((instr_binary as u32) >> REG1_SHIFT) & REG_MASK) as i32;
                        arg2 = (((instr_binary as u32) >> REG2_SHIFT) & REG_MASK) as i32;
                        arg3 = 0;

                        if !reg.is_pending(arg1 as usize) && !reg.is_pending(arg2 as usize) {
                            instruction = Instruction {
                                instr_type: instr_type,
                                device: Devices::Decode,
                                type_field: type_field as i32,
                                opcode: opcode as i32,
                                arg1: arg1,
                                arg2: arg2,
                                arg3: arg3,
                                reg1: 0,
                                reg2: 0,
                                reg3: 0,
                                result: result,
                                pc: pc
                            };
                            self.instruction = None;
                            self.dec_instruction = Some(instruction);

                        }

                        else {
                            dec_status = InstructionType::Blocked;
                        }

                    }

                    else if opcode == RET {
                        //return sets is a special jump effectively, sets the pc to lr
                        arg1 = 0;
                        arg2 = 0;
                        arg3 = 0;

                        instruction = Instruction {
                                instr_type: instr_type,
                                device: Devices::Decode,
                                type_field: type_field as i32,
                                opcode: opcode as i32,
                                arg1: arg1,
                                arg2: arg2,
                                arg3: arg3,
                                reg1: 0,
                                reg2: 0,
                                reg3: 0,
                                result: result,
                                pc: pc
                            };
                            self.instruction = None;
                            self.dec_instruction = Some(instruction);
                    }

                    

                }

                else if opcode >= MEMORY_RANGE[0] as u32 && opcode <= MEMORY_RANGE[1] as u32 {
                    instr_type = InstructionType::Memory;
                    if opcode == FDR || opcode == FTR {
                        arg1 = -1;
                        arg2 = -1;
                        arg3 = -1;
                        
                        instruction = Instruction {
                                instr_type: instr_type,
                                device: Devices::Memory,
                                type_field: type_field as i32,
                                opcode: opcode as i32,
                                arg1: arg1,
                                arg2: arg2,
                                arg3: arg3,
                                reg1: 0,
                                reg2: 0,
                                reg3: 0,
                                result: None,
                                pc: pc
                            };
                            self.instruction = None;
                            self.dec_instruction = Some(instruction);

                    }
                    else if opcode == LDR_PC || opcode == GDR_PC {
                        arg1 = 32;
                        arg2 = (((instr_binary as u32) >> REG2_SHIFT) & REG_MASK) as i32;
                        arg3 = ((instr_binary as u32 >> IMMEDIATE3_SHIFT) & IMMEDIATE_MASK) as i32;

                        instruction = Instruction {
                                instr_type: instr_type,
                                device: Devices::Memory,
                                type_field: type_field as i32,
                                opcode: opcode as i32,
                                arg1: arg1,
                                arg2: arg2,
                                arg3: arg3,
                                reg1: 0,
                                reg2: 0,
                                reg3: 0,
                                result: None,
                                pc: pc
                            };
                            self.instruction = None;
                            self.dec_instruction = Some(instruction);

                    }
                    else if opcode == STR_PC || opcode == GTR_PC {
                        arg1 = (((instr_binary as u32) >> REG1_SHIFT) & REG_MASK) as i32;
                        arg2 = 32;
                        arg3 = ((instr_binary as u32 >> IMMEDIATE3_SHIFT) & IMMEDIATE_MASK) as i32;

                        if !reg.is_pending(arg1 as usize) {
                            instruction = Instruction {
                                instr_type: instr_type,
                                device: Devices::Memory,
                                type_field: type_field as i32,
                                opcode: opcode as i32,
                                arg1: arg1,
                                arg2: arg2,
                                arg3: arg3,
                                reg1: 0,
                                reg2: 0,
                                reg3: 0,
                                result: None,
                                pc: pc
                            };
                            self.instruction = None;
                            self.dec_instruction = Some(instruction);
                        }
                    }
                    else if opcode == PSH {
                        arg1 = (((instr_binary as u32) >> REG1_SHIFT) & REG_MASK) as i32; //register being pushed, src
                        arg2 = 33; //sp register for "dst"
                        arg3 = 0; //no offset
                        let reg1 = arg1;
                        let reg2 = arg2;
                        let reg3 = arg3;

                        if !reg.is_pending(reg1 as usize) {
                            instruction = Instruction {
                                instr_type: instr_type,
                                device: Devices::Memory,
                                type_field: type_field as i32,
                                opcode: opcode as i32,
                                arg1: arg1,
                                arg2: arg2,
                                arg3: arg3,
                                reg1: reg1,
                                reg2: reg2,
                                reg3: reg3,
                                result: None,
                                pc: pc
                            };
                            self.instruction = None;
                            self.dec_instruction = Some(instruction);
                        }

                        else {
                             dec_status = InstructionType::Blocked;
                        }
                    }
                    else if opcode == POP {
                        //arg1 is src so SP, arg2 is register to be restored arg3 is nothing
                        arg1 = 33; 
                        arg2 = (((instr_binary as u32) >> REG2_SHIFT) & REG_MASK) as i32;
                        arg3 = 0; //no offset
                        let reg1 = arg1;
                        let reg2 = arg2;
                        let reg3 = arg3;

                        if !reg.is_pending(reg1 as usize) { //if stack is about to be written to, don't do it
                            instruction = Instruction {
                                instr_type: instr_type,
                                device: Devices::Memory,
                                type_field: type_field as i32,
                                opcode: opcode as i32,
                                arg1: arg1,
                                arg2: arg2,
                                arg3: arg3,
                                reg1: reg1,
                                reg2: reg2,
                                reg3: reg3,
                                result: None,
                                pc: pc
                            };
                            self.instruction = None;
                            self.dec_instruction = Some(instruction);
                        }

                        else {
                             dec_status = InstructionType::Blocked;
                        }

                    }
                    else { //some sort of normal load or store
                        //assume arg1 src and arg2 dst registers with an immediate offset (12 bits) arg3
                        arg1 = (((instr_binary as u32) >> REG1_SHIFT) & REG_MASK) as i32;
                        arg2 = (((instr_binary as u32) >> REG2_SHIFT) & REG_MASK) as i32;
                        arg3 = ((instr_binary as u32 >> IMMEDIATE3_SHIFT) & IMMEDIATE_MASK) as i32;

                        if !reg.is_pending(arg1 as usize) {
                            instruction = Instruction {
                                instr_type: instr_type,
                                device: Devices::Memory,
                                type_field: type_field as i32,
                                opcode: opcode as i32,
                                arg1: arg1,
                                arg2: arg2,
                                arg3: arg3,
                                reg1: 0,
                                reg2: 0,
                                reg3: 0,
                                result: None,
                                pc: pc
                            };
                            self.instruction = None;
                            self.dec_instruction = Some(instruction);
                        }

                        else {
                            dec_status = InstructionType::Blocked;
                        }
                    }
                    

                }
                else {
                    
                    instruction = Instruction {
                        instr_type: InstructionType::NOOP,
                        device: Devices::Decode,
                        type_field: -1,
                        opcode: -1,
                        arg1: -1,
                        arg2: -1,
                        arg3: -1,
                        reg1: 0,
                        reg2: 0,
                        reg3: 0,
                        result: None,
                        pc: -1
                    };
                    self.instruction = None;
                    self.dec_instruction = Some(instruction);


                }
            }

            println!("Decode status: {}", dec_status.to_string());
            let (fetch_return, new_instr, instr_pc) = self.fetch_stage.call(dec_status, reg, cache, ctrl);

            if fetch_return == InstructionType::NotBlocked {
                println!("Decode recieved an instruction from fetch: {} with a pc value of {}", new_instr.to_string(), instr_pc.to_string());
                self.instruction = Some((fetch_return, new_instr, instr_pc));
            }
            else {
                println!("Decode recieved a stall from fetch");
                //self.instruction = Some((fetch_return, new_instr, instr_pc));
            }

            if self.dec_instruction.is_some() && exec_status != InstructionType::Blocked {
                let Some(mut instr) = self.dec_instruction.take() else { return None };
                if instr.opcode == HALT as i32 {
                    self.dec_instruction = None;
                    //self.instruction = None;
                    return Some(instr); //pass it on
                }
                if instr.instr_type == InstructionType::ALU {
                    if instr.opcode == ADD_RR as i32 {
                        if  !reg.is_pending(instr.arg1 as usize) && !reg.is_pending(instr.arg2 as usize) {
                            let reg1 = instr.arg1;
                            let reg2 = instr.arg2;
                            let reg3 = instr.arg3;
                            instr.arg1 = reg.get_gp(instr.arg1 as usize);
                            instr.arg2 = reg.get_gp(instr.arg2 as usize);
                            instr.reg1 = reg1;
                            instr.reg2 = reg2;
                            instr.reg3 = reg3;
                            reg.update_pending(instr.arg3 as usize, true);
                            self.dec_instruction = None;
                            //self.instruction = None;
                            return Some(instr);
                        }
                        else {
                            self.dec_instruction = Some(instr);
                            return Some(Instruction {
                                    instr_type: InstructionType::Stall,
                                    device: Devices::Decode,
                                    type_field: -1,
                                    opcode: -1,
                                    arg1: -1,
                                    arg2: -1,
                                    arg3: -1,
                                    reg1: 0,
                                    reg2: 0,
                                    reg3: 0,
                                    result: None,
                                    pc: -1
                            });
                        }
                    }
                    else if instr.opcode == ADD_RI as i32 {
                        if  !reg.is_pending(instr.arg1 as usize)  { //arg2 is immediate
                            let reg1 = instr.arg1;
                            let reg2 = -1;
                            let reg3 = instr.arg3;
                            instr.arg1 = reg.get_gp(instr.arg1 as usize);
                            instr.reg1 = reg1;
                            instr.reg2 = reg2;
                            instr.reg3 = reg3;
                            reg.update_pending(instr.arg3 as usize, true);
                            let ret_instr = self.dec_instruction.take();
                            self.dec_instruction = None;
                            return Some(instr);
                        }
                        else {
                            self.dec_instruction = Some(instr);
                            return Some(Instruction {
                                    instr_type: InstructionType::Stall,
                                    device: Devices::Decode,
                                    type_field: -1,
                                    opcode: -1,
                                    arg1: -1,
                                    arg2: -1,
                                    arg3: -1,
                                    reg1: 0,
                                    reg2: 0,
                                    reg3: 0,
                                    result: None,
                                    pc: -1
                            });
                        }
                        
                    }
                    else {
                            self.dec_instruction = Some(instr);
                            return Some(Instruction {
                                    instr_type: InstructionType::Stall,
                                    device: Devices::Decode,
                                    type_field: -1,
                                    opcode: -1,
                                    arg1: -1,
                                    arg2: -1,
                                    arg3: -1,
                                    reg1: 0,
                                    reg2: 0,
                                    reg3: 0,
                                    result: None,
                                    pc: -1
                            });
                        }
                    
                }
                else if instr.instr_type == InstructionType::Control {
                    if instr.opcode == JMP_D as i32|| instr.opcode == JMP_I as i32 
                    || instr.opcode == JL_D as i32 || instr.opcode == JL_I as i32
                    || instr.opcode == JE_D as i32 || instr.opcode == JE_I as i32
                    || instr.opcode == JG_D as i32 || instr.opcode == JG_I as i32 {
                        if !reg.is_pending(instr.arg1 as usize) && !reg.is_pending(instr.arg2 as usize) {
                                let reg1 = instr.arg1;
                                let reg2 = instr.arg2;
                                let reg3 = -1;
                                instr.arg1 = reg.get_gp(instr.arg1 as usize);
                                
                                if instr.type_field == 1 { //if offset
                                    instr.arg2 = reg.get_gp(instr.arg2 as usize);
                                }
                                
                                instr.reg1 = reg1;
                                instr.reg2 = reg2;
                                instr.reg3 = reg3;
                                if instr.opcode == JMP_D as i32 || instr.opcode == JMP_I as i32 {
                                    reg.update_pending(34, true); //set lr to pending to prevent weird stuff
                                }
                                self.dec_instruction = None;
                                //self.instruction = None;
                                return Some(instr);
                        }
                        else {
                                self.dec_instruction = Some(instr);
                                return Some(Instruction {
                                instr_type: InstructionType::Stall,
                                device: Devices::Decode,
                                type_field: -1,
                                opcode: -1,
                                arg1: -1,
                                arg2: -1,
                                arg3: -1,
                                reg1: 0,
                                reg2: 0,
                                reg3: 0,
                                result: None,
                                pc: -1
    
                            });
                        }
                    }
                    else if instr.opcode == JMP_PC as i32 || instr.opcode == JL_PC as i32 || instr.opcode == JE_PC as i32 || instr.opcode == JG_PC as i32 {
                        if !reg.is_pending(instr.arg2 as usize) {
                                let reg1 = 0;
                                let reg2 = instr.arg2;
                                let reg3 = -1;
                                instr.arg1 = instr.pc;
                                
                                if instr.type_field == 1 { //if offset
                                    instr.arg2 = reg.get_gp(instr.arg2 as usize);
                                }
                                
                                instr.reg1 = reg1;
                                instr.reg2 = reg2;
                                instr.reg3 = reg3;
                                if instr.opcode == JMP_PC as i32 {
                                    reg.update_pending(34, true); //set lr to pending to prevent weird stuff
                                }
                                self.dec_instruction = None;
                                //self.instruction = None;
                                return Some(instr);
                        }
                        else {
                                self.dec_instruction = Some(instr);
                                return Some(Instruction {
                                instr_type: InstructionType::Stall,
                                device: Devices::Decode,
                                type_field: -1,
                                opcode: -1,
                                arg1: -1,
                                arg2: -1,
                                arg3: -1,
                                reg1: 0,
                                reg2: 0,
                                reg3: 0,
                                result: None,
                                pc: -1
    
                            });
                        }
                    }
                    else if instr.opcode == CMP_RI as i32 { //if CMP
                            // register cmp immediate
                            if !reg.is_pending(instr.arg1 as usize) {
                                let reg1 = instr.arg1;
                                instr.arg1 = reg.get_gp(instr.arg1 as usize);
                                instr.reg1 = reg1;
                                instr.reg2 = -1;
                                instr.reg3 = -1;
                                self.dec_instruction = None;
                                //self.instruction = None;
                                return Some(instr);
                            }
                            else {
                                self.dec_instruction = Some(instr);
                                return Some(Instruction {
                                    instr_type: InstructionType::Stall,
                                    device: Devices::Decode,
                                    type_field: -1,
                                    opcode: -1,
                                    arg1: -1,
                                    arg2: -1,
                                    arg3: -1,
                                    reg1: 0,
                                    reg2: 0,
                                    reg3: 0,
                                    result: None,
                                    pc: -1
                                });
                            } 

                            
                        }
                         else if instr.opcode == CMP_RR as i32 { //register cmp register
                                if !reg.is_pending(instr.arg1 as usize) && !reg.is_pending(instr.arg2 as usize) {
                                    let reg1 = instr.arg1;
                                    let reg2 = instr.arg2;

                                    instr.arg1 = reg.get_gp(instr.arg1 as usize);
                                    instr.arg2 = reg.get_gp(instr.arg2 as usize);
                                    instr.reg1 = reg1;
                                    instr.reg2 = reg2;
                                    instr.reg3 = -1;
                                    self.dec_instruction = None;
                                    return Some(instr);
                                }
                                else {
                                    self.dec_instruction = Some(instr);
                                    return Some(Instruction {
                                        instr_type: InstructionType::Stall,
                                        device: Devices::Decode,
                                        type_field: -1,
                                        opcode: -1,
                                        arg1: -1,
                                        arg2: -1,
                                        arg3: -1,
                                        reg1: 0,
                                        reg2: 0,
                                        reg3: 0,
                                        result: None,
                                        pc: -1
                                    });
                                }

                            }
                            else if instr.opcode == RET as i32 {
                                if !reg.is_pending(34) { //if lr is not about to be written to
                                    self.dec_instruction = None;
                                    return Some(instr);
                                }
                                else {
                                    self.dec_instruction = Some(instr);
                                    return Some(Instruction {
                                        instr_type: InstructionType::Stall,
                                        device: Devices::Decode,
                                        type_field: -1,
                                        opcode: -1,
                                        arg1: -1,
                                        arg2: -1,
                                        arg3: -1,
                                        reg1: 0,
                                        reg2: 0,
                                        reg3: 0,
                                        result: None,
                                        pc: -1
                                    });
                                }
                                
                            }
                    else {
                        //self.instruction = None;
                        return Some(instr);
                    }

                }

                else if instr.instr_type == InstructionType::Memory {
                    if !reg.is_pending(instr.arg1 as usize) {
                        if instr.opcode == LDR_D  as i32 || instr.opcode == LDR_I as i32 || instr.opcode == GDR_D as i32 || instr.opcode == GDR_I as i32 { //if LDR
                            reg.update_pending(instr.arg2 as usize, true); //set dst addr to pending so it isn't overwritten
                            let reg1 = instr.arg1; 
                            let reg2 = instr.arg2;
                            let reg3 = instr.arg3;
                            instr.arg1 = reg.get_gp(instr.arg1 as usize); //get addr from src reg

                            instr.reg1 = reg1;
                            instr.reg2 = reg2;
                            instr.reg3 = reg3;
                            //self.instruction = None;
                            self.dec_instruction = None;
                            return Some(instr);
                        }
                        else if instr.opcode == STR_D as i32 || instr.opcode == STR_I as i32 || instr.opcode == GTR_D as i32 || instr.opcode == GTR_I as i32 { //assume STR else for now
                            //no write, so no update pending
                            let reg1 = instr.arg1; 
                            let reg2 = instr.arg2;
                            let reg3 = instr.arg3;

                            instr.arg1 = reg.get_gp(instr.arg1 as usize); //put register data in instruction
                            instr.arg2 = reg.get_gp(instr.arg2 as usize); //get dst address

                            instr.reg1 = reg1;
                            instr.reg2 = reg2;
                            instr.reg3 = reg3;
                            //self.instruction = None;
                            self.dec_instruction = None;
                            return Some(instr);
                        }
                        else if instr.opcode == LDR_PC as i32 || instr.opcode == GDR_PC as i32 {
                            reg.update_pending(instr.arg2 as usize, true); //set dst addr to pending so it isn't overwritten
                            let reg1 = instr.arg1; 
                            let reg2 = instr.arg2;
                            let reg3 = instr.arg3;
                            
                            instr.arg1 = instr.pc; //get use own pc as addr base

                            instr.reg1 = reg1;
                            instr.reg2 = reg2;
                            instr.reg3 = reg3;
                            //self.instruction = None;
                            self.dec_instruction = None;
                            return Some(instr);
                        }
                        else if instr.opcode == STR_PC as i32 || instr.opcode == GTR_PC as i32 {
                            //no write, so no update pending
                            let reg1 = instr.arg1; 
                            let reg2 = instr.arg2;
                            let reg3 = instr.arg3;

                            instr.arg1 = reg.get_gp(instr.arg1 as usize); //put register data in instruction
                            instr.arg2 =instr.pc; //use pc as base for dst

                            instr.reg1 = reg1;
                            instr.reg2 = reg2;
                            instr.reg3 = reg3;
                            //self.instruction = None;
                            self.dec_instruction = None;
                            return Some(instr);
                        }
                        else if instr.opcode == POP as i32 {
                            instr.arg1 = reg.get_gp(instr.arg1 as usize);
                            instr.arg2 = reg.get_gp(instr.arg2 as usize);
                            reg.update_pending(instr.reg2 as usize, true); //set reg being written to as pending
                            self.dec_instruction = None;
                            return Some(instr);
                        }
                        else if instr.opcode == PSH as i32 || instr.opcode == POP as i32 { //once 
                            instr.arg1 = reg.get_gp(instr.arg1 as usize);
                            instr.arg2 = reg.get_gp(instr.arg2 as usize);
                            self.dec_instruction = None;
                            return Some(instr);
                        }
                       
                        else {
                        self.dec_instruction = Some(instr);
                        return Some(Instruction {
                                instr_type: InstructionType::Stall,
                                device: Devices::Decode,
                                type_field: -1,
                                opcode: -1,
                                arg1: -1,
                                arg2: -1,
                                arg3: -1,
                                reg1: 0,
                                reg2: 0,
                                reg3: 0,
                                result: None,
                                pc: -1
                        });
                    }
                            
                    }
                    else {
                        self.dec_instruction = Some(instr);
                        return Some(Instruction {
                                instr_type: InstructionType::Stall,
                                device: Devices::Decode,
                                type_field: -1,
                                opcode: -1,
                                arg1: -1,
                                arg2: -1,
                                arg3: -1,
                                reg1: 0, 
                                reg2: 0,
                                reg3: 0,
                                result: None,
                                pc: -1
                        });
                    }
                }
                else if instr.instr_type == InstructionType::Squashed {
                    self.dec_instruction = None;
                    return Some(instr);
                }

                else {
                    
                    //self.instruction = None;
                    self.dec_instruction = Some(instr);
                    return Some(instr)
                }

            }
            else {
                return None;
            }
                

        }

            //returns (cur_instruction, dec_instruction) as strings, used by UI to get info
        pub fn state(&self) -> (String, String) {
            let cur_inst: i32;
            let ret_cur_inst: String;
            let _pc: i32;
            let _type: InstructionType;
            let dec_inst: String;

            if self.instruction.is_some() {
                (_type, cur_inst, _pc) = self.instruction.unwrap();
                ret_cur_inst = cur_inst.to_string();

            }
            else {
                ret_cur_inst = "None".to_owned();
            }

            if self.dec_instruction.is_some() {
                dec_inst = self.dec_instruction.unwrap().formatted_to_string();
            }
            else {
                dec_inst = "None".to_owned();
            }

            return (ret_cur_inst, dec_inst);
        }


        }

        impl Fetch {
        pub fn new() -> Self {
            Fetch {
                load_instruction: None,
                cur_pc: 0,
                cur_instruction: None
            }
        }

        //call: used by Decode to call Fetch. 
        //Returns: InstructionType/integer triple, (stall/not stall, instruction bits, instruction pc). 
        //InstructionType is to indicate if it is blocked/stalling. Any other return value just means there is an instruction being returned.
        pub fn call(&mut self, decode_status: InstructionType, reg: &mut Registers, cache: &mut Cache, ctrl: &mut Controler) -> (InstructionType, i32, i32) { //add PC value to return element
            
            if decode_status == InstructionType::Squashed {
                self.load_instruction = None;
                self.cur_instruction = None;
                cache.squash_cache();
                return (InstructionType::Squashed, -1, -1);
            }

            if !ctrl.status() && ctrl.in_process() > 0 {
                return return (InstructionType::Stall, -1, -1);
            }

            if self.load_instruction.is_none() {
                self.cur_pc = reg.get_pc();
                self.load_instruction = Some(Instruction { 
                instr_type: InstructionType::Memory, 
                device: Devices::Fetch, 
                type_field: 0, //register direct
                opcode: LDR_D as i32, //LDR 
                arg1: self.cur_pc, //PC register 
                arg2: 0, //don't matter 
                arg3: 0, //no offset
                reg1: 0,
                reg2: 0,
                reg3: 0,
                result: None,
                pc: self.cur_pc 
            }); //don't matter
            }
            
            
            if self.cur_instruction.is_none() { 
                
                let next_instr = cache.call(self.load_instruction.unwrap()); 
                
                match next_instr {
               
                ReturnVal::Data(i) => { 
                    println!("Fetch has gotten a new instruction from the cache");
                    self.cur_instruction = Some(i);
                }
                ReturnVal::Wait(y) => {
                    self.cur_instruction = None; //if no data, then stall
                }
            } 
            }//try to get from memory

            else { //if already have, just set again to do the next bit
                println!("Fetch already has an instruction");
            }
            
            

            if decode_status != InstructionType::Blocked && self.cur_instruction.is_some() {
                println!("Fetch has returned an instruction to decode: {}", self.cur_instruction.unwrap().to_string());
                reg.inc_pc();
                self.load_instruction = None;
                let ret_val = self.cur_instruction.unwrap();
                self.cur_instruction = None;
                ctrl.inc();
                return (InstructionType::NotBlocked, ret_val, self.cur_pc);
            }
            else {
                println!("Fetch is stalling because it has no instruction or decode is blocked");
                return (InstructionType::Stall, -1, -1);
            }

        }

        //called by UI to access fetch stage state
        //returns strings representing the current instruction if any and the current PC value being dealt with
        pub fn state(&self) -> (String, String) {
            let cur_inst: String;

            if self.cur_instruction.is_some() {
                cur_inst = self.cur_instruction.unwrap().to_string();
            }
            else {
                cur_inst = "None".to_owned();
            }

            return (cur_inst, self.cur_pc.to_string());
        }
    } 


}
