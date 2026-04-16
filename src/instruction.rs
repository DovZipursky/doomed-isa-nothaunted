pub mod instruction {
    use std::fmt;
    use strum_macros::Display;


    #[derive(Eq, PartialEq, Copy, Clone, Display, Debug)]
    pub enum InstructionType {
        ALU,
        Memory,
        Control,
        NOOP,
        Stall,
        Squashed,
        Blocked,
        NotBlocked
    }

    #[derive(Eq, PartialEq, Copy, Clone, Display, Debug)]
    pub enum Devices {
        Clock,
        Writeback,
        Memory,
        Excecute,
        Decode,
        Fetch,
        Free,
        Cache
    }

    #[derive(Eq, PartialEq, Clone, Copy, Debug)]
    pub struct Instruction {
        pub instr_type: InstructionType,
        pub device: Devices,
        pub type_field: i32,
        pub opcode: i32,
        pub arg1: i32,
        pub arg2: i32,
        pub arg3: i32,
        pub result: Option<i32>,
        pub pc: i32
    }

    impl Instruction {
        pub fn new() -> Self {
            Instruction {
                instr_type: InstructionType::Blocked,
                device: Devices::Clock,
                type_field: 0,
                opcode: 0,
                arg1: 0,
                arg2: 0,
                arg3: 0,
                result: None,
                pc: 0
            }
        }

        pub fn formatted_to_string(&mut self) -> String {
            let result: String;
            if self.result.is_some() {
                result = self.result.unwrap().to_string();
            }
            else {
                result = "None".to_owned();
            }

            return format!("Instruction Type: {} \n\nPC Value: {}\nType Field: {}\nOpcode: {}\nArgs: {}, {}, {} \nResult: {}\n", 
                        self.instr_type.to_string(), self.pc.to_string(), self.type_field.to_string(), self.opcode.to_string(), 
                        self.arg1.to_string(), self.arg2.to_string(), self.arg3.to_string(), result);

        }
    }


}