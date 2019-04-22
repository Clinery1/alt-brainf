use std::{
    env,
    fs::{
        self,
        OpenOptions,
    },
    io::{
        Stdin,
        stdin,
        BufRead,
        Write,
    },
    collections::HashMap,
};


const BLOCK_COMMENT_ERR:&str = "Unterminated block comment!";
const PARSER_ERR:&str = "The parser didn't do it's job! Please contact the creator with the code and interpreter version.";
const FUNCTION_WITHIN:&str = "Found a function within a function! This is not allowed! Please revise your code.";
const FUNCTION_WHILE_RUNNING:&str = "Tried to make a function while the program was running! This is a fatal error.
Please submit a bug report to the authors.";
const CELL_PTR_PAST_MAX:&str = "Tried to increment the cell pointer past the max amount of cells: 1048567.";
const CELL_PTR_BELOW_MAX:&str = "Tried to decrement the cell pointer below 0.";
const NOT_REMOVE_FUNC:&str = "The interpreter did not remove all functions from the instructions.";
const FUNC_IN_FUNC:&str = "There is a function within a function after parsing. This is a fatal error. Please submit a bug report to the authors.";
const INFO_IN_EXE_LOG:&str =
"Format for this file:
    Command:Cursor:Buffer at cursor location
    The other words are just identifiers.
    The printed characters are in the format 'Out: (char)'
    The read characters are in the format 'Read: (char)'

Instruction count: ";


// Eventually add in file operators
#[derive(PartialEq,Debug,Clone)]
enum Commands {
    Print,                              // .
    Read,                               // ,
    IncPtr,                             // >
    DecPtr,                             // <
    IncCell,                            // +
    DecCell,                            // -
    Loop(Vec<Commands>),                // []
    FunctionCaller,                     // !
    Function(Vec<Commands>,(u8,u8)),    // {*#*} I made it so there are over 65000 possible function identifiers instead of 255
}
impl Commands {
    fn parse(input:String) -> Result<Vec<Commands>,String> {
        let mut res = Vec::<Commands>::new();
        let input = input.chars().collect::<Vec<char>>();
        let mut functions = Vec::<Commands>::new();
        let mut i = 0;
        let mut err_val = String::new();
        while i<input.len() {
            let char = input[i];
            match char {
                '+'=>{
                    res.push(Commands::IncCell)
                },
                '-'=>{
                    res.push(Commands::DecCell)
                },
                '>'=>{
                    res.push(Commands::IncPtr)
                },
                '<'=>{
                    res.push(Commands::DecPtr)
                },
                '['=>{
                    let res1 = Commands::parse_loop(input[i+1..].to_vec());
                    let (mut loop_contents,mut inc) = (vec![],0);
                    match res1 {
                        Ok(val)=>{loop_contents=val.0;inc=val.1;},
                        Err(val)=>{err_val=val;},
                    }
                    res.push(Commands::Loop(loop_contents));
                    i+=inc;
                },
                ']'=>{
                    err_val = PARSER_ERR.to_string();
                    println!("2");
                },
                '.'=>{
                    res.push(Commands::Print)
                },
                ','=>{
                    res.push(Commands::Read)
                },
                '!'=>{
                    res.push(Commands::FunctionCaller)
                }, // My own awesome implementation for this incredible language.
                '{'=>{
                    let res = Commands::parse_function(input[i+1..].to_vec());
                    let (mut contents,mut id,mut inc) = (vec![],(0,0),0);
                    match res {
                        Ok(val)=>{contents=val.0;id=val.1;inc=val.2;},
                        Err(val)=>{err_val=val;}
                    }
                    functions.push(Commands::Function(contents,id));
                    i+=inc;
                },
                '}'=>{
                    err_val = PARSER_ERR.to_string();
                    println!("1");
                },
                '/'=>{
                    if !(i+1>=input.len()) {
                        if input[i+1]=='/' {
                            while i<input.len() {
                                if input[i]=='\n' {break}
                                i+=1;
                                if input.len()==i {break}
                            }
                        } else if input[i+1]=='*' {
                            i+=2;
                            if i>=input.len() {err_val = BLOCK_COMMENT_ERR.to_string();}
                            while i<input.len() {
                                if input[i]=='*'&&input[i+1]=='/' {break}
                                else if input.len()<=i+1 {err_val = BLOCK_COMMENT_ERR.to_string();break;}
                                i+=1;
                                if input.len()<=i+1 {err_val = BLOCK_COMMENT_ERR.to_string();break;}
                            }
                            i+=2;
                        }
                    } else {
                        err_val = BLOCK_COMMENT_ERR.to_string();
                    }
                },
                _=>{},
            }
            i+=1;
            if err_val.len()>0 {return Err(err_val);}
        }
        functions.append(&mut res);
        return Ok(functions);
    }
    fn parse_function(input:Vec<char>) -> Result<(Vec<Commands>,(u8,u8),usize),String> {
        let mut i = 0;
        let mut res = Vec::<Commands>::new();
        let mut id = (0,0);
        let mut err_val = String::new();
        use Commands::*;
        while i<input.len() {
            match input[i] {
                '#'=>{
                    let commands = res.clone(); res = vec![];
                    let mut tmp = Memory::new();
                    tmp.instructions = commands;
                    match tmp.run() {
                        Ok(())=>{},
                        Err(val)=>{err_val = val;},
                    }
                    id = (tmp.buf[0],tmp.buf[1]); // The 1st and 2nd cell are the two parts of the ID
                }
                '+'=>{res.push(IncCell)},
                '-'=>{res.push(DecCell)},
                '>'=>{res.push(IncPtr)},
                '<'=>{res.push(DecPtr)},
                '.'=>{res.push(Print)},
                ','=>{res.push(Read)},
                '['=>{
                    let res1 = Commands::parse_loop(input[i+1..].to_vec());
                    let (mut contents,mut inc) = (vec![],0);
                    match res1 {
                        Ok(val)=>{contents=val.0;inc=val.1;},
                        Err(val)=>{err_val=val;},
                    }
                    res.push(Commands::Loop(contents));
                    i+=inc;
                },
                ']'=>{err_val = PARSER_ERR.to_string();},
                '{'=>{err_val = FUNCTION_WITHIN.to_string();},
                '}'=>{return Ok((res,id,i+1))},
                '/'=>{
                    if !(i+1>=input.len()) {
                        if input[i+1]=='/' {
                            while i<input.len() {
                                if input[i]=='\n' {break}
                                i+=1;
                                if input.len()==i {break}
                            }
                        } else if input[i+1]=='*' {
                            i+=2;
                            if i>=input.len() {err_val = BLOCK_COMMENT_ERR.to_string();}
                            while i<input.len() {
                                if input[i]=='*'&&input[i+1]=='/' {break}
                                else if input.len()<=i+1 {err_val = BLOCK_COMMENT_ERR.to_string();break;}
                                i+=1;
                                if input.len()<=i+1 {err_val = BLOCK_COMMENT_ERR.to_string();break;}
                            }
                            i+=2;
                        }
                    } else {
                        err_val = BLOCK_COMMENT_ERR.to_string();
                    }
                },
                _=>{},
            }
            i+=1;
            if err_val.len()>0 {break;}
        }
        return Ok((vec![],(0,0),0));
    }
    fn parse_loop(input:Vec<char>) -> Result<(Vec<Commands>,usize),String> {
        let mut i = 0;
        let mut res = Vec::<Commands>::new();
        let mut err_val = String::new();
        while i<input.len() {
            match input[i] {
                '+'=>{res.push(Commands::IncCell)},
                '-'=>{res.push(Commands::DecCell)},
                '['=>{
                    let res1 = Commands::parse_loop(input[i+1..].to_vec());
                    let (mut loop_commands,mut inc) = (vec![],0);
                    match res1 {
                        Ok(val)=>{loop_commands=val.0;inc=val.1;},
                        Err(val)=>{err_val=val;}
                    }
                    res.push(Commands::Loop(loop_commands));
                    i+=inc;
                },
                ']'=>{i+=1;break;},
                '>'=>{res.push(Commands::IncPtr)},
                '<'=>{res.push(Commands::DecPtr)},
                '.'=>{res.push(Commands::Print)},
                ','=>{res.push(Commands::Read)},
                '!'=>{res.push(Commands::FunctionCaller)},
                '/'=>{
                    if !(i+1>=input.len()) {
                        if input[i+1]=='/' {
                            while i<input.len() {
                                if input[i]=='\n' {break}
                                i+=1;
                                if input.len()==i {break}
                            }
                        } else if input[i+1]=='*' {
                            i+=2;
                            if i>=input.len() {err_val = BLOCK_COMMENT_ERR.to_string();}
                            while i<input.len() {
                                if input[i]=='*'&&input[i+1]=='/' {break}
                                else if input.len()<=i+1 {err_val = BLOCK_COMMENT_ERR.to_string();break;}
                                i+=1;
                                if input.len()<=i+1 {err_val = BLOCK_COMMENT_ERR.to_string();break;}
                            }
                            i+=2;
                        }
                    } else {
                        err_val = BLOCK_COMMENT_ERR.to_string();
                    }
                },
                _=>{},
            }
            i+=1;
            if err_val.len()>0 {break;}
        }
        return Ok((res,i));
    }
    fn c(&self) -> String {
        use Commands::*;
        match self {
            Print=>".".to_string(),
            Read=>",".to_string(),
            IncPtr=>">".to_string(),
            DecPtr=>"<".to_string(),
            IncCell=>"+".to_string(),
            DecCell=>"-".to_string(),
            Loop(x)=>{
                let mut res = String::new();
                res.push('[');
                for c in x {res.push_str(&c.c())}
                res.push(']');
                return res.clone();
            },
            Function(x,i)=>{
                let mut r = String::new();
                r.push('{');
                r.push_str(&format!("{:?}#",i));
                for c in x {r.push_str(&c.c());}
                r.push('}');
                return r;
            }
            FunctionCaller=>"!".to_string(),
        }
    }
}


#[derive(Clone)]
struct Memory {
    buf:[u8;1048576],
    cursor:usize,
    instructions:Vec<Commands>,
    instruction_ptr:usize,
    instruction:usize,
    execution_log:String,
    functions:HashMap<(u8,u8),Vec<Commands>>,
}
impl Memory {
    fn new() -> Memory {
        Memory {
            buf:[0;1048576],
            cursor:0,
            instructions:Vec::new(),
            instruction_ptr:0,
            instruction:0,
            execution_log:String::new(),
            functions:HashMap::new(),
        }
    }
    fn run(&mut self) -> Result<(),String> {
        use Commands::*;
        let mut stdin = stdin();
        let mut err_val = String::new();
        let mut iterations = 0;
        let mut i = 0;
        while i<self.instructions.len() {
            let com = &self.instructions[0];
            match com {
                Function(val,id)=>{
                    self.functions.insert(*id,val.clone());
                    self.instructions.remove(0);
                },
                _=>{break;},
            }
            i+=1;
        }
        while self.instruction_ptr<self.instructions.len() {
            if iterations > 500 {break}
            let instruction = &self.clone().instructions[self.instruction_ptr];
            match instruction {
                Print=>{
                    print!("{}",char::from(self.buf[self.cursor]));
                    self.execution_log.push_str(&format!("Out: {:?}\n",char::from(self.buf[self.cursor])));
                },
                Read=>{
                    let mut stdin = stdin.lock();
                    let res = stdin.fill_buf().unwrap()[0];
                    stdin.consume(1);
                    self.buf[self.cursor] = res;
                    self.execution_log.push_str(&format!("Read: {:?}\n",char::from(res)));
                },
                IncPtr=>{
                    err_val = self.move_cell_ptr(true);
                },
                DecPtr=>{
                    err_val = self.move_cell_ptr(false);
                },
                IncCell=>{
                    err_val = self.change_cell(true);
                },
                DecCell=>{
                    err_val = self.change_cell(false);
                },
                Loop(val)=>{
                    self.execution_log.push_str("\nLoop\n");
                    err_val= self.execute_loop(val.to_vec(),&mut stdin);
                },
                FunctionCaller=>{
                    let id = (self.buf[self.cursor],self.buf[self.cursor+1]);
                    if let Some(_) = self.functions.get(&id) {
                        err_val = self.execute_function(id, &mut stdin);
                    }
                },
                Function(_,_)=>{panic!(FUNCTION_WHILE_RUNNING)},
            }
            self.instruction_ptr+=1;
            self.instruction+=1;
            self.execution_log.push_str(&format!("{}:{}:{}\n",instruction.c(),self.cursor,self.buf[self.cursor]));
            if err_val.len()>0 {
                return Err(format!("{}",err_val));
            }
            iterations+=1;
        }
        self.execution_log.push_str(&format!("\n\nInstruction count: {}",self.instruction));
        Ok(())
    }
    fn change_cell(&mut self,inc:bool) -> String {
        if inc {
            if self.buf[self.cursor]==255 {
                return format!(
                    "Tried to increment cell {} past 255, the max amount.",
                    self.cursor
                );
            }
            self.buf[self.cursor]+=1;
        } else {
            if self.buf[self.cursor]==0 {
                return format!(
                    "Tried to decrement cell {} below zero.",
                    self.cursor
                );
            }
            self.buf[self.cursor]-=1;
        }
        String::new()
    }
    fn move_cell_ptr(&mut self,inc:bool) -> String {
        if inc {
            if self.cursor==1048576 {
                return CELL_PTR_PAST_MAX.to_string();
            }   
            self.cursor+=1
        } else {
            if self.cursor==0 {
                return CELL_PTR_BELOW_MAX.to_string();
            }
            self.cursor-=1
        }
        String::new()
    }
    fn execute_loop(&mut self,input:Vec<Commands>,mut stdin:&mut Stdin) -> String {
        let mut i = 0;
        let mut err_val = String::new();
        loop {
            if self.buf[self.cursor]==0 {break}
            while i<input.len() {
                let instruction = &input[i];
                match instruction {
                    Commands::Read=>{
                        let mut stdin = stdin.lock();
                        let res = stdin.fill_buf().unwrap()[0];
                        stdin.consume(1);
                        self.buf[self.cursor]=res;
                        self.execution_log.push_str(&format!("Read: {:?}\n",char::from(res)));
                    },
                    Commands::Print=>{
                        print!("{}",char::from(self.buf[self.cursor]));
                        self.execution_log.push_str(&format!("Out: {:?}\n",char::from(self.buf[self.cursor])));
                    },
                    Commands::Loop(val)=>{
                        self.execution_log.push_str("Start new loop\n");
                        err_val = self.execute_loop(val.to_vec(),&mut stdin);
                    },
                    Commands::IncPtr=>{
                        err_val = self.move_cell_ptr(true);
                    },
                    Commands::DecPtr=>{
                        err_val = self.move_cell_ptr(false);
                    },
                    Commands::IncCell=>{
                        err_val = self.change_cell(true);
                    },
                    Commands::DecCell=>{
                        err_val = self.change_cell(false);
                    },
                    Commands::FunctionCaller=>{
                        let id = (self.buf[self.cursor],self.buf[self.cursor+1]);
                        if let Some(_) = self.functions.get(&id) {
                            err_val = self.execute_function(id,&mut stdin);
                        }
                    },
                    Commands::Function(_,_)=>{err_val = NOT_REMOVE_FUNC.to_string();},
                }
                self.instruction+=1;
                if err_val.len()>0 {return err_val;}
                i+=1;
                self.execution_log.push_str(&format!("{}:{}:{}\n",instruction.c(),self.cursor,self.buf[self.cursor]));
            }
            i=0;
            if self.buf[self.cursor]==0 {self.execution_log.push_str("End loop\n");break}
            else {self.execution_log.push_str("Restart loop\n")}
        }
        String::new()
    }
    fn execute_function(&mut self,id:(u8,u8),mut stdin:&mut Stdin) -> String {
        let instructions = self.functions.get(&id).unwrap().clone();
        let mut i = 0;
        let mut err_val = String::new();
        use Commands::*;
        while i<instructions.len() {
            let instruction = &instructions[i];
            match instruction {
                IncCell=>{err_val = self.change_cell(true);},
                DecCell=>{err_val = self.change_cell(false);},
                IncPtr=>{err_val = self.move_cell_ptr(true);},
                DecPtr=>{err_val = self.move_cell_ptr(false);},
                Print=>{
                    print!("{}",char::from(self.buf[self.cursor]));
                    self.execution_log.push_str(&format!("Out: {}",char::from(self.buf[self.cursor])));
                },
                Read=>{
                    let mut stdin = stdin.lock();
                    let buf = stdin.fill_buf().unwrap();
                    let res = buf[0];
                    stdin.consume(1);
                    self.buf[self.cursor] = res;
                    self.execution_log.push_str(&format!("Read: {}",char::from(res)));
                },
                FunctionCaller=>{
                    let id = (self.buf[self.cursor],self.buf[self.cursor+1]);
                    if let Some(_) = self.functions.get(&id) {
                        err_val = self.execute_function(id,&mut stdin);
                    }
                },
                Loop(val) =>{err_val = self.execute_loop(val.clone(),&mut stdin)},
                Function(_,_)=>{err_val = FUNC_IN_FUNC.to_string();},
            }
            self.execution_log.push_str(&format!("{}:{}:{}\n",instruction.c(),self.cursor,self.buf[self.cursor]));
            if err_val.len()>0 {return err_val}
            i+=1;
        }
        String::new()
    }
    fn new_instructions(&mut self, instructions:String) -> Result<&mut Memory,String> {
        let instructions_raw = Commands::parse(instructions);
        let mut instructions;
        match instructions_raw {
            Ok(val)=>{instructions=val;},
            Err(val)=>{return Err(val);},
        }
        self.instructions = instructions;
        Ok(self)
    }
    fn write_log(&self) {
        let mut log = OpenOptions::new().read(true).write(true).append(false).create(true).open("./execution.log").unwrap();
        log.set_len(0).unwrap();
        log.write(
            format!(
                "{}{}\n\n{}",
                INFO_IN_EXE_LOG,
                self.instructions.len(),
                self.execution_log
            ).as_bytes()
        ).unwrap();
    }
}


fn main() {
    let mut args:Vec<String> = env::args().collect();
    args.remove(0);
    let mut mem = Memory::new();
    if args.len()==0 {
        println!("You didn't put in file name(s)!");
    }
    for arg in args {
        match mem.new_instructions(fs::read_to_string(arg).unwrap()) {
            Ok(mem)=>{
                match mem.run() {
                    Ok(())=>{},
                    Err(err)=>{println!("{}",err);break},
                }
            },
            Err(err)=>{println!("{}",err);break},
        }
    }
    mem.write_log();
}