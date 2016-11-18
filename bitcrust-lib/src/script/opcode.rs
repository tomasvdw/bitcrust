/*
 * 2016 Tomas van der Wansem

 * CURRENTLY ONLY USED FOR DEBUG-RENDERING
 * Actual script processing is done in
 */
 

use std::io;


use script::ScriptError;
use script::context::Context;


use super::opcode_pushdata::*;



pub struct OpCode {
        
    pub name:        &'static str,
    pub execute:     fn(&mut Context) -> Result<(), ScriptError>,
    pub skip:        fn(ctx: &mut Context) -> Result<(), ScriptError>,
    pub display:     fn(ctx: &mut Context, writer: &mut io::Write) -> io::Result<()>    
}


fn skip_none(_: &mut Context) -> Result<(), ScriptError> {
    Ok(())
}

fn skip_invalid(_: &mut Context) -> Result<(), ScriptError> {
    Err(ScriptError::InvalidOpcode)
}


fn disp_name(ctx: &mut Context, writer: &mut io::Write) -> io::Result<()> {
    let opcode = &OPCODES[ctx.script1[ctx.ip] as usize];
    write!(writer, "{} ", opcode.name)
}

fn disp_invalid(_: &mut Context, writer: &mut io::Write) -> io::Result<()> {
    write!(writer, " [INVALID-OPCODE] ")
}


fn op_false(ctx: &mut Context) -> Result<(), ScriptError> {

    // push empty array
    ctx.stack.push(Box::new([]))
}

fn op_nop(_: &mut Context) -> Result<(), ScriptError> {
    Ok(())
}

fn op_add(ctx: &mut Context) -> Result<(), ScriptError> {
    let n1 = { try!(ctx.stack.pop_scriptnum()) };
    let n2 = { try!(ctx.stack.pop_scriptnum()) };
     
    ctx.stack.push_scriptnum(n1 + n2)   
    
}

pub fn run()  {
    println!("{}", OPCODES[0].name);
}
    
fn op_unimplemented(_: &mut Context) -> Result<(), ScriptError> {
    unimplemented!()
}

fn skip_unimplemented(_: &mut Context) -> Result<(), ScriptError> {
    unimplemented!()
}


fn op_invalid(_: &mut Context) -> Result<(), ScriptError> {
    Err(ScriptError::InvalidOpcode)
}

pub static OPCODES: [OpCode; 256] = [
    //0-7
    OpCode { name: "OP_FALSE", display: disp_name,                     execute: op_false,                    skip: skip_none     },
    OpCode { name: "",         display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "",         display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "",         display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "",         display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "",         display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "",         display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "",         display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    
    //8-15
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    
    //16-23
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    
    //24-31
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    
    //32-39
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    
    //40-47
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    
    //48-55
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    
    //56-63
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    
    //64-71
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    OpCode { name: "", display: disp_pushdata_count_by_opcode, execute: op_pushdata_count_by_opcode, skip: skip_pushdata_count_by_opcode },
    
    //72-79
    OpCode { name: "",           display: disp_pushdata_count_by_opcode,     execute: op_pushdata_count_by_opcode,      skip: skip_pushdata_count_by_opcode },
    OpCode { name: "",           display: disp_pushdata_count_by_opcode,     execute: op_pushdata_count_by_opcode,      skip: skip_pushdata_count_by_opcode },
    OpCode { name: "",           display: disp_pushdata_count_by_opcode,     execute: op_pushdata_count_by_opcode,      skip: skip_pushdata_count_by_opcode },
    OpCode { name: "",           display: disp_pushdata_count_by_opcode,     execute: op_pushdata_count_by_opcode,      skip: skip_pushdata_count_by_opcode },
    OpCode { name: "",           display: disp_pushdata_count_by_next_bytes, execute: op_pushdata_count_by_next_bytes,  skip: skip_pushdata_count_by_next_bytes },
    OpCode { name: "",           display: disp_pushdata_count_by_next_bytes, execute: op_pushdata_count_by_next_bytes,  skip: skip_pushdata_count_by_next_bytes },
    OpCode { name: "",           display: disp_pushdata_count_by_next_bytes, execute: op_pushdata_count_by_next_bytes,  skip: skip_pushdata_count_by_next_bytes },
    OpCode { name: "OP_1NEGATE", display: disp_name,                         execute: op_pushdata_value_by_opcode,      skip: skip_none     },
    
    //80-87
    OpCode { name: "",         display: disp_invalid,                   execute: op_invalid,                   skip: skip_none     },
    OpCode { name: "OP_TRUE",  display: disp_name,                      execute: op_pushdata_value_by_opcode,  skip: skip_none     },
    OpCode { name: "",         display: disp_pushdata_value_by_opcode,  execute: op_pushdata_value_by_opcode,  skip: skip_none     },
    OpCode { name: "",         display: disp_pushdata_value_by_opcode,  execute: op_pushdata_value_by_opcode,  skip: skip_none     },
    OpCode { name: "",         display: disp_pushdata_value_by_opcode,  execute: op_pushdata_value_by_opcode,  skip: skip_none     },
    OpCode { name: "",         display: disp_pushdata_value_by_opcode,  execute: op_pushdata_value_by_opcode,  skip: skip_none     },
    OpCode { name: "",         display: disp_pushdata_value_by_opcode,  execute: op_pushdata_value_by_opcode,  skip: skip_none     },
    OpCode { name: "",         display: disp_pushdata_value_by_opcode,  execute: op_pushdata_value_by_opcode,  skip: skip_none     },
    
    //88-95
    OpCode { name: "",         display: disp_pushdata_value_by_opcode,  execute: op_pushdata_value_by_opcode,  skip: skip_none     },
    OpCode { name: "",         display: disp_pushdata_value_by_opcode,  execute: op_pushdata_value_by_opcode,  skip: skip_none     },
    OpCode { name: "",         display: disp_pushdata_value_by_opcode,  execute: op_pushdata_value_by_opcode,  skip: skip_none     },
    OpCode { name: "",         display: disp_pushdata_value_by_opcode,  execute: op_pushdata_value_by_opcode,  skip: skip_none     },
    OpCode { name: "",         display: disp_pushdata_value_by_opcode,  execute: op_pushdata_value_by_opcode,  skip: skip_none     },
    OpCode { name: "",         display: disp_pushdata_value_by_opcode,  execute: op_pushdata_value_by_opcode,  skip: skip_none     },
    OpCode { name: "",         display: disp_pushdata_value_by_opcode,  execute: op_pushdata_value_by_opcode,  skip: skip_none     },
    OpCode { name: "",         display: disp_pushdata_value_by_opcode,  execute: op_pushdata_value_by_opcode,  skip: skip_none     },
    
    //96-103
    OpCode { name: "",         display: disp_pushdata_value_by_opcode,  execute: op_pushdata_value_by_opcode,    skip: skip_none     },
    OpCode { name: "OP_NOP",   display: disp_name,                      execute: op_nop,                         skip: skip_none     },
    OpCode { name: "",         display: disp_invalid,                   execute: op_invalid,                     skip: skip_none     },
    OpCode { name: "OP_IF",    display: disp_name,                      execute: op_unimplemented,               skip: skip_unimplemented },
    OpCode { name: "OP_NOTIF", display: disp_name,                      execute: op_unimplemented,               skip: skip_unimplemented },
    OpCode { name: "",         display: disp_invalid,                   execute: op_invalid,                     skip: skip_invalid     },
    OpCode { name: "",         display: disp_invalid,                   execute: op_invalid,                     skip: skip_invalid     },
    OpCode { name: "OP_ELSE",  display: disp_name,                      execute: op_unimplemented,               skip: skip_unimplemented },

    //104-111
    OpCode { name: "OP_ENDIF",           display: disp_name,            execute: op_unimplemented,      skip: skip_unimplemented },
    OpCode { name: "OP_VERIFY",          display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_RETURN",          display: disp_name,            execute: op_invalid,            skip: skip_none },
    OpCode { name: "OP_TOALTSTACK",      display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_FROMALTSTACK",    display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_2DROP",           display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_2DUP",            display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_3DUP",            display: disp_name,            execute: op_unimplemented,      skip: skip_none },

    //112-119
    OpCode { name: "OP_2OVER",            display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_2ROT",             display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_2SWAP",            display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_IFDUP",            display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_DEPTH",            display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_DROP",             display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_DUP",              display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_NIP",              display: disp_name,            execute: op_unimplemented,      skip: skip_none },

    //120-127
    OpCode { name: "OP_OVER",             display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_PICK",             display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_ROLL",             display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_ROT",              display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_SWAP",             display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "OP_TUCK",             display: disp_name,            execute: op_unimplemented,      skip: skip_none },
    OpCode { name: "",                    display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                    display: disp_invalid,         execute: op_invalid,            skip: skip_none     },

    //128-135
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "OP_SIZE",            display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "OP_EQUAL",           display: disp_name,            execute: op_unimplemented,      skip: skip_none     },

    //136-143
    OpCode { name: "OP_EQUALVERIFY",     display: disp_name,         execute: op_unimplemented,         skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "OP_1ADD",            display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_1SUB",            display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "OP_NEGATE",          display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    
    //144-151
    OpCode { name: "OP_ABS",             display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_NOT",             display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_0NOTEQUAL",       display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_ADD",             display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_SUB",             display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    
    //152-159
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "OP_BOOLAND",         display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_BOOLOR",          display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_NUMEQUAL",        display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_NUMEQUALVERIFY",  display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_NUMNOTEQUAL",     display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_LESSTHEN",        display: disp_name,            execute: op_unimplemented,      skip: skip_none     },

    //160-167
    OpCode { name: "OP_GREATERTHEN",        display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_LESSTHANOREQUAL",    display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_GREATERTHANOREQUAL", display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_MIN",                display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_MAX",                display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_WITHIN",             display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_RIPEMD160",          display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_SHA1",               display: disp_name,            execute: op_unimplemented,      skip: skip_none     },

    //168-175
    OpCode { name: "OP_SHA256",              display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_HASH160",             display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_HASH256",             display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_CODESEPARATOR",       display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_CHECKSIG",            display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_CHECKSIGVERIFY",      display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_CHECKMULTISIG",       display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_CHECKMULTISIGVERIFY", display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    
    //176-183
    OpCode { name: "OP_NOP1",                 display: disp_name,            execute: op_nop,                skip: skip_none     },
    OpCode { name: "OP_CHECKLOCKTIMEVERIFY", display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_CHECKSEQUENCEVERIFY", display: disp_name,            execute: op_unimplemented,      skip: skip_none     },
    OpCode { name: "OP_NOP4",                 display: disp_name,            execute: op_nop,                skip: skip_none     },
    OpCode { name: "OP_NOP5",                 display: disp_name,            execute: op_nop,                skip: skip_none     },
    OpCode { name: "OP_NOP6",                 display: disp_name,            execute: op_nop,                skip: skip_none     },
    OpCode { name: "OP_NOP7",                 display: disp_name,            execute: op_nop,                skip: skip_none     },
    OpCode { name: "OP_NOP8",                 display: disp_name,            execute: op_nop,                skip: skip_none     },

    //184-185
    OpCode { name: "OP_NOP9",                 display: disp_name,            execute: op_nop,                skip: skip_none     },
    OpCode { name: "OP_NOP10",                display: disp_name,            execute: op_nop,                skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    
    //192-199
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    
    //200-207
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    
    //208-215
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    
    //216-223
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    
    //224-231
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    
    //232-239
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    
    //240-247
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    
    //247-255
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    OpCode { name: "",                   display: disp_invalid,         execute: op_invalid,            skip: skip_none     },
    
];


mod tests {
    #![cfg(test)]
    use ::script::context::Context;

    
    #[test]
    fn test_op_false()
    {
        let script = vec![0x00];
        let _ = Context::new(&script);
        
        
    }
}