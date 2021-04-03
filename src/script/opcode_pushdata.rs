

/*
 * 2016 Tomas van der Wansem
 * No rights reserved
 */

//! Implementing the opcodes that push data on the stack
//!
//! These have special display functions as they are rendered
//! as hex-strings instead of opcode names


use std::io;


use script::ScriptError;
use script::context::Context;


/// Maximum number of bytes to push to the stack at once
const MAX_SCRIPT_ELEMENT_SIZE: u64 = 520;


/// Pushes the next `n` bytes to the stack, where `n` is the current opcode
pub fn op_pushdata_count_by_opcode(ctx: &mut Context) -> Result<(), ScriptError> {
    let count = ctx.script1[ctx.ip] as u64;
    op_pushdata_next_bytes(ctx, count)
}

/// Skips the next `n` bytes where `n` is the current opcode
pub fn skip_pushdata_count_by_opcode(ctx: &mut Context) -> Result<(), ScriptError> {
    let count = ctx.script1[ctx.ip] as usize;
    ctx.ip += count;

    Ok(())
}

/// Renders the next `n` bytes to `writer` where `n` is the current opcode
pub fn disp_pushdata_count_by_opcode(ctx: &mut Context,  writer: &mut dyn io::Write) -> io::Result<()> {
    
    let count = ctx.script1[ctx.ip] as u64;
    
    disp_pushdata_next_bytes(ctx, writer, Ok(count))
    
}

pub fn op_pushdata_value_by_opcode(ctx: &mut Context) -> Result<(), ScriptError> {
    let value = ctx.script1[ctx.ip] as i32 - 0x80_i32;
    ctx.stack.push(Box::new([value as u8]))
}



pub fn disp_pushdata_value_by_opcode(ctx: &mut Context,  writer: &mut dyn io::Write) -> io::Result<()> {
    let value = ctx.script1[ctx.ip] as i32 - 0x80_i32;
    write!(writer, "{:02x}", value)
}


/// Returns the size uint that specifies the pushdata-length
/// for OP_PUSHDATA1,OP_PUSHDATA2 and OP_PUSHDATA4
fn size_from_opcode(ctx: &Context) -> u64 {
    match ctx.script1[ctx.ip] {
        76 => 1,
        77 => 2,
        78 => 4,
        _  => panic!("disp_pushdata_n called for incorrect opcode")
    }
}

/// Skips an OP_PUSHDATA1, OP_PUSHDATA2 and OP_PUSHDATA4
///
pub fn skip_pushdata_count_by_next_bytes(ctx: &mut Context) -> Result<(), ScriptError> {
    let size = { size_from_opcode(ctx) };
    let bytecount = ctx.next_uint(size)?;

    ctx.ip += bytecount as usize;

    Ok(())
}

/// Displays the pushdata for OP_PUSHDATA1, OP_PUSHDATA2 and OP_PUSHDATA4 as hex
///
pub fn disp_pushdata_count_by_next_bytes(ctx: &mut Context,  writer: &mut dyn io::Write) -> io::Result<()> {
    let size = { size_from_opcode(ctx) };
    let bytecount = ctx.next_uint(size);
    
    disp_pushdata_next_bytes(ctx, writer, bytecount)
}

/// Pushes the next `n` bytes where `n` is grabbed from the next bytes of the script
/// 
/// The number of bytes depends on the opcode being OP_PUSHDATA1, OP_PUSHDATA2, OP_PUSHDATA4
pub fn op_pushdata_count_by_next_bytes(ctx: &mut Context) -> Result<(), ScriptError> {
    let size = { size_from_opcode(ctx) };
    let bytecount = ctx.next_uint(size)?;

    op_pushdata_next_bytes(ctx, bytecount)
}

/// Helper to push the next `count` bytes to the stack
///
/// Can return an error if `count` is too large
fn op_pushdata_next_bytes(ctx: &mut Context,  count: u64) -> Result<(), ScriptError> {
    if count > MAX_SCRIPT_ELEMENT_SIZE {
        return Err(ScriptError::PushdataTooLarge);
    }

    // grab next bytes and box them
    // this means copying as we need ownership on the stack
    let bytes = ctx.next_bytes(count)?
        .to_vec()
        .into_boxed_slice();
    
    // and push them on the stack
    ctx.stack.push(bytes)
}


/// Internal helper to display the next `count` bytes to writer;
/// used to render one of the pushdata operations
fn disp_pushdata_next_bytes(ctx: &mut Context,  writer: &mut io::Write, count: Result<u64, ScriptError>) -> io::Result<()> {
    
    const UNEXPECTED_EOS: &'static str = "[UNEXPECTED-END-OF-SCRIPT]"; 
    const PUSHDATA_TOO_LARGE: &'static str  = "[PUSHDATA-TOO-LARGE]"; 

    if let Err(_) = count {
        return write!(writer, "{}", UNEXPECTED_EOS);
    }

    let count_val = count.unwrap();
    if count_val > MAX_SCRIPT_ELEMENT_SIZE {
        return write!(writer, "{}", PUSHDATA_TOO_LARGE);
        
    }
    
    match ctx.next_bytes(count_val) {
        Ok(bytes) => {
            for byte in bytes {
                write!(writer, "{:02x}", byte)?;
            }
        }
        Err(_) => {
            write!(writer, "{}", UNEXPECTED_EOS)?;
        }
    }

    Ok(())
}


#[cfg(test)]
mod test {

    use super::*;
    use std::io;
    use script::context::Context;

    fn test_pushdata(
        script:    Vec<u8>, 
        expected:  &'static str,
        disp_func: fn(ctx: &mut Context, writer: &mut io::Write) -> io::Result<()>) 
    {
        let buf: Vec<u8> = Vec::new();
    
        let mut wr = io::Cursor::new(buf);
    
        let mut ctx = Context::new(&script);
        disp_func(&mut ctx, &mut wr).unwrap();
        
        assert_eq!(String::from_utf8(wr.into_inner()).unwrap(), expected);
    }

    #[test]
    fn test_disp_pushdata_count_by_opcode() {
        
        test_pushdata(
            vec![0x04_u8, 1u8, 2u8, 3u8, 0u8],
            "01020300",
            disp_pushdata_count_by_opcode
        );
        
        test_pushdata(
            vec![0x04_u8, 1u8, 2u8, 3u8],
            "[UNEXPECTED-END-OF-SCRIPT]",
            disp_pushdata_count_by_opcode
        );

        test_pushdata(
            vec![0x04_u8, 1u8, 2u8, 3u8],
            "[UNEXPECTED-END-OF-SCRIPT]",
            disp_pushdata_count_by_opcode
        );

        
    }

    #[test]
    fn test_size_from_opcode() {
        assert_eq!(
            1, super::size_from_opcode(&mut Context::new(&[76]))
        );
        assert_eq!(
            2, super::size_from_opcode(&mut Context::new(&[77]))
        );
        assert_eq!(
            4, super::size_from_opcode(&mut Context::new(&[78]))
        );
    }
}