use std::io::{self, Error, ErrorKind, SeekFrom, prelude::*};

extern crate byteorder;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

const MAGIC1: u8 = 0xda;
const MAGIC2: u8 = 0x27;
const CHARSET_FAMILY: u8 = 0;
const CHAR_SIZE: u8 = 2;

const MAGIC_NUMBER_CHECK_FAILED: &str = "ICU data file error: Not an ICU data file";
const HEADER_CHECK_FAILED: &str = "ICU data file error: Header authentication failed, please check if you have a valid ICU data file";

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
            ResourceBundle => (format_version[0] == 1 && format_version[1] >= 1) ||
                format_version[0] == 2 || format_version[0] == 3,
            Collation => format_version[0] == 5,
            Dictionary => true,
            Dat => format_version[0] == 1,
            Normalized2 => format_version[0] == 3,
            CharacterProperty => format_version[0] == 7,
            BreakIteration => {
                let ver = u32::from(format_version[0]) << 24 +
                    u32::from(format_version[1]) << 16 +
                    u32::from(format_version[2]) << 8 +
                    u32::from(format_version[3]);
                ver == 0x04000000
            },
            Spoof => (format_version[0] == 2 || format_version[1] != 0 || format_version[2] != 0 || format_version[3] != 0),
            StringPrep => (format_version[0] == 0x3 && format_version[2] == 0x5 && format_version[3] == 0x2),
            BiDi => format_version[0] == 2,
            Case => format_version[0] == 3,
            CharacterName => format_version[0] == 1,
            ConverterAlias => (format_version[0] == 3 && format_version[1] == 0 && format_version[2] == 1),
            Converter => format_version[0] == 6,
            PropertyAlias => format_version[0] == 2
        }
    }
}

#[derive(Debug)]
pub struct Header;

#[derive(Clone, Copy, Debug)]
pub enum Order {
    BigEndian,
    LittleEndian,
}

#[derive(Debug)]
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

pub fn read_header<B: Read + Seek>(bytes: &mut B, data_format: DataFormat) -> io::Result<u32> {
    check_magic(bytes)?;
    let big_endian = read_endianness(bytes)?;
    let order = if big_endian == 1 {
        Order::BigEndian
    } else {
        Order::LittleEndian
    };
    let mut reader = OrderedReader::wrap(bytes, order);

    let header_size = read_header_size(&mut reader)?;
    validate_format_version(&mut reader, data_format)?;

    let data_version = read_data_version(&mut reader)?;
    reader.seek(SeekFrom::Start(header_size.into()))?;
    Ok(data_version)
}

fn read_data_version<R: Read + Seek>(reader: &mut OrderedReader<R>) -> io::Result<u32> {
    reader.seek(SeekFrom::Start(20))?;
    let data_version = u32::from(reader.read_u8()?) << 24 |
        u32::from(reader.read_u8()?) << 16 |
        u32::from(reader.read_u8()?) << 8 |
        u32::from(reader.read_u8()?);
    Ok(data_version)
}

fn read_header_size<B: Read + Seek>(reader: &mut OrderedReader<B>) -> io::Result<u16> {
    let header_size = reader.read_u16_from(SeekFrom::Start(0))?;
    let data_info_size = reader.read_u16_from(SeekFrom::Start(4))?;
    if data_info_size < 20 || header_size < (data_info_size + 4) {
        return Err(Error::new(ErrorKind::InvalidData, "header size error"));
    }
    Ok(header_size)
}

fn check_magic<B: Read + Seek>(bytes: &mut B) -> io::Result<()> {
    bytes.seek(SeekFrom::Start(2))?;
    let magic1 = bytes.read_u8()?;
    let magic2 = bytes.read_u8()?;
    if magic1 != MAGIC1 || magic2 != MAGIC2 {
        Err(Error::new(ErrorKind::InvalidData, MAGIC_NUMBER_CHECK_FAILED))
    } else {
        Ok(())
    }
}

fn read_endianness<B: Read + Seek>(bytes: &mut B) -> io::Result<u8> {
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

fn validate_format_version<R: Read + Seek>(reader: &mut OrderedReader<R>, data_format: DataFormat) -> io::Result<()> {
    let val = data_format as u32;
    reader.seek(SeekFrom::Start(12))?;
    let df = [reader.read_u8()?, reader.read_u8()?, reader.read_u8()?, reader.read_u8()?];
    if  df[0] != ((val >> 24) as u8) ||
        df[1] != ((val >> 16) as u8) ||
        df[2] != ((val >>  8) as u8) ||
        df[3] !=  (val        as u8) {
        return Err(Error::new(ErrorKind::InvalidInput, HEADER_CHECK_FAILED));
    }
    // format version starts at 16
    let format_version = [
        reader.read_u8()?,
        reader.read_u8()?,
        reader.read_u8()?,
        reader.read_u8()?
    ];
    if !data_format.is_acceptable_version(format_version) {
        // TODO print data format and format_version bytes with error message
        return Err(Error::new(ErrorKind::InvalidData, HEADER_CHECK_FAILED))
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use read_header;
    use DataFormat;
    use std::io::Cursor;

    #[test]
    fn read_header_doesnt_fail() {
        // header from a real resource bundle
        let mut c = Cursor::new(vec![0x0,  0x20, 0xda, 0x27,
                                     0x0,  0x14, 0x0,  0x0,
                                     0x1,  0x0,  0x02, 0x0,
                                     0x52, 0x65, 0x73, 0x42,
                                     0x03, 0x0,  0x0,  0x0,
                                     0x01, 0x04, 0x0,  0x0,
                                     0x0,  0x0,  0x0,  0x0,
                                     0x0,  0x0,  0x0,  0x0]);
        let r = read_header(&mut c, DataFormat::ResourceBundle).expect("Failed to read header");
        assert_eq!(r, 0x01040000u32);
    }
}
