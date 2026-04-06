pub mod pipeline {
    use crate::{instruction::{self, instruction::Instruction, instruction::InstructionType, instruction::Devices}, memory::{self, memory::{Cache, Registers, ReturnVal}}};
    use std::cell::RefCell;

    //constants that define the range of opcodes that refer to different instruction types
    const ALU_RANGE: [i32; 2] = [1, 14];
    const CONTROL_RANGE: [i32; 2] = [15, 19];
    const MEMORY_RANGE: [i32; 2] = [20, 25];

const TYPE_SHIFT: u32 = 31;
const OPCODE_SHIFT: u32 = 26;
const REG1_SHIFT: u32 = 21;
const REG2_SHIFT: u32 = 16;
const REG3_SHIFT: u32 = 11;
const IMMEDIATE2_SHIFT: u32 = 5;
const IMMEDIATE3_SHIFT: u32 = 0;

const TYPE_MASK: u32 = 0b1;
const OPCODE_MASK: u32 = 0b1_1111;
const REG_MASK: u32 = 0b1_1111;
const IMMEDIATE_MASK: u32 = 0b1111_1111_1111_1111;

    pub struct Fetch {
        load_instruction: Option<Instruction>,
        cur_instruction: Option<i32>,
        cur_pc: i32,
    }
    pub struct Decode  {
        instruction: Option<(i32, i32)>,
        dec_instruction: Option<Instruction>,
        pub fetch_stage: Fetch,
    }
    pub struct Execute  {
        instruction: Option<Instruction>,
        pub dec_stage: Decode,
    }
    pub struct Memory  {
        instruction: Option<Instruction>,
        pub exec_stage: Execute
    }
    pub struct Writeback  {
        instruction: Option<Instruction>,
        pub mem_stage: Memory,
    }

    impl Writeback {
        pub fn new(mem_stage: Memory) -> Self {
            Writeback {
                instruction: None,
                mem_stage: mem_stage,
            }
        }

        pub fn call(&mut self, reg: &mut Registers, cache: &mut Cache) -> Option<Instruction>{
            let mut wb_status = InstructionType::NotBlocked;
            //if there is some instruction that is not NOOP, Stalled, or Squashed:
            if self.instruction.is_some()  && (self.instruction.unwrap().instr_type !=  InstructionType::NOOP ||self.instruction.unwrap().instr_type !=  InstructionType::Squashed || self.instruction.unwrap().instr_type !=  InstructionType::Stall) {
                
                let instr = self.instruction.unwrap();
                //if there is a result and it is not a control flow instruction
                if instr.result.is_some() && instr.instr_type == InstructionType::Memory {
                    //assume arg2 is a destination if LDR
                    if instr.opcode == 20 {
                        reg.update_gp(instr.arg2 as usize, instr.result.unwrap());
                        reg.update_pending(instr.arg2 as usize, false); 
                    }
                    else if instr.opcode == 21 {
                        //with STR registers are not written to, but update src as not pending
                        reg.update_pending(instr.arg1 as usize, false);
                    }
                    //write to register and update to no longer pending
                    
                    
                }
                else if instr.result.is_some() && instr.instr_type == InstructionType::ALU {
                    //assume arg3 is a destination
                    
                    reg.update_gp(instr.arg3 as usize, instr.result.unwrap());
                    
                    reg.update_pending(instr.arg3 as usize, false); 
                }   

                else if instr.instr_type == InstructionType::Control { //if it is a control flow instruction
                    //assume result is the update to PC
                    if instr.opcode == 15 { //JMP
                        reg.update_gp(32, instr.result.unwrap());
                        wb_status = InstructionType::Squashed;
                    }
                    else if instr.opcode == 18 { //JL
                        wb_status = InstructionType::Squashed;
                        reg.update_gp(32, instr.result.unwrap());
                        
                    }

                   
                }
            }

            let next_instr = self.mem_stage.call(reg, cache, wb_status);

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
                exec_stage: exec_stage
            }
        }

        pub fn call(&mut self, reg: &mut Registers, cache: &mut Cache, wb_status: InstructionType) -> Option<Instruction>{
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
                                        result: None,
                                        pc: -1
                    });
                mem_status = InstructionType::Squashed;
                let maybe_next_instr = self.exec_stage.call(mem_status, reg, cache);
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

            let maybe_next_instr = self.exec_stage.call(mem_status, reg, cache);

            
            let ret_instr = self.instruction;
            if ret_instr.is_none() || (ret_instr.is_some() && (ret_instr.unwrap().instr_type != InstructionType::Memory || ret_instr.unwrap().result.is_some())) { 
                    //if not a memeory instruction, or result has been gotten for a memory instruction:
                    self.instruction = maybe_next_instr; 
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

        pub fn call(&mut self, mem_status: InstructionType, reg: &mut Registers, cache: &mut Cache) -> Option<Instruction> {
            if mem_status == InstructionType::Squashed {
                self.instruction = Some(Instruction {
                                        instr_type: InstructionType::Squashed,
                                        device: Devices::Memory,
                                        type_field: -1,
                                        opcode: -1,
                                        arg1: -1,
                                        arg2: -1,
                                        arg3: -1,
                                        result: None,
                                        pc: -1
                    });
                let dec_ret = self.dec_stage.call(mem_status, reg, cache);
                return self.instruction;
            }
            if let Some(instr) = self.instruction.as_mut() {
                if instr.instr_type == InstructionType::ALU {
                    if instr.opcode == 1 { //if ADD, use values provided by decode
                        instr.result.replace( instr.arg1 +  instr.arg2);
                        
                    }
                }
                else if  instr.instr_type == InstructionType::Control {
                    if  instr.opcode == 15 { //JMP
                        instr.result.replace( instr.arg1 + instr.arg2); //src + offset
                    }
                    else if instr.opcode == 18 {//JL
                        if reg.get_flags() == -1 {
                            instr.result.replace(instr.arg1 + instr.arg2);
                        }
                        else {
                            instr.result.replace(reg.get_pc());
                        }

                    }
                    else if  instr.opcode == 16 { //CMP
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
                    if  instr.opcode == 20 { //if LDR
                       instr.arg1 =  instr.arg1 + instr.arg3;
                    }
                    else if instr.opcode == 21 {
                        instr.arg2 =  instr.arg2 +  instr.arg3; 
                    }
                    else {
                        //whatever
                    }

                }




            }

            let next = self.dec_stage.call(mem_status, reg, cache);
            let cur = self.instruction;
            if next.is_some() {
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
                dec_instruction: None
            }
        }

        pub fn call(&mut self, exec_status: InstructionType, reg: &mut Registers, cache: &mut Cache) -> Option<Instruction> {
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
                                        result: None,
                                        pc: -1
                    });
                self.instruction = None;
                let _fetch_ret = self.fetch_stage.call(dec_status, reg, cache);
                return self.dec_instruction;
            }
            
            if self.instruction.is_some() {
                let (instr_binary, pc) = self.instruction.unwrap();
                let type_field = ((instr_binary as u32) >> TYPE_SHIFT) & TYPE_MASK; //most sig bit
                let opcode = (((instr_binary as u32) >> OPCODE_SHIFT)) & OPCODE_MASK; //next 5 bits

                let arg1: i32;
                let arg2: i32;
                let arg3: i32;
                let instr_type: InstructionType;
                let result: Option<i32> = None; 
                let instruction: Instruction;

                if opcode >= ALU_RANGE[0] as u32 && opcode <= ALU_RANGE[1] as u32 { //ALU section
                    instr_type = InstructionType::ALU;
                    if type_field == 0 { //assume reg + immediate to dst arg3
                        arg1 = (((instr_binary as u32) >> REG1_SHIFT) & REG_MASK) as i32;
                        arg2 = (((instr_binary as u32) >> IMMEDIATE2_SHIFT) & IMMEDIATE_MASK) as i32;
                        arg3 = ((instr_binary as u32) & REG_MASK) as i32; //no shift since whole structure is taken up

                        if !reg.is_pending(arg1 as usize) {
                            instruction = Instruction {
                                instr_type: instr_type,
                                device: Devices::Decode,
                                type_field: type_field as i32,
                                opcode: opcode as i32,
                                arg1: arg1,
                                arg2: arg2,
                                arg3: arg3,
                                result: result,
                                pc: pc
                            };

                            self.dec_instruction = Some(instruction);

                        }
                        else {
                            dec_status = InstructionType::Blocked;
                        }


                    }

                    else { //register + register 
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
                                result: result,
                                pc: pc
                            };

                             self.dec_instruction = Some(instruction);

                        }

                        else {
                            dec_status = InstructionType::Blocked;
                        }

                    }
                }

                else if opcode >= CONTROL_RANGE[0] as u32 && opcode <= CONTROL_RANGE[1] as u32 {
                    instr_type = InstructionType::Control;

                    if opcode == 15 || opcode == 17 || opcode == 18 || opcode == 19{  //JMP, JE, JL, JG 
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
                                result: result,
                                pc: pc
                            };

                            self.dec_instruction = Some(instruction);

                        }

                        else {
                            dec_status = InstructionType::Blocked;
                        }


                    }
                    else if opcode == 16 { //CMP
                        if type_field == 0 { //assume reg cmp imm
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
                                result: result,
                                pc: pc
                            };

                             self.dec_instruction = Some(instruction);

                        }
                        else {
                            dec_status = InstructionType::Blocked;
                        }


                    }

                    else { //register cmp register 
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
                                result: result,
                                pc: pc
                            };

                             self.dec_instruction = Some(instruction);

                        }

                        else {
                            dec_status = InstructionType::Blocked;
                        }

                    }

                    }



                }

                else if opcode >= MEMORY_RANGE[0] as u32 && opcode <= MEMORY_RANGE[1] as u32 {
                    instr_type = InstructionType::Memory;

                    //assume arg1 src and arg2 dst registers with a 16 bit offset arg3
                    arg1 = (((instr_binary as u32) >> REG1_SHIFT) & REG_MASK) as i32;
                    arg2 = (((instr_binary as u32) >> REG2_SHIFT) & REG_MASK) as i32;
                    arg3 = ((instr_binary as u32) & IMMEDIATE_MASK) as i32;

                    if !reg.is_pending(arg1 as usize) {
                        instruction = Instruction {
                            instr_type: instr_type,
                            device: Devices::Memory,
                            type_field: type_field as i32,
                            opcode: opcode as i32,
                            arg1: arg1,
                            arg2: arg2,
                            arg3: arg3,
                            result: None,
                            pc: pc
                        };

                        self.dec_instruction = Some(instruction);
                    }

                    else {
                        dec_status = InstructionType::Blocked;
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
                        result: None,
                        pc: -1
                    };

                    self.dec_instruction = Some(instruction);


                }
            }

            let (fetch_return, new_instr, instr_pc) = self.fetch_stage.call(dec_status, reg, cache);

            if fetch_return == InstructionType::NotBlocked {
                self.instruction.replace((new_instr, instr_pc));
            }
            else {
                    //might happen but fine
            }

            if let Some(mut instr) = self.dec_instruction.take() && exec_status != InstructionType::Blocked {
                if instr.instr_type == InstructionType::ALU {
                    if instr.type_field == 1 {
                        if  !reg.is_pending(instr.arg1 as usize) && !reg.is_pending(instr.arg2 as usize) {
                            instr.arg1 = reg.get_gp(instr.arg1 as usize);
                            instr.arg2 = reg.get_gp(instr.arg2 as usize);
                            reg.update_pending(instr.arg3 as usize, true);
                            self.instruction = None;
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
                                    result: None,
                                    pc: -1
                            });
                        }
                    }
                    else {
                        if  !reg.is_pending(instr.arg1 as usize)  { //arg2 is immediate
                            instr.arg1 = reg.get_gp(instr.arg1 as usize);
                            reg.update_pending(instr.arg3 as usize, true);
                            let ret_instr = self.dec_instruction.take();
                            self.instruction = None;
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
                                    result: None,
                                    pc: -1
                            });
                        }
                    }
                    
                }
                else if instr.instr_type == InstructionType::Control {
                    if instr.opcode == 15 || instr.opcode == 17 || instr.opcode == 18 || instr.opcode == 19 {
                        if !reg.is_pending(instr.arg1 as usize) && !reg.is_pending(instr.arg2 as usize) {
                                instr.arg1 = reg.get_gp(instr.arg1 as usize);
                                if instr.type_field == 1 {
                                    instr.arg2 = reg.get_gp(instr.arg2 as usize);
                                }

                                self.dec_instruction = None;
                                self.instruction = None;
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
                                result: None,
                                pc: -1
    
                            });
                        }
                    }
                    else if instr.opcode == 16 { //if CMP
                            if instr.type_field == 0 { // register cmp immediate
                                if !reg.is_pending(instr.arg1 as usize) {
                                    instr.arg1 = reg.get_gp(instr.arg1 as usize);
                                    self.dec_instruction = None;
                                    self.instruction = None;
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
                                        result: None,
                                        pc: -1
                                    });
                                }

                            }
                            else { //register cmp register
                                if !reg.is_pending(instr.arg1 as usize) && reg.is_pending(instr.arg2 as usize) {
                                    instr.arg1 = reg.get_gp(instr.arg1 as usize);
                                    instr.arg2 = reg.get_gp(instr.arg2 as usize);
                                    self.instruction = None;
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
                                        result: None,
                                        pc: -1
                                    });
                                }

                            }
                        }
                    else {
                        self.instruction = None;
                        return Some(instr);
                    }

                }

                else if instr.instr_type == InstructionType::Memory {
                    if !reg.is_pending(instr.arg1 as usize) {
                        if instr.opcode == 20 { //if LDR
                            reg.update_pending(instr.arg2 as usize, true); //set dst addr to pending so it isn't overwritten
                            instr.arg1 = reg.get_gp(instr.arg1 as usize); //get addr from src reg
                            self.instruction = None;
                            self.dec_instruction = None;
                            return Some(instr);
                        }
                        else { //assume STR else for now
                            //no write, so no update pending
                            instr.arg2 = reg.get_gp(instr.arg2 as usize); //get dst address
                            self.instruction = None;
                            self.dec_instruction = None;
                            return Some(instr);
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
                                result: None,
                                pc: -1
                        });
                    }
                }

                else {
                    
                    self.instruction = None;
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
            let dec_inst: String;

            if self.instruction.is_some() {
                (cur_inst, _pc) = self.instruction.unwrap();
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
        pub fn call(&mut self, decode_status: InstructionType, reg: &mut Registers, cache: &mut Cache) -> (InstructionType, i32, i32) { //add PC value to return element
            
            if decode_status == InstructionType::Squashed {
                self.load_instruction = None;
                self.cur_instruction = None;
                cache.squash_cache();
                return (InstructionType::Squashed, -1, -1);
            }

            if self.load_instruction.is_none() {
                self.cur_pc = reg.get_pc();
                self.load_instruction = Some(Instruction { 
                instr_type: InstructionType::Memory, 
                device: Devices::Fetch, 
                type_field: 0, //register direct
                opcode: 20, //LDR 
                arg1: self.cur_pc, //PC register 
                arg2: 0, //don't matter 
                arg3: 0, //no offset
                result: None,
                pc: self.cur_pc 
            }); //don't matter
            }
            let next_instr;
            if self.cur_instruction.is_none() { 
                next_instr = cache.call(self.load_instruction.unwrap());  
            }//try to get from memory

            else { //if already have, just set again to do the next bit
                next_instr = ReturnVal::Data(self.cur_instruction.unwrap_or(0));
            }
            
            match next_instr {
               
                ReturnVal::Data(i) => { //NOOP is just a non-stall placeholder, Decode will ignore that
                    self.cur_instruction = Some(i);
                }
                ReturnVal::Wait(y) => {
                    self.cur_instruction = None; //if no data, then stall
                }
            }

            if decode_status != InstructionType::Blocked && self.cur_instruction.is_some() {
                reg.inc_pc();
                self.load_instruction = None;
                let ret_val = self.cur_instruction.unwrap();
                self.cur_instruction = None;
                return (InstructionType::NotBlocked, ret_val, self.cur_pc);
            }
            else {
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
