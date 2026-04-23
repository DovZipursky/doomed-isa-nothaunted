pub mod memory {
    use std::{collections::LinkedList, fs};

    use crate::instruction::instruction::{Devices, Instruction};
    use crate::opcode::opcode::{FDR, FTR, GDR_D, GDR_I, GTR_D, GTR_I, LDR_D, LDR_I, LDR_PC, POP, PSH, STR_D, STR_I, STR_PC};

    const CACHE_SIZE: i32 = 250;
    pub const MEMORY_SIZE: i32 = 1000;
    pub const GRAPHICS_OFFSET: i32 = ((FRAME_WIDTH * FRAME_HEIGHT) / 32); //includes the size given by line width
    pub const STACK_INIT: i32 = ((GRAPHICS_OFFSET / 4) - 1) * 4; //set inital stack to top of memory below graphics memory
    
    const CACHE_DELAY: i32 = 0;                     
    const MEMORY_DELAY: i32 = 3;
    const TAG_LENGTH: u32 = 32 - CACHE_SIZE.ilog2();
    const INDEX_LENGTH: u32 = CACHE_SIZE.ilog2();
    const FRAME_WIDTH: i32 = 320;
    const FRAME_HEIGHT: i32 = 240;
    
    #[derive(Debug)]
    #[derive(Eq, PartialEq)]
    pub enum ReturnVal {
        Wait(bool),
        Data(i32)
    }
    

    pub struct Registers {
        pub reg: [i32; 36], //pc = reg 33 (index 32), sp = reg 34 (index 33), lr = reg 35 (index 34)
        pending: [bool; 32],
        cmp_flag: i32
    }

    impl Registers {
        pub fn new() -> Self {
            let mut reg = [0; 35];
            reg[33] = STACK_INIT;

            Registers {
                reg: [0; 36],
                pending: [false; 32],
                cmp_flag: 0
            }
        }


        pub fn update_gp(&mut self, reg_number: usize, value: i32) -> bool {

            self.reg[reg_number] = value;
            return true;
        }

        pub fn update_pending(&mut self, reg_number: usize, value: bool) {
            self.pending[reg_number] = value;
        }

        pub fn is_pending(&self, reg_number: usize) -> bool {
            return self.pending[reg_number];
        }

        pub fn get_gp(&self, reg_number: usize) -> i32 {
            return self.reg[reg_number];
        }

        pub fn inc_pc(&mut self) -> i32 {
            self.reg[32] = self.reg[32] + 1;
            return self.reg[32];
        }

        pub fn pc_jump(&mut self, jump_val: i32) -> i32 {
            self.reg[32] = self.reg[32] + jump_val;
            return self.reg[32];
        }
        pub fn get_pc(&self) -> i32 {
            return self.reg[32];
        }

        pub fn update_flags(&mut self, value: i32) {
            self.cmp_flag = value;
        }

        pub fn get_flags(&self) -> i32 {
            return self.cmp_flag;
        }
    }

    pub struct Cache {
        pub data: [[i32; 7]; CACHE_SIZE as usize],
        pub main_memory: [[i32; 4]; MEMORY_SIZE as usize],
        frame_buffer: [[i32; 4]; (GRAPHICS_OFFSET / 4) as usize],
        delay: i32,
        counter: i32,
        servicing: Devices,
        instruction: Option<Instruction>,
        hit: bool,
        pub on: bool
    }

    impl Cache {
        pub fn new(data: [[i32; 7]; CACHE_SIZE as usize], main_memory: [[i32; 4]; MEMORY_SIZE as usize]) -> Self {
            Cache { 
                data: data, //address of element: data[addr % CACHE_SIZE][3]
                                            //data[index][0] = the tag, which is the remaining 23 bits of the address
                                            //data[index][1] = the valid bit, which indicates whether the cache line contains valid data or not
                                            //data[index][2] = dirty bit, whether the cache line has been written to and not evicted
                                            //data[index][3...6] = the data in the line, 4 words of 32 bits
                main_memory: main_memory, //address of element: main_memory[addr/4][addr % 4]
                frame_buffer: [[-1; 4]; (GRAPHICS_OFFSET / 4) as usize],
                delay: CACHE_DELAY,         
                counter: 0,                 
                servicing: Devices::Free,
                instruction: None, 
                hit: false,
                on: true
            }
        }

        pub fn call(&mut self, instr: Instruction) -> ReturnVal {
            
            let calling_device = instr.device;
            
            //if calling device is not the same and cache is working, return wait
            if self.servicing != Devices::Free && calling_device != self.servicing {
                return ReturnVal::Wait(true);
            }

            //start new interaction if free
            if self.servicing == Devices::Free { //Address of structure TAG | INDEX | OFFSET
                                                    // for cache: tag = addr / (4 * CACHE_SIZE), index = (addr / 4) % CACHE_SIZE, offset = addr % CACHE_SIZE) 
                self.servicing = calling_device;    // for memory: index = addr / 4, offset = addr % 4 
                self.instruction = Some(instr);

                
                    //expect arg1 to be an address 

                if instr.opcode == LDR_D as i32 || instr.opcode == GDR_D as i32 || instr.opcode == POP as i32 || instr.opcode ==  LDR_PC as i32 { //if register direct
                    //check hit/miss
                    let index = (instr.arg1 / 4) % CACHE_SIZE;
                    let tag = instr.arg1 / (4 * CACHE_SIZE);
                    
                    if !self.on {
                        self.delay = MEMORY_DELAY;
                        self.hit = false;
                    }


                    else if self.data[index as usize][1] == 1 && self.data[index as usize][0] == tag  { //both valid and proper tag
                        self.delay = CACHE_DELAY;
                        self.hit = true;
                    }
                    else { //else miss, 
                        self.delay = MEMORY_DELAY;
                        self.hit = false;
                    }
                    
                }
                else if instr.opcode == LDR_I as i32 || instr.opcode == GDR_I as i32  { //if register indirect
                    let main_addr = instr.arg1; //main memory address specified by register
                    //check cache for that memory
                    let mut index_i = (main_addr / 4) % CACHE_SIZE;
                    let mut tag_i = main_addr / (4 * CACHE_SIZE);
                    
                    if !self.on {
                        self.delay = 2 * MEMORY_DELAY;
                        self.hit = false;
                    }

                    else if self.data[index_i as usize][1] == 1 && self.data[index_i as usize][0] == tag_i  { //both valid and proper tag
                        self.delay = CACHE_DELAY; //base delay is cache to get indirect address
                    }
                    else { //else miss, 
                        self.delay = MEMORY_DELAY; //base delay is memory to get indirect address
                    }

                    let addr = self.main_memory[(main_addr  / 4) as usize][(main_addr % 4) as usize]; //find address in the memory cell specified by the register and add offset
                    
                    let index = (addr / 4) % CACHE_SIZE; //find mapped location
                    let tag = addr / (4 * CACHE_SIZE);

                    if self.data[index as usize][1] == 1 && self.data[index as usize][0] == tag  && self.on { //both valid and proper tag for actual requested data
                        self.delay = self.delay + CACHE_DELAY; //add aditional cache delay for hit
                        self.hit = true;
                    }
                    else if self.on { //else miss, 
                        self.delay = self.delay + MEMORY_DELAY;  //add aditional memory delay for miss
                        self.hit = false;
                    }

                }
                    
                

            
                    //expect arg2 to be an address
                    
                else if instr.opcode == STR_D as i32  || instr.opcode == GTR_D as i32 ||instr.opcode == PSH as i32  || instr.opcode == STR_PC as i32 { //if register direct
                    //let index =  instr.arg2 % CACHE_SIZE; //map address with offset to cache index
                    //let tag = instr.arg2 >> (32 - TAG_LENGTH);
                    self.delay = MEMORY_DELAY; //due to write through

                }

                else if instr.opcode == STR_I as i32 || instr.opcode == GTR_I as i32  { //if register indirect
                    let main_addr = instr.arg2; //main memory address specified by instruction
                    //check cache for that memory
                    let mut index_i = (main_addr / 4) % CACHE_SIZE;
                    let mut tag_i = main_addr / (4 * CACHE_SIZE);

                    if !self.on {
                        self.delay = MEMORY_DELAY;
                        self.hit = false;
                    }

                    else if self.data[index_i as usize][1] == 1 && self.data[index_i as usize][0] == tag_i  { //both valid and proper tag
                        self.delay = CACHE_DELAY; //base delay is cache to get indirect address
                    }
                    else { //else miss, 
                        self.delay = MEMORY_DELAY; //base delay is memory to get indirect address
                    }

                    //let addr = self.main_memory[(main_addr / 4) % (MEMORY_SIZE * 4)][main_addr % 4] + instr.arg3; //find address in the memory cell specified by the register and add offset
                    //index = addr % CACHE_SIZE; //find mapped location
                    //tag = (addr << TAG_LENGTH) >> TAG_LENGTH;

                    self.delay = self.delay + MEMORY_DELAY; //to account for write through
                    

                }

                else if instr.opcode == FDR as i32 || instr.opcode == FTR as i32 { //load graphics memory into frame buffer
                    self.delay = MEMORY_DELAY * (MEMORY_SIZE - (GRAPHICS_OFFSET / 4)); //delay equivalent to loading or storing  all lines individually
                }
                 

                return ReturnVal::Wait(true) //return wait on first call 
            }

            //continue old interaction if instruction is the same as stored one
            if self.instruction.as_ref().is_some_and(|i| i == &instr) && calling_device == self.servicing  {
                
                if self.counter < self.delay  { //still need to delay
                    self.counter = self.counter + 1;
                    //return ReturnVal::Wait(true)
                }

                if self.counter == self.delay { //if counter reached delay this cycle
                    //if LDR, simply return data
                    //TODO: !self.on
                        
                    if instr.opcode == LDR_D as i32 ||  instr.opcode == GDR_D as i32 || instr.opcode == POP as i32 || instr.opcode == LDR_PC as i32 { //if register direct
                        if !self.on {
                            let index = instr.arg1 / 4; 
                            let offset = instr.arg1 % 4;
                            return self.finish_instruction(index, offset);

                        }

                        else if self.hit {
                                let index = (instr.arg1 / 4)  % CACHE_SIZE; //map address with offset to cache index
                            //self.counter = 0;
                            //self.servicing = Devices::Free;
                            //return ReturnVal::Data(self.data[index as usize][3]) 
                            return self.finish_instruction(index, instr.arg1 % 4);
                        }
                        else { //miss
                            let data = self.main_memory[(instr.arg1 / 4) as usize]; //use address in register to index memory
                            self.data[((instr.arg1 / 4) % CACHE_SIZE) as usize][3] = data[0]; //put data in cache for later
                            self.data[((instr.arg1 / 4) % CACHE_SIZE) as usize][4] = data[1];
                            self.data[((instr.arg1 / 4) % CACHE_SIZE) as usize][5] = data[2];
                            self.data[((instr.arg1 / 4) % CACHE_SIZE) as usize][6] = data[3];
                            self.data[((instr.arg1 / 4) % CACHE_SIZE) as usize][0] = instr.arg1 / (CACHE_SIZE * 4);
                            self.data[((instr.arg1 / 4) % CACHE_SIZE) as usize][1] = 1;
                            //self.counter = 0;
                            //self.servicing = Devices::Free;
                            //return ReturnVal::Data(data) 
                            return self.finish_instruction((instr.arg1 / 4) % CACHE_SIZE, instr.arg1 % 4);
                        }
                        
                    }
                    
                    else if instr.opcode == LDR_I as i32  || instr.opcode == GDR_I as i32 { //if indirect
                        if !self.on {
                            let main_index = instr.arg1 / 4; 
                            let main_offset = instr.arg1 % 4;
                            let addr = self.main_memory[(main_index / 4) as usize][(main_offset % 4) as usize];

                            return self.finish_instruction(addr / 4, addr % 4);

                        }

                        else if self.hit {
                            let main_addr = instr.arg1; //get address from memory, delay already accounted for if any
                            let addr = self.main_memory[(main_addr / 4) as usize][(main_addr % 4) as usize]; 
                            let index = (addr / 4) % CACHE_SIZE;
                            let offset = addr % 4; 
                         
                            return self.finish_instruction(index, offset);
                        }
                        
                        else {
                            let main_addr = instr.arg1; //get from memory, delay already accounted for if any
                            let addr = self.main_memory[(main_addr / 4) as usize][(main_addr % 4) as usize]; 
                            let data = self.main_memory[(addr / 4) as usize];

                            self.data[((addr / 4) % CACHE_SIZE) as usize][3] = data[0]; //put data in cache for later
                            self.data[((addr / 4) % CACHE_SIZE) as usize][4] = data[1];
                            self.data[((addr / 4) % CACHE_SIZE) as usize][5] = data[2];
                            self.data[((addr / 4) % CACHE_SIZE) as usize][6] = data[3];
                            self.data[((addr / 4) % CACHE_SIZE) as usize][0] = addr/ (CACHE_SIZE * 4);
                            self.data[((addr / 4) % CACHE_SIZE) as usize][1] = 1;
                            
                            //self.counter = 0;
                            //self.servicing = Devices::Free;
                            //return ReturnVal::Data(data) 
                            return self.finish_instruction((addr / 4) % CACHE_SIZE, addr % 4);
                        }
                        

                    }

                        
                    else if instr.opcode == STR_D as i32 || instr.opcode == GTR_D as i32 || instr.opcode == PSH as i32 || instr.opcode == STR_PC as i32  { //assumes that memory and cache are synched
                            
                            //if register direct
                            let addr = instr.arg2;
                            let index = (addr / 4) % CACHE_SIZE;
                            
                            if !self.on { //just update and return
                                self.main_memory[(addr / 4) as usize][(addr % 4) as usize] = instr.arg1;
                                return self.finish_instruction(addr / 4, addr % 4);
                            }

                            //let mut wthrough_addr = index << CACHE_SIZE.ilog2(); //find real memory address of currently stored data by using index and tag
                            //wthrough_addr = wthrough_addr | index;
                           
                            let tag = addr / (CACHE_SIZE * 4);
                            

                            if self.data[index as usize][0] != tag && self.data[index as usize][1] == 1 { //line is occupied with different section of memory
                                //full "eviction", so write over whole cache line before replacing the tag and relevant word
                                //assume whatever memory the old cache info was reffering to is up to date due to write through
                                //let e_addr = (self.data[index as usize][0] << INDEX_LENGTH) | index; if we were doing writeback this would be handy
                                for i in 0..4 {
                                    self.data[index as usize][(i + 3) as usize] = self.main_memory[(addr / 4) as usize][i as usize]; 
                                }

                            }
                            
                            self.main_memory[(addr / 4) as usize][(addr % 4) as usize] = instr.arg1; //write through
                            self.data[index as usize][((addr % 4) + 3) as usize] = instr.arg1; //store data in cache
                            self.data[index as usize][0] = tag; //set tag
                            self.data[index as usize][1] = 1; //set valid bit

                           
                            return self.finish_instruction(index, addr % 4);
                    }

                    else if instr.opcode == STR_I as i32 || instr.opcode == GTR_I as i32 {
                            let main_addr = instr.arg2;
                            let addr = self.main_memory[(main_addr / 4) as usize][(main_addr % 4) as usize]; //find address in the memory cell specified by the register and add offset
                            let index = (addr / 4) % CACHE_SIZE; //find mapped location

                            if !self.on { //just update and return
                                self.main_memory[(addr / 4) as usize][(addr % 4) as usize] = instr.arg1;
                                return self.finish_instruction(addr / 4, addr % 4);
                            }

                            let tag = addr / (CACHE_SIZE * 4);
                            
                            
                            if self.data[index as usize][0] != tag && self.data[index as usize][1] == 1 { //line is occupied with different section of memory
                                //full "eviction", so write over whole cache line before replacing the tag and relevant word
                                //assume whatever memory the old cache info was reffering to is up to date due to write through
                                //let e_addr = (self.data[index as usize][0] << INDEX_LENGTH) | index;
                                for i in 0..4 {
                                    self.data[index as usize][(i + 3) as usize] = self.main_memory[(addr / 4) as usize][i as usize]; 
                                }

                            }
                            
                            self.main_memory[(addr / 4) as usize][(addr % 4) as usize] = instr.arg1; //write through
                            self.data[index as usize][((addr % 4) + 3) as usize] = instr.arg1; //store data in cache
                            self.data[index as usize][0] = tag; //set tag
                            self.data[index as usize][1] = 1; //set valid bit

                           
                            return self.finish_instruction(index, addr % 4);
                    }

                    if instr.opcode == FDR as i32 { //load graphics memory into frame buffer
                        for i in (GRAPHICS_OFFSET / 4)..(MEMORY_SIZE - 1) { //for line between beginning of graphics memory and top of memory
                            for j in 0..4 { //for word in line
                                self.frame_buffer[(i - (GRAPHICS_OFFSET / 4)) as usize][j as usize] = self.main_memory[i as usize][j as usize] 
                            }
                        }

                        return self.finish_instruction((MEMORY_SIZE - 1) % CACHE_SIZE, (MEMORY_SIZE - 1) % 4) //confirmation return, no real info
                    }

                    if instr.opcode == FTR as i32 {
                        for i in (GRAPHICS_OFFSET / 4)..(MEMORY_SIZE) { //for line between beginning of graphics memory and top of memory
                            for j in 0..4 { //for word in line
                                self.main_memory[i as usize][j as usize] = self.frame_buffer[(i - (GRAPHICS_OFFSET / 4)) as usize][j as usize] 
                            }
                        }

                        return self.finish_instruction((MEMORY_SIZE - 1) % CACHE_SIZE, (MEMORY_SIZE - 1) % 4) //confirmation return, no real info
                    }

                        

                    return ReturnVal::Wait(true);

                }
            
            
            }

            return ReturnVal::Wait(true);
       
           
        } //end of fn

        pub fn squash_cache(&mut self) {
            self.counter = 0;
            self.servicing = Devices::Free;
            self.instruction = None;
            self.hit = false;
            return;
        }

        fn finish_instruction(&mut self, index: i32, offset: i32) -> ReturnVal {
            self.counter = 0;
            self.servicing = Devices::Free;
            if !self.on {
                return ReturnVal::Data(self.main_memory[index as usize][offset as usize]);
            }
            return ReturnVal::Data(self.data[index as usize][(offset + 3) as usize]); //just something that isn't wait
        }

        pub fn get_cache(&self, line: i32) -> [i32; 7] {
            return self.data[((line / 4) % CACHE_SIZE) as usize];
        }

        pub fn get_memory(&self, addr: i32) -> [i32; 4] {
            return self.main_memory[(addr / 4) as usize];
        }

        pub fn switch(&mut self, val: bool) {
            self.on = val;
        }
        
    
        pub fn load_memory_from_file(&mut self, path: String) {
            // I'm not sure of the best way to architect this, so for now return a list with each unparsed instruction on a new entry
            let mut instructions: Vec<u32> = Vec::new();

            let instruction_bytes = fs::read(path).expect("File read");
            let mut instruction_bytes_list: LinkedList<_> = instruction_bytes.into_iter().collect();

            while (instruction_bytes_list.len() != 0){
                instructions.push(u32::from_le_bytes([instruction_bytes_list.pop_front().expect("Popped first byte"), instruction_bytes_list.pop_front().expect("Popped second byte"), instruction_bytes_list.pop_front().expect("Popped third byte"), instruction_bytes_list.pop_front().expect("Popped fourth byte")]));
            }

            let mut i = 0;
            let mut j = 0;
            for instr in instructions {
                
                self.main_memory[i as usize][j] = instr as i32;

                if j == 3 {
                    i = i + 1;
                }

                j = (j + 1) % 4;
                
            }
            
            return;
        }

    }
    

    
}
