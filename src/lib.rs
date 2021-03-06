#![feature(try_from)]
#![allow(dead_code)]

extern crate byteorder;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::convert::TryFrom;
use std::io::{self, Error, ErrorKind, SeekFrom, prelude::*};
use version::Version;

pub mod version;
use version::PiecewiseVersion;

const MAGIC1: u8 = 0xda;
const MAGIC2: u8 = 0x27;
const CHARSET_FAMILY: u8 = 0;
const CHAR_SIZE: u8 = 2;

const MAGIC_NUMBER_CHECK_FAILED: &str = "ICU data file error: Not an ICU data file";
const HEADER_CHECK_FAILED: &str = "ICU data file error: Header authentication failed, please check if you have a valid ICU data file";

/// Indices for the indexes[] array, located after the header, and
/// directly after the root resource.
const RES_INDEX_LENGTH: u64 = 0;
const RES_INDEX_KEYS_TOP: u64 = 1;
const RES_INDEX_RESOURCES_TOP: u64 = 2;
const RES_INDEX_BUNDLE_TOP: u64 = 3;
const RES_INDEX_MAX_TABLE_LENGTH: u64 = 4;
const RES_INDEX_ATTRIBUTES: u64 = 5;
const RES_INDEX_16BIT_TOP: u64 = 6;
const RES_INDEX_POOL_CHECKSUM: u64 = 7;

// resource attribute bits
const RES_ATT_NO_FALLBACK: u32 = 1;
const RES_ATT_IS_POOL_BUNDLE: u32 = 2;
const RES_ATT_USES_POOL_BUNDLE: u32 = 4;

#[derive(Clone, Copy, Debug)]
pub enum DataFormat {
    // "ResB"
    ResourceBundle = 0x5265_7342,
    // "UCol"
    Collation = 0x5543_6f6c,
    // "Dict
    Dictionary = 0x4469_6374,
    // "CmnD"
    Dat = 0x436d_6e44,
    // "Nrm2"
    Normalized2 = 0x4e72_6d32,
    // "UPro"
    CharacterProperty = 0x5550_726f,
    // "Brk "
    BreakIteration = 0x4272_6b20,
    // "Cfu "
    Spoof = 0x4366_7520,
    // "SPDR"
    StringPrep = 0x5350_5250,
    // "BiDi"
    BiDi = 0x4269_4469,
    // "cAsE"
    Case = 0x6341_5345,
    // "unam"
    CharacterName = 0x756e_616d,
    // "CvAl"
    ConverterAlias = 0x4376_416c,
    // "cnvt"
    Converter = 0x636e_7674,
    // "pnam"
    PropertyAlias = 0x706e_616d,
}

impl DataFormat {
    fn is_acceptable_version(&self, format_version: [u8; 4]) -> bool {
        use DataFormat::*;
        match *self {
            ResourceBundle => {
                (format_version[0] == 1 && format_version[1] >= 1) || format_version[0] == 2
                    || format_version[0] == 3
            }
            Collation => format_version[0] == 5,
            Dictionary => true,
            Dat => format_version[0] == 1,
            Normalized2 => format_version[0] == 3,
            CharacterProperty => format_version[0] == 7,
            BreakIteration => {
                let ver = u32::from(format_version[0]) << 24 + u32::from(format_version[1])
                    << 16 + u32::from(format_version[2])
                    << 8 + u32::from(format_version[3]);
                ver == 0x04000000
            }
            Spoof => {
                format_version[0] == 2 || format_version[1] != 0 || format_version[2] != 0
                    || format_version[3] != 0
            }
            StringPrep => {
                format_version[0] == 0x3 && format_version[2] == 0x5 && format_version[3] == 0x2
            }
            BiDi => format_version[0] == 2,
            Case => format_version[0] == 3,
            CharacterName => format_version[0] == 1,
            ConverterAlias => {
                format_version[0] == 3 && format_version[1] == 0 && format_version[2] == 1
            }
            Converter => format_version[0] == 6,
            PropertyAlias => format_version[0] == 2,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Order {
    BigEndian,
    LittleEndian,
}

#[derive(Clone, Copy, Debug)]
pub struct OrderedReader<R>
where
    R: Read + Seek,
{
    reader: R,
    order: Order,
}

impl<R> OrderedReader<R>
where
    R: Read + Seek,
{
    pub fn wrap(reader: R, order: Order) -> OrderedReader<R> {
        OrderedReader { reader, order }
    }
}

impl<R> Read for OrderedReader<R>
where
    R: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.reader.read(buf)
    }
}

impl<R> Seek for OrderedReader<R>
where
    R: Read + Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Error> {
        self.reader.seek(pos)
    }
}

impl<R> EndianReader for OrderedReader<R>
where
    R: Read + Seek,
{
    fn order(&self) -> Order {
        self.order
    }
}

trait EndianReader: Read + Seek {
    fn order(&self) -> Order;

    fn read_u16(&mut self) -> Result<u16, io::Error> {
        match self.order() {
            Order::BigEndian => <Self as ReadBytesExt>::read_u16::<BigEndian>(self),
            Order::LittleEndian => <Self as ReadBytesExt>::read_u16::<LittleEndian>(self),
        }
    }

    fn read_u32(&mut self) -> Result<u32, io::Error> {
        match self.order() {
            Order::BigEndian => <Self as ReadBytesExt>::read_u32::<BigEndian>(self),
            Order::LittleEndian => <Self as ReadBytesExt>::read_u32::<LittleEndian>(self),
        }
    }

    fn read_u8_from(&mut self, pos: SeekFrom) -> Result<u8, io::Error> {
        self.seek(pos)?;
        <Self as ReadBytesExt>::read_u8(self)
    }

    fn read_u16_from(&mut self, pos: SeekFrom) -> Result<u16, io::Error> {
        self.seek(pos)?;
        self.read_u16()
    }

    fn read_u32_from(&mut self, pos: SeekFrom) -> Result<u32, io::Error> {
        self.seek(pos)?;
        self.read_u32()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ResourceBundleReader<R>
where
    R: Read + Seek,
{
    reader: OrderedReader<R>,
    data_version: Version,
    root_resource: u32,
    no_fallback: bool,
    is_pool_bundle: bool,
    uses_pool_bundle: bool,
    pool_string_index_limit: u32,
    pool_string_index_16_limit: u32,
}

#[allow(unused_variables)]
impl<R> ResourceBundleReader<R>
where
    R: Read + Seek,
{
    pub fn try_init(mut bytes: R, data_format: DataFormat) -> io::Result<ResourceBundleReader<R>> {
        check_magic(&mut bytes)?;
        let order = if read_endianness(&mut bytes)? == 1 {
            Order::BigEndian
        } else {
            Order::LittleEndian
        };
        let mut reader = OrderedReader::wrap(bytes, order);
        let (header_size, data_version) = read_header(&mut reader, data_format)?;
        let root_resource = <OrderedReader<R> as EndianReader>::read_u32(&mut reader)?;
        let offset = |n| {
            header_size as u64 + 4 + n
        };

        let indexes_0 = EndianReader::read_u32(&mut reader)?;
        let indexes_length = indexes_0 & 0xff;
        if indexes_length as u64 <= RES_INDEX_MAX_TABLE_LENGTH {
            return Err(Error::new(ErrorKind::InvalidData, "not enough indexes"));
        }

        let mut no_fallback = false;
        let mut is_pool_bundle = false;
        let mut uses_pool_bundle = false;
        let mut pool_string_index_limit = 0;
        let mut pool_string_index_16_limit = 0;

        reader.seek(SeekFrom::Start(offset(RES_INDEX_BUNDLE_TOP)))?;
        let max_offset = EndianReader::read_u32(&mut reader)? - 1;

        reader.seek(SeekFrom::Start(16))?;
        let file_format_major_version = reader.read_u8()?;
        if file_format_major_version >= 3 {
            pool_string_index_limit = indexes_0 >> 8;
        }

        if indexes_length as u64 > RES_INDEX_ATTRIBUTES {
            reader.seek(SeekFrom::Start(offset(RES_INDEX_ATTRIBUTES)))?;
            let att = EndianReader::read_u32(&mut reader)?;
            no_fallback = (att & RES_ATT_NO_FALLBACK) != 0;
            is_pool_bundle = (att & RES_ATT_IS_POOL_BUNDLE) != 0;
            uses_pool_bundle = (att & RES_ATT_USES_POOL_BUNDLE) != 0;
            pool_string_index_limit |= (att & 0xf000) << 12; // bits 15..12 -> 27..24
            pool_string_index_16_limit = att >> 16;
        }

        let key_bytes: Vec<u8>;
        let mut local_key_limit = 0;

        let keys_bottom = 1 + indexes_length;
        reader.seek(SeekFrom::Start(offset(RES_INDEX_KEYS_TOP)))?;
        let keys_top = EndianReader::read_u32(&mut reader)?;
        if keys_top > keys_bottom {
            if is_pool_bundle {
                key_bytes = Vec::with_capacity((keys_top - keys_bottom) as usize);
            } else {
                local_key_limit = (keys_top as usize) << 2;
                key_bytes = Vec::with_capacity(local_key_limit);
            }
            
        } else {
            key_bytes = Vec::new();
        }

        Ok(ResourceBundleReader {
            reader,
            data_version: Version::try_from(data_version)?,
            root_resource,
            no_fallback,
            is_pool_bundle,
            uses_pool_bundle,
            pool_string_index_limit,
            pool_string_index_16_limit,
        })
    }

    pub fn version(&self) -> Version {
        self.data_version
    }

    pub fn root_resource(&self) -> u32 {
        self.root_resource
    }
}

pub fn read_header<R>(
    reader: &mut OrderedReader<R>,
    data_format: DataFormat,
) -> io::Result<(u16, PiecewiseVersion)>
where
    R: Read + Seek,
{
    let header_size = read_header_size(reader)?;
    validate_format_version(reader, data_format)?;

    let data_version = read_data_version(reader)?;
    reader.seek(SeekFrom::Start(header_size.into()))?;
    Ok((header_size, data_version))
}

fn read_data_version<R>(reader: &mut OrderedReader<R>) -> io::Result<PiecewiseVersion>
where
    R: Read + Seek,
{
    reader.seek(SeekFrom::Start(20))?;
    let data_version = (
        reader.read_u8()?,
        reader.read_u8()?,
        reader.read_u8()?,
        reader.read_u8()?,
    );
    Ok(data_version)
}

fn read_header_size<R>(reader: &mut OrderedReader<R>) -> io::Result<u16>
where
    R: Read + Seek,
{
    let header_size = reader.read_u16_from(SeekFrom::Start(0))?;
    let data_info_size = reader.read_u16_from(SeekFrom::Start(4))?;
    if data_info_size < 20 || header_size < (data_info_size + 4) {
        return Err(Error::new(ErrorKind::InvalidData, "header size error"));
    }
    Ok(header_size)
}

fn check_magic<B>(bytes: &mut B) -> io::Result<()>
where
    B: Read + Seek,
{
    bytes.seek(SeekFrom::Start(2))?;
    let magic1 = bytes.read_u8()?;
    let magic2 = bytes.read_u8()?;
    if magic1 != MAGIC1 || magic2 != MAGIC2 {
        Err(Error::new(
            ErrorKind::InvalidData,
            MAGIC_NUMBER_CHECK_FAILED,
        ))
    } else {
        Ok(())
    }
}

fn read_endianness<B>(bytes: &mut B) -> io::Result<u8>
where
    B: Read + Seek,
{
    bytes.seek(SeekFrom::Start(8))?;
    let big_endian = bytes.read_u8()?;
    let charset_family = bytes.read_u8()?;
    let char_size = bytes.read_u8()?;
    if big_endian > 1 || charset_family != CHARSET_FAMILY || char_size != CHAR_SIZE {
        Err(Error::new(ErrorKind::InvalidData, HEADER_CHECK_FAILED))
    } else {
        Ok(big_endian)
    }
}

fn validate_format_version<R>(
    reader: &mut OrderedReader<R>,
    data_format: DataFormat,
) -> io::Result<()>
where
    R: Read + Seek,
{
    let val = data_format as u32;
    reader.seek(SeekFrom::Start(12))?;
    let df = [
        reader.read_u8()?,
        reader.read_u8()?,
        reader.read_u8()?,
        reader.read_u8()?,
    ];
    if df[0] != ((val >> 24) as u8) || df[1] != ((val >> 16) as u8) || df[2] != ((val >> 8) as u8)
        || df[3] != (val as u8)
    {
        return Err(Error::new(ErrorKind::InvalidInput, HEADER_CHECK_FAILED));
    }
    // format version starts at 16
    let format_version = [
        reader.read_u8()?,
        reader.read_u8()?,
        reader.read_u8()?,
        reader.read_u8()?,
    ];
    if !data_format.is_acceptable_version(format_version) {
        // TODO print data format and format_version bytes with error message
        return Err(Error::new(ErrorKind::InvalidData, HEADER_CHECK_FAILED));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use DataFormat;
    use ResourceBundleReader;
    use std::io::Cursor;
    use version::Version;
    #[test]
    fn read_header_doesnt_fail() {
        // header from a real resource bundle
        let mut c = Cursor::new(vec![
            0x0, 0x20, 0xda, 0x27,
            0x0, 0x14, 0x0, 0x0,
            0x1, 0x0, 0x02, 0x0,
            0x52, 0x65, 0x73, 0x42,
            0x03, 0x0, 0x0, 0x0,
            0x01, 0x04, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0,
            0x20, 0x0, 0x18, 0x78,
            0x0, 0xcb, 0x92, 0x08,
            0x0, 0x0, 0x0, 0x09,
            0x0, 0x0, 0x18, 0x92,
        ]);
        let r = ResourceBundleReader::try_init(&mut c, DataFormat::ResourceBundle)
            .expect("Failed to read header");
        assert_eq!(r.version(), Version::Unicode10_0((0x01, 0x04, 0x0, 0x0)));
    }
}
