use byteorder::{ReadBytesExt, WriteBytesExt};
use std::{
    error::Error,
    fmt,
    io::{Cursor, Read, Write, Seek, SeekFrom}
};

// Serialized versions of Unreal Engine's FString type. Mostly used as an intermediate between bytes and a full string
// TODO: Make it not completely die if an inappropriate endianess is passed - would require some trait Length to check that
// the read string length isn't larger than the entire file lol
pub trait FStringDeserializer {
    // Take a byte stream (cursor variant required, provides a nice wrapper for a cursor + Seek functions) and alloc a new string
    // Implementors are required to provide a UTF-8 string that doesn't include the null terminator
    // Ideally there'd be some way to automatically determine that for implementations of from_buffer() but idk i'll figure that out later
    fn from_buffer<R: Read + Seek, E: byteorder::ByteOrder>(reader: &mut R) -> Result<Option<String>, Box<dyn Error>>;
}
pub trait FStringSerializer {
    // Take a string and convert into a byte stream that can be written out onto a file
    // Don't consume it, in case that string needs to be written multiple times
    fn to_buffer<W: Write + Seek, E: byteorder::ByteOrder>(rstr: &str, writer: &mut W) -> Result<(), Box<dyn Error>>;
}

pub trait FStringSerializerText {
    // Take the text portion of a string and convert it into a byte stream that can be written out to
    // This exists for types that store their string and hash in separate blocks, as is the case with strings in IO store packages
    // Since this doesn't happen with PAK package strings and we're only interested in converting from PAK -> IO Store, there's 
    // currently no need for a Text and Hash variation for Deserializer
    fn to_buffer_text<W: Write + Seek, E: byteorder::ByteOrder>(rstr: &str, writer: &mut W) -> Result<(), Box<dyn Error>>;
}
pub trait FStringSerializerHash {
    // Take the hash portion of a string and convert it into a byte stream that can be written out to
    // I'm not writing out the same description again
    fn to_buffer_hash<W: Write + Seek, E: byteorder::ByteOrder>(rstr: &str, writer: &mut W) -> Result<(), Box<dyn Error>>;
}

pub trait FStringSerializerBlockAlign {
    // IO Store packages add padding bytes between strings and hashes to ensure all hashes are aligned
    fn get_block_alignment() -> u64;
    #[inline]
    fn to_buffer_alignment<W: Write + Seek, E: byteorder::ByteOrder>(writer: &mut W) {
        to_buffer_alignment_super::<Self, W, E>(writer);
    }
}
pub trait FStringSerializerExpectedLength {
    // Get the expected length in bytes of a serialized string given a string slice
    // In param has to be a string slice since there's no "Length" trait
    fn get_expected_length(value: &str) -> u64;
}

// Workaround since it's not possible to extend a trait method's default implementation
// (Java classes in university gave me OOP brainrot)
fn to_buffer_alignment_super<
        T: FStringSerializerBlockAlign + ?Sized, 
        W: Write + Seek, 
        E: byteorder::ByteOrder
    >(writer: &mut W) {
    let align = writer.stream_position().unwrap() % T::get_block_alignment();
    if align == 0 {
        return;
    }
    let diff = T::get_block_alignment() - align;
    writer.seek(SeekFrom::Current(diff as i64));
}

pub const NAME_HASH_ALGORITHM: u64 = 0xC1640000; // FNameHash::AlgorithmId

#[allow(dead_code)]
#[derive(Debug)]
// Used in a couple places, mostly in PAK package headers (see FolderName, SavedByEngineVersion in FPackageFilePackageSummary). 
// Serialized version of Unreal Engine's FString
pub struct FString32NoHash;
    // 0x0: len: u32
    // 0x4: data: [u8; len]
impl FString32NoHash {
    fn from_buffer_inner<R: Read + Seek, E: byteorder::ByteOrder>(reader: &mut R) -> Result<Option<String>, Box<dyn Error>> {
        let len = reader.read_u32::<E>()?; // length
        if len < 1 {
            return Ok(None); // we correctly parsed it, there's just nothing there lol
        }
        let mut buf = vec![0; (len - 1) as usize]; // get rid of that pesky \0
        reader.read_exact(&mut buf);
        reader.seek(SeekFrom::Current(1));
        Ok(Some(unsafe { String::from_utf8_unchecked(buf)}))
    }

    fn to_buffer_text_inner<W: Write, E: byteorder::ByteOrder>(rstr: &str, writer: &mut W) -> Result<(), Box<dyn Error>> {
        // add an extra byte for null terminator
        let len = (rstr.len() + if !rstr.ends_with("\0") { 1 } else { 0 }) as u32;
        writer.write_u32::<E>(len)?;
        writer.write_all(rstr.as_bytes());
        if !rstr.ends_with("\0") {
            writer.write_u8(b'\0')?;
        }
        Ok(())
    }
}

impl FStringDeserializer for FString32NoHash {
    fn from_buffer<R: Read + Seek, E: byteorder::ByteOrder>(reader: &mut R) -> Result<Option<String>, Box<dyn Error>> {
        FString32NoHash::from_buffer_inner::<R, E>(reader)
    }
}
impl FStringSerializer for FString32NoHash {
    fn to_buffer<W: Write, E: byteorder::ByteOrder>(rstr: &str, writer: &mut W) -> Result<(), Box<dyn Error>> {
        // check to see if our string has a null terminator, if we've made it in Rust, it won't, and if it's an import from another
        // Unreal stream, it shouldn't
        FString32NoHash::to_buffer_text_inner::<W, E>(rstr, writer)
    }
}
impl FStringSerializerText for FString32NoHash {
    fn to_buffer_text<W: Write, E: byteorder::ByteOrder>(rstr: &str, writer: &mut W) -> Result<(), Box<dyn Error>> {
        // check to see if our string has a null terminator, if we've made it in Rust, it won't, and if it's an import from another
        // Unreal stream, it shouldn't
        FString32NoHash::to_buffer_text_inner::<W, E>(rstr, writer)
    }
}
impl FStringSerializerExpectedLength for FString32NoHash {
    fn get_expected_length(value: &str) -> u64 {
        let str_len = (value.len() + if !value.ends_with("\0") { 1 } else { 0 }) as u64; // include null terminator
        str_len + 4 // 4 bytes at beginning to define string length
    }
}

#[allow(dead_code)]
#[derive(Debug)]
// Used for PAK package headers name map (FNameEntry)
pub struct FString32;
    // 0x0:         len: u32
    // 0x4:         data: [u8; len]
    // 0x4 + len:   hash: u32

impl FStringDeserializer for FString32 {
    fn from_buffer<R: Read + Seek, E: byteorder::ByteOrder>(reader: &mut R) -> Result<Option<String>, Box<dyn Error>> {
        let ret = FString32NoHash::from_buffer_inner::<R, E>(reader);
        reader.read_u32::<E>()?; // hash that we don't need GET THAT OUTTA HERE
        ret
    }
}

impl FStringSerializer for FString32 {
    fn to_buffer<W: Write, E: byteorder::ByteOrder>(rstr: &str, writer: &mut W) -> Result<(), Box<dyn Error>> {
        todo!("Wake me up when I need to figure out FString32's hash");
    }
}

#[allow(dead_code)]
#[derive(Debug)]
// Used for Name Map in IO Package headers (FStrings, followed by u64 hashes)
pub struct FString16;
    // 0x0:         len: u16
    // 0x2:         data: [u8; len]
    // 0x2 + len:   hash: u64 (cityhash64 of data.to_lowercase())

impl FString16 {
    pub fn check_hash(rstr: &str) -> u64 {
        Hasher::get_cityhash64(rstr)
    }

    fn to_buffer_text_inner<W: Write, E: byteorder::ByteOrder>(rstr: &str, writer: &mut W) -> Result<(), Box<dyn Error>> {
        writer.write_u16::<byteorder::BigEndian>(rstr.len().try_into()?)?; // length
        writer.write_all(rstr.as_bytes());
        Ok(())
    }
    fn to_buffer_hash_inner<W: Write, E: byteorder::ByteOrder>(rstr: &str, writer: &mut W) -> Result<(), Box<dyn Error>> {
        writer.write_u64::<E>(Hasher::get_cityhash64(&rstr));
        Ok(())
    }
}

impl FStringDeserializer for FString16 {
    fn from_buffer<R: Read, E: byteorder::ByteOrder>(reader: &mut R) -> Result<Option<String>, Box<dyn Error>> {
        // written in big endian for a u16. might be because of the path limit and 0x0 is reserved for some byte flag?
        // i'll figure that out later
        let len = reader.read_u16::<byteorder::BigEndian>()?;

        let mut buf = vec![0; len as usize];
        reader.read_exact(&mut buf);
        reader.read_u64::<E>()?;
        Ok(Some(unsafe { String::from_utf8_unchecked(buf)}))
    }
}
impl FStringSerializer for FString16 {
    fn to_buffer<W: Write, E: byteorder::ByteOrder>(rstr: &str, writer: &mut W) -> Result<(), Box<dyn Error>> {
        FString16::to_buffer_text_inner::<W, E>(rstr, writer);
        FString16::to_buffer_hash_inner::<W, E>(rstr, writer);
        Ok(())
    }
}
impl FStringSerializerText for FString16 {
    fn to_buffer_text<W: Write, E: byteorder::ByteOrder>(rstr: &str, writer: &mut W) -> Result<(), Box<dyn Error>> {
        FString16::to_buffer_text_inner::<W, E>(rstr, writer);
        Ok(())
    }
}
impl FStringSerializerHash for FString16 {
    fn to_buffer_hash<W: Write, E: byteorder::ByteOrder>(rstr: &str, writer: &mut W) -> Result<(), Box<dyn Error>> {
        FString16::to_buffer_hash_inner::<W, E>(rstr, writer);
        Ok(())
    }
}
impl FStringSerializerBlockAlign for FString16 {
    fn get_block_alignment() -> u64 {
        8
    }
    fn to_buffer_alignment<W: Write + Seek, E: byteorder::ByteOrder>(writer: &mut W) {
        to_buffer_alignment_super::<Self, W, E>(writer);
        writer.write_u64::<E>(NAME_HASH_ALGORITHM);
    }
}

// Rename to Hasher8 later
pub struct Hasher;
impl Hasher {
    pub fn get_cityhash64(bytes: &str) -> u64 {
        let to_hash = String::from(bytes).to_lowercase();
        cityhasher::hash(to_hash.as_bytes())
    }
}

// TODO: Switch IoStoreObjectIndex to use Hasher16 as a base implementation
pub struct Hasher16;
impl Hasher16 {
    pub fn get_cityhash64(bytes: &str) -> u64 {
        let to_hash = String::from(bytes).to_lowercase();
        // hash chars are sized according to if the platform supports wide characters, which is usually the case
        let to_hash: Vec<u16> = to_hash.encode_utf16().collect();
        // safety: Vec is contiguous, so a Vec<u8> of length `2 * n` will take the same memory as a Vec<u16> of len `n`
        let to_hash = unsafe { std::slice::from_raw_parts(to_hash.as_ptr() as *const u8, to_hash.len() * 2) };
        // verified: the strings are identical (no null terminator) when using FString16
        cityhasher::hash(to_hash) // cityhash it
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FMappedName(u32, u32); // NameIndex, ExtraIndex
// first field is index in name map

impl FMappedName {
    pub fn get_name_index(&self) -> u32 {
        self.0
    }
    pub fn get_extra_index(&self) -> u32 {
        self.1
    }
}

impl From<u64> for FMappedName {
    fn from(value: u64) -> Self {
        let name_index: u32 = (value & u32::MAX as u64) as u32;
        let extra_index: u32 = ((value >> 0x20) & u32::MAX as u64) as u32;
        Self(name_index, extra_index)
    }
}

impl From<FMappedName> for u64 {
    fn from(value: FMappedName) -> Self {
        value.0 as u64 | (value.1 as u64) << 0x20
    }
}