use std::io;

use crate::ReadEx;

pub const DDS_SIGNATURE: &str = "DDS ";

#[repr(C, packed)]
pub struct Header {
    pub size: u32,
    pub flags: u32,
    pub height: u32,
    pub width: u32,
    pub pitch_or_linear_size: u32,
    pub depth: u32,
    pub mip_map_count: u32,
    pub reserved_1: [u32; 11],
    pub pixel_format: PixelFormat,
    pub caps: u32,
    pub caps2: u32,
    pub caps3: u32,
    pub caps4: u32,
    pub reserved_2: u32,
}

impl Header {
    pub fn read(r: &mut impl io::BufRead) -> io::Result<Self> {
        let mut hdr: Self = r.read_c_struct()?;

        if cfg!(target_endian = "big") {
            hdr.size = hdr.size.swap_bytes();
            hdr.flags = hdr.flags.swap_bytes();
            hdr.height = hdr.height.swap_bytes();
            hdr.width = hdr.width.swap_bytes();
            hdr.pitch_or_linear_size = hdr.pitch_or_linear_size.swap_bytes();
            hdr.depth = hdr.depth.swap_bytes();
            hdr.mip_map_count = hdr.mip_map_count.swap_bytes();
            hdr.caps = hdr.caps.swap_bytes();
            hdr.caps2 = hdr.caps2.swap_bytes();
            hdr.caps3 = hdr.caps3.swap_bytes();
            hdr.caps4 = hdr.caps4.swap_bytes();

            hdr.pixel_format.size = hdr.pixel_format.size.swap_bytes();
            hdr.pixel_format.flags = hdr.pixel_format.flags.swap_bytes();
            hdr.pixel_format.four_cc = hdr.pixel_format.four_cc.swap_bytes();
            hdr.pixel_format.rgb_bit_count = hdr.pixel_format.rgb_bit_count.swap_bytes();
            hdr.pixel_format.rbit_mask = hdr.pixel_format.rbit_mask.swap_bytes();
            hdr.pixel_format.gbit_mask = hdr.pixel_format.gbit_mask.swap_bytes();
            hdr.pixel_format.bbit_mask = hdr.pixel_format.bbit_mask.swap_bytes();
            hdr.pixel_format.abit_mask = hdr.pixel_format.abit_mask.swap_bytes();
        }

        Ok(hdr)
    }

    pub fn write(&self, w: &mut impl io::Write) -> io::Result<()> {
        todo!();
    }
}

#[repr(C, packed)]
pub struct PixelFormat {
    size: u32,
    flags: u32,
    four_cc: u32,
    rgb_bit_count: u32,
    rbit_mask: u32,
    gbit_mask: u32,
    bbit_mask: u32,
    abit_mask: u32,
}
