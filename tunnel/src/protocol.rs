
pub const HDR_LEN      : usize = 2;
pub const HDR_ANNOUNCE : u16 = 0xA9CE;
pub const HDR_ECHO     : u16 = 0xEC50;
pub const HDR_DATA     : u16 = 0xDA7A;

pub const SIGNATURE_LEN: usize = 16;
// 15E4C396-82B7-4406-9FD9-81E52CA9C82F
pub const SIGNATURE: u128 = 0x15E4C39682B744069FD981E52CA9C82F;

#[track_caller]
pub fn msg_announce(buf: &mut [u8]) -> &[u8]
{
    set_hdr(buf, HDR_ANNOUNCE);
    set_signature(buf);
    &buf[..HDR_LEN + SIGNATURE_LEN]
}

#[track_caller]
pub fn msg_echo(buf: &mut [u8]) -> &[u8]
{
    set_hdr(buf, HDR_ECHO);
    set_signature(buf);
    &buf[..HDR_LEN + SIGNATURE_LEN]
}

#[track_caller]
pub fn msg_data<'a>(buf: &'a mut [u8], port: u16, data: &[u8]) -> &'a [u8]
{
    set_hdr(buf, HDR_DATA);
    set_data_port(buf, port ^ 0xFFFF);
    set_data(buf, data);
    &buf[..HDR_LEN + 2 + data.len()]
}

#[track_caller]
fn set_hdr(buf: &mut [u8], hdr: u16)
{
    unsafe {
        buf[..2].as_mut_ptr().cast::<u16>().write_unaligned(hdr.to_le());
    }
}

#[track_caller]
fn set_signature(buf: &mut [u8])
{
    unsafe {
        buf[2..][..SIGNATURE_LEN].as_mut_ptr().cast::<u128>().write_unaligned(SIGNATURE.to_le());
    }
}

#[track_caller]
fn set_data_port(buf: &mut [u8], port: u16)
{
    unsafe {
        buf[HDR_LEN..].as_mut_ptr().cast::<u16>().write_unaligned(port.to_le());
    }
}

#[track_caller]
fn set_data(buf: &mut [u8], data: &[u8])
{
    buf[HDR_LEN..][2..][..data.len()].copy_from_slice(data);
}

pub enum Message<'a> {
    Announce,
    Echo,
    Data (
        u16,
        &'a [u8],
    ),
}

#[track_caller]
pub fn msg_kind(buf: &[u8]) -> Option<Message<'_>>
{
    if check_hdr(buf, HDR_DATA) {
        return Some (Message::Data (
            0xFFFF ^ u16::from_le(unsafe {
                buf[HDR_LEN..].as_ptr().cast::<u16>().read_unaligned()
            }),
            &buf[HDR_LEN+2..],
        ));
    }

    if check_hdr(buf, HDR_ECHO) {
        if !check_signature(buf) {
            return None;
        }
        return Some (Message::Echo);
    }

    if check_hdr(buf, HDR_ANNOUNCE) {
        if !check_signature(buf) {
            return None;
        }
        return Some (Message::Announce);
    }

    None
}

#[track_caller]
fn check_hdr(buf: &[u8], hdr: u16) -> bool
{
    buf.len() >= HDR_LEN && hdr == u16::from_le(unsafe {
        buf.as_ptr().cast::<u16>().read_unaligned()
    })
}

#[track_caller]
fn check_signature(buf: &[u8]) -> bool
{
    buf.len() >= HDR_LEN + SIGNATURE_LEN && SIGNATURE == u128::from_le(unsafe {
        buf[HDR_LEN..].as_ptr().cast::<u128>().read_unaligned()
    })
}
