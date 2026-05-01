use std::io::Read;

use anyhow::{Context, Result};

mod varint;

pub fn decompress<R: Read>(mut r: R) -> Result<Vec<u8>> {
    let len: u32 = varint::read(&mut r).context("read total len")?;
    let len = usize::try_from(len).unwrap();
    let mut out = Vec::with_capacity(len);

    while out.len() < len {
        let mut buf = [0];
        r.read_exact(&mut buf)
            .with_context(|| format!("unexpected EOF; decoded {} of {} bytes", out.len(), len))?;
        let tag = buf[0];

        if tag & 0x3 != 0x0 {
            todo!("non-literal element");
        }

        let lit_len: u32 = read_literal_len(&mut r, tag).context("read literal len")?;
        let lit_len = usize::try_from(lit_len).unwrap();
        let curr_offset = out.len();
        out.resize(curr_offset + lit_len, 0);
        r.read_exact(&mut out[curr_offset..])
            .context("EOF while reading literal")?;
    }

    Ok(out)
}

fn read_literal_len<R: Read>(mut r: R, tag: u8) -> Result<u32> {
    debug_assert_eq!(tag & 0x3, 0x0);
    let width = match tag >> 2 {
        60 => 1,
        61 => 2,
        62 => 3,
        63 => 4,
        n => return Ok(u32::from(n) + 1),
    };

    let mut buf = [0; 4];
    r.read_exact(&mut buf[..width]).context("unexpected EOF")?;

    let len = u32::from_le_bytes(buf);
    Ok(len)
}
