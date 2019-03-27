use std::io::{Cursor, Write};

pub trait Encode {
    type Error;
    fn encode<T: Write>(&self, writer: &mut T) -> ::std::result::Result<(), Self::Error>;
}

pub fn encode_into_buffer<'a, 'b, T>(
    value: &'a T,
    buf: &'b mut [u8],
) -> ::std::result::Result<&'b mut [u8], T::Error>
where
    T: Encode,
{
    let (length, buf2) = {
        let mut cursor = Cursor::new(buf);
        value.encode(&mut cursor)?;
        (cursor.position() as usize, cursor.into_inner())
    };

    Ok(&mut buf2[..length])
}
