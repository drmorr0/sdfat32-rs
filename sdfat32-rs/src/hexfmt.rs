use ufmt::{
    uWrite,
    Formatter,
};

fn hexfmt_u8(n: u8) -> u8 {
    match n & 0xf {
        v if v <= 9 => (v as u8) + 48,
        v if v > 9 => (v as u8) + 87,
        _ => 63, // '?'
    }
}

pub(crate) fn hexfmt32_le<W>(serial: &mut Formatter<W>, mut n: u32) -> Result<(), W::Error>
where
    W: uWrite + ?Sized,
{
    serial.write_str("0x")?;
    let mut chars = [0u8; 8];
    for i in 0..4 {
        chars[7 - i * 2] = hexfmt_u8((n & 0x0f) as u8);
        n >>= 4;
        chars[6 - i * 2] = hexfmt_u8((n & 0x0f) as u8);
        n >>= 4;
    }
    for c in chars.iter() {
        serial.write_char(*c as char)?;
    }
    Ok(())
}
