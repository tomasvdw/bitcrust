/*
 * 2016 Tomas van der Wansem
 * No rights reserved
 */
 

use std::io;


use script::ScriptError;
use script::context::Context;


struct OpCode {
        
    name:     &'static str,
    execute:  fn(&mut Context) -> Result<(), ScriptError>,
    display:  fn(ctx: &mut Context, writer: &mut io::Write) -> io::Result<()>
}


fn op_false(ctx: &mut Context) -> Result<(), ScriptError> {
    ctx.stack.push(Vec::new())
}

fn op_add(ctx: &mut Context) -> Result<(), ScriptError> {
    let n1 = { try!(ctx.stack.pop_scriptnum()) };
    let n2 = { try!(ctx.stack.pop_scriptnum()) };
     
    ctx.stack.push_scriptnum(n1 + n2)
    
    
}


fn op_pushdata(ctx: &mut Context) -> Result<(), ScriptError> {
    Ok(())
}

fn disp_pushdata(ctx: &mut Context,  writer: &mut io::Write) -> io::Result<()> {
    
    // the size of the data to push is count
    let count = ctx.script1[ctx.ip];
    Result::Ok(())
    
}



fn disp_name(ctx: &mut Context, writer: &mut io::Write) -> io::Result<()> {
    let opcode = &OPCODES[ctx.script1[ctx.ip] as usize];
    write!(writer, "{} ", opcode.name)
}

static OP_FALSE : OpCode = OpCode { 
    name:    "OP_FALSE",
    display: disp_name,
    execute: op_false,
 };


static OP_PUSHDATA : OpCode = OpCode { 
    name: "OP_PUSHDATA",     
    execute: op_pushdata,        
    display: disp_pushdata 
};



static OPCODES: [&'static OpCode; 16] = [
    OpCode { name: "OP_FALSE",    display: disp_name,    execute: op_false,    skip: skip_single,   can_succeed: verify
 }
    
    &OP_FALSE,    &OP_PUSHDATA, &OP_PUSHDATA, &OP_PUSHDATA, &OP_PUSHDATA, &OP_PUSHDATA, &OP_PUSHDATA, &OP_PUSHDATA,
    &OP_PUSHDATA, &OP_PUSHDATA, &OP_PUSHDATA, &OP_PUSHDATA, &OP_PUSHDATA, &OP_PUSHDATA, &OP_PUSHDATA, &OP_PUSHDATA,
];


mod tests {
    #![cfg(test)]
    use ::script::context::Context;

    
    #[test]
    fn test_op_false()
    {
        let script = vec![0x00];
        let ctx = Context::create(script);
        
        
    }
}