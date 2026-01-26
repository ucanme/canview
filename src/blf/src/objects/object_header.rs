//! Defines the ObjectHeader struct for BLF log objects.
//!
//! This module mirrors the C++ class hierarchy:
//! - ObjectHeaderBase (base class with common fields)
//! - ObjectHeader (V1, inherits from ObjectHeaderBase)
//! - ObjectHeader2 (V2, inherits from ObjectHeaderBase)
//!
//! In Rust, we use composition instead of inheritance.

use crate::ObjectType;
use crate::{BlfParseError, BlfParseResult};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Write};

/// Object signature constant ("LOBJ" = 0x4A424F4C)
pub const OBJECT_SIGNATURE: u32 = 0x4A424F4C;

/// Base object header (corresponds to C++ ObjectHeaderBase).
///
/// This contains the common fields present in all BLF object headers.
/// Size: 16 bytes
///
/// Memory layout:
/// ```text
/// +0x00  signature (u32)        - 0x4A424F4C
/// +0x04  headerSize (u16)       - 16 (base), 32 (V1), 40 (V2)
/// +0x06  headerVersion (u16)    - 1 or 2
/// +0x08  objectSize (u32)       - total object size
/// +0x0C  objectType (u32)       - object type enum
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectHeaderBase {
    /// Object signature, should be "LOBJ" (0x4A424F4C).
    pub signature: u32,
    /// Size of this header in bytes (16 for base, 32 for V1, 48 for V2).
    pub header_size: u16,
    /// Version of the object header (1 or 2).
    pub header_version: u16,
    /// Total size of the object in bytes (header + data).
    pub object_size: u32,
    /// Type of the object.
    pub object_type: ObjectType,
}

impl ObjectHeaderBase {
    /// Creates a new ObjectHeaderBase (corresponds to C++ ObjectHeaderBase constructor).
    ///
    /// # Arguments
    /// * `header_version` - Version number (1 for ObjectHeader, 2 for ObjectHeader2)
    /// * `object_type` - Type of the object
    pub fn new(header_version: u16, object_type: ObjectType) -> Self {
        ObjectHeaderBase {
            signature: OBJECT_SIGNATURE,
            header_size: 16, // Will be calculated by derived classes
            header_version,
            object_size: 0, // Will be calculated by derived classes
            object_type,
        }
    }

    /// Reads the base header fields from a byte stream.
    pub fn read(cursor: &mut Cursor<&[u8]>) -> BlfParseResult<Self> {
        let signature = cursor.read_u32::<LittleEndian>()?;
        if signature != OBJECT_SIGNATURE {
            return Err(BlfParseError::InvalidContainerMagic);
        }
        let header_size = cursor.read_u16::<LittleEndian>()?;
        let header_version = cursor.read_u16::<LittleEndian>()?;
        let object_size = cursor.read_u32::<LittleEndian>()?;
        let object_type = ObjectType::from(cursor.read_u32::<LittleEndian>()?);

        Ok(ObjectHeaderBase {
            signature,
            header_size,
            header_version,
            object_size,
            object_type,
        })
    }

    /// Writes the base header fields to a byte stream.
    pub fn write<W: Write>(&self, writer: &mut W) -> BlfParseResult<()> {
        writer.write_u32::<LittleEndian>(self.signature)?;
        writer.write_u16::<LittleEndian>(self.header_size)?;
        writer.write_u16::<LittleEndian>(self.header_version)?;
        writer.write_u32::<LittleEndian>(self.object_size)?;
        writer.write_u32::<LittleEndian>(self.object_type as u32)?;
        Ok(())
    }

    /// Calculates the header size in bytes (corresponds to C++ ObjectHeaderBase::calculateHeaderSize).
    ///
    /// For the base class, this is always 16 bytes:
    /// - signature (4) + headerSize (2) + headerVersion (2) + objectSize (4) + objectType (4)
    pub fn calculate_header_size(&self) -> u16 {
        std::mem::size_of::<u32>() as u16 + // signature
        std::mem::size_of::<u16>() as u16 + // headerSize
        std::mem::size_of::<u16>() as u16 + // headerVersion
        std::mem::size_of::<u32>() as u16 + // objectSize
        std::mem::size_of::<u32>() as u16 // objectType
    }

    /// Calculates the object size in bytes (corresponds to C++ ObjectHeaderBase::calculateObjectSize).
    ///
    /// For the base class, this is just the header size.
    /// Derived classes should override this to include their data.
    pub fn calculate_object_size(&self) -> u32 {
        self.calculate_header_size() as u32
    }
}

/// Object flags (corresponds to C++ ObjectHeader::ObjectFlags).
#[derive(Debug, Clone, Copy)]
pub enum ObjectFlags {
    /// 10 microsecond timestamp
    TimeTenMics = 0x00000001,
    /// 1 nanosecond timestamp
    TimeOneNans = 0x00000002,
}

/// Timestamp status flags (corresponds to C++ ObjectHeader2::TimeStampStatus).
#[derive(Debug, Clone, Copy)]
pub enum TimeStampStatus {
    /// Valid original timestamp
    Orig = 0x01,
    /// Software (1) vs Hardware (0) generated timestamp
    SwHw = 0x02,
    /// Protocol-specific meaning
    User = 0x10,
}

/// Complete object header for BLF log objects (V1 and V2).
///
/// This corresponds to:
/// - C++ ObjectHeader (V1, 32 bytes) which inherits from ObjectHeaderBase
/// - C++ ObjectHeader2 (V2, 40 bytes) which inherits from ObjectHeaderBase
///
/// Rust uses composition instead of inheritance, so we include
/// ObjectHeaderBase as a field and add version-specific fields.
///
/// Memory layout for V1 (32 bytes):
/// ```text
/// +0x00  signature (u32)        [ObjectHeaderBase]
/// +0x04  headerSize (u16)       [ObjectHeaderBase]
/// +0x06  headerVersion (u16)    [ObjectHeaderBase]
/// +0x08  objectSize (u32)       [ObjectHeaderBase]
/// +0x0C  objectType (u32)       [ObjectHeaderBase]
/// +0x10  objectFlags (u32)
/// +0x14  clientIndex (u16)
/// +0x16  objectVersion (u16)
/// +0x18  objectTimeStamp (u64)
/// ```
///
/// Memory layout for V2 (40 bytes):
/// ```text
/// +0x00  signature (u32)        [ObjectHeaderBase]
/// +0x04  headerSize (u16)       [ObjectHeaderBase]
/// +0x06  headerVersion (u16)    [ObjectHeaderBase]
/// +0x08  objectSize (u32)       [ObjectHeaderBase]
/// +0x0C  objectType (u32)       [ObjectHeaderBase]
/// +0x10  objectFlags (u32)
/// +0x14  timeStampStatus (u8)
/// +0x15  reserved (u8)
/// +0x16  objectVersion (u16)
/// +0x18  objectTimeStamp (u64)
/// +0x20  originalTimeStamp (u64)
/// ```
#[derive(Debug, Clone)]
pub struct ObjectHeader {
    /// Base header fields (common to all versions)
    pub base: ObjectHeaderBase,

    // V1 & V2 common fields (offset +16 from base)
    /// Object-specific flags (e.g., timestamp precision).
    pub object_flags: u32,

    // V1-specific fields (offset +20 from base)
    /// Client index of the sending node (V1 only).
    pub client_index: u16,
    /// Object-specific version number.
    pub object_version: u16,

    // V1 & V2 common fields
    /// Timestamp of the object.
    pub object_time_stamp: u64,

    // V2-specific fields
    /// Original timestamp (only in V2 headers).
    pub original_time_stamp: Option<u64>,
    /// Timestamp status (only in V2 headers).
    pub time_stamp_status: Option<u8>,
    /// Reserved byte (only in V2 headers).
    pub reserved: u8,
}

impl ObjectHeader {
    /// Creates a new ObjectHeader for V1 (corresponds to C++ ObjectHeader constructor).
    ///
    /// # Arguments
    /// * `object_type` - Type of the object
    /// * `object_version` - Object-specific version number (default 0)
    ///
    /// # Example
    /// ```
    /// let header = ObjectHeader::new_v1(ObjectType::CanMessage, 0);
    /// ```
    ///
    /// # C++ Correspondence
    /// ```cpp
    /// // C++: ObjectHeader::ObjectHeader(const ObjectType objectType, const uint16_t objectVersion = 0)
    /// ObjectHeader header(ObjectType::CAN_MESSAGE, 0);
    /// ```
    ///
    /// # Header Size
    /// V1 headers are 32 bytes total.
    pub fn new_v1(object_type: ObjectType, object_version: u16) -> Self {
        let base = ObjectHeaderBase::new(1, object_type);

        ObjectHeader {
            base,
            object_flags: ObjectFlags::TimeOneNans as u32,
            client_index: 0,
            object_version,
            object_time_stamp: 0,
            original_time_stamp: None,
            time_stamp_status: None,
            reserved: 0,
        }
    }

    /// Creates a new ObjectHeader for V2 (corresponds to C++ ObjectHeader2 constructor).
    ///
    /// # Arguments
    /// * `object_type` - Type of the object
    ///
    /// # Example
    /// ```
    /// let header = ObjectHeader::new_v2(ObjectType::CanMessage2);
    /// ```
    ///
    /// # C++ Correspondence
    /// ```cpp
    /// // C++: ObjectHeader2::ObjectHeader2(const ObjectType objectType)
    /// ObjectHeader2 header2(ObjectType::CAN_MESSAGE2);
    /// ```
    ///
    /// # Header Size
    /// V2 headers are 40 bytes total.
    pub fn new_v2(object_type: ObjectType) -> Self {
        let base = ObjectHeaderBase::new(2, object_type);

        ObjectHeader {
            base,
            object_flags: ObjectFlags::TimeOneNans as u32,
            client_index: 0,
            object_version: 0,
            object_time_stamp: 0,
            original_time_stamp: Some(0),
            time_stamp_status: Some(0),
            reserved: 0,
        }
    }

    /// Creates a new ObjectHeader with specified version (convenience method).
    pub fn new(header_version: u16, object_type: ObjectType) -> Self {
        if header_version == 1 {
            Self::new_v1(object_type, 0)
        } else if header_version == 2 {
            Self::new_v2(object_type)
        } else {
            panic!("Unsupported header version: {}", header_version);
        }
    }

    /// Reads an `ObjectHeader` (V1 or V2) from a byte stream.
    pub fn read(cursor: &mut Cursor<&[u8]>) -> BlfParseResult<Self> {
        // Read base header first
        let base = ObjectHeaderBase::read(cursor)?;

        let mut object_flags = 0;
        let mut client_index = 0;
        let mut object_version = 0;
        let mut object_time_stamp = 0;
        let mut original_time_stamp = None;
        let mut time_stamp_status = None;
        let mut reserved = 0;

        if base.header_version == 1 {
            // V1 header 解析
            // 注意：有些 BLF 文件的 header_size 字段不准确
            // 即使 header_size=16，实际数据中也可能包含完整的 32 字节 header
            // 我们需要根据实际可用数据来判断

            if base.header_size >= 32 {
                // 标准的完整 V1 header (32 字节)
                object_flags = cursor.read_u32::<LittleEndian>()?;
                client_index = cursor.read_u16::<LittleEndian>()?;
                object_version = cursor.read_u16::<LittleEndian>()?;
                object_time_stamp = cursor.read_u64::<LittleEndian>()?;
            } else if base.header_size == 16 {
                // header_size 声称是 16 字节，但我们尝试读取完整数据
                // 因为很多 BLF 文件的 header_size 字段不准确

                // 检查是否还有足够的数据（至少 16 字节）
                let remaining = cursor.get_ref().len() - cursor.position() as usize;

                if remaining >= 16 {
                    // 有足够数据，尝试读取完整 header
                    object_flags = cursor.read_u32::<LittleEndian>()?;
                    client_index = cursor.read_u16::<LittleEndian>()?;
                    object_version = cursor.read_u16::<LittleEndian>()?;
                    object_time_stamp = cursor.read_u64::<LittleEndian>()?;
                } else {
                    // 数据不足，这才是真正的紧凑型 header
                    // 保持默认值（全零）
                }
            } else {
                return Err(BlfParseError::UnknownHeaderVersion(base.header_version));
            }
        } else if base.header_version == 2 {
            // V2 header: flags + timeStampStatus + reserved + objectVersion + timestamp + originalTimestamp
            object_flags = cursor.read_u32::<LittleEndian>()?;
            time_stamp_status = Some(cursor.read_u8()?);
            reserved = cursor.read_u8()?;
            object_version = cursor.read_u16::<LittleEndian>()?;
            object_time_stamp = cursor.read_u64::<LittleEndian>()?;
            original_time_stamp = Some(cursor.read_u64::<LittleEndian>()?);
            client_index = 0; // Not used in V2
        } else {
            return Err(BlfParseError::UnknownHeaderVersion(base.header_version));
        }

        Ok(ObjectHeader {
            base,
            object_flags,
            client_index,
            object_version,
            object_time_stamp,
            original_time_stamp,
            time_stamp_status,
            reserved,
        })
    }

    /// Writes an `ObjectHeader` to a byte stream (corresponds to C++ ObjectHeader::write).
    ///
    /// Note: This method does NOT automatically calculate sizes like the C++ version.
    /// Call `prepare_for_write()` before this method if you need to update header_size and object_size.
    ///
    /// # Example
    /// ```
    /// header.prepare_for_write();  // Calculate sizes first
    /// header.write(&mut writer)?;
    /// ```
    pub fn write<W: Write>(&self, writer: &mut W) -> BlfParseResult<()> {
        // Write base header first
        self.base.write(writer)?;

        if self.base.header_version == 1 {
            // V1 has two variants:
            // - 16 bytes: compact header (no additional fields)
            // - 32 bytes: full header with flags, clientIndex, objectVersion, timestamp
            if self.base.header_size >= 32 {
                // Full V1 header
                writer.write_u32::<LittleEndian>(self.object_flags)?;
                writer.write_u16::<LittleEndian>(self.client_index)?;
                writer.write_u16::<LittleEndian>(self.object_version)?;
                writer.write_u64::<LittleEndian>(self.object_time_stamp)?;
            }
            // If header_size == 16, don't write anything else (compact header)
        } else if self.base.header_version == 2 {
            // V2 header: flags + timeStampStatus + reserved + objectVersion + timestamp + originalTimestamp
            writer.write_u32::<LittleEndian>(self.object_flags)?;
            writer.write_u8(self.time_stamp_status.unwrap_or(0))?;
            writer.write_u8(self.reserved)?;
            writer.write_u16::<LittleEndian>(self.object_version)?;
            writer.write_u64::<LittleEndian>(self.object_time_stamp)?;
            writer.write_u64::<LittleEndian>(self.original_time_stamp.unwrap_or(0))?;
        } else {
            return Err(BlfParseError::UnknownHeaderVersion(
                self.base.header_version,
            ));
        }
        Ok(())
    }

    /// Calculates the header size in bytes (corresponds to C++ ObjectHeader::calculateHeaderSize).
    ///
    /// For V1: 32 bytes (base 16 + objectFlags 4 + clientIndex 2 + objectVersion 2 + objectTimeStamp 8)
    /// For V2: 40 bytes (base 16 + objectFlags 4 + timeStampStatus 1 + reserved 1 + objectVersion 2 + objectTimeStamp 8 + originalTimeStamp 8)
    pub fn calculate_header_size(&self) -> u16 {
        if self.base.header_version == 1 {
            // V1: ObjectHeader
            self.base.calculate_header_size() + // 16
            std::mem::size_of::<u32>() as u16 + // objectFlags
            std::mem::size_of::<u16>() as u16 + // clientIndex
            std::mem::size_of::<u16>() as u16 + // objectVersion
            std::mem::size_of::<u64>() as u16 // objectTimeStamp
        } else if self.base.header_version == 2 {
            // V2: ObjectHeader2 = 40 bytes
            self.base.calculate_header_size() + // 16
            std::mem::size_of::<u32>() as u16 + // objectFlags (4)
            std::mem::size_of::<u8>() as u16 +  // timeStampStatus (1)
            std::mem::size_of::<u8>() as u16 +  // reserved (1)
            std::mem::size_of::<u16>() as u16 + // objectVersion (2)
            std::mem::size_of::<u64>() as u16 + // objectTimeStamp (8)
            std::mem::size_of::<u64>() as u16 // originalTimeStamp (8)
        // Total: 16 + 4 + 1 + 1 + 2 + 8 + 8 = 40 bytes
        } else {
            self.base.header_size
        }
    }

    /// Calculates the object size in bytes (corresponds to C++ ObjectHeader::calculateObjectSize).
    ///
    /// This should be overridden by actual object types to include their data.
    /// For the header itself, this just returns the header size.
    pub fn calculate_object_size(&self) -> u32 {
        self.calculate_header_size() as u32
    }

    /// Prepares the header for writing by calculating sizes (corresponds to C++ ObjectHeaderBase::write preprocessing).
    ///
    /// This method should be called before `write()` to ensure header_size and object_size are set correctly.
    ///
    /// # Example
    /// ```
    /// header.prepare_for_write();
    /// header.write(&mut writer)?;
    /// ```
    pub fn prepare_for_write(&mut self) {
        self.base.header_size = self.calculate_header_size();
        self.base.object_size = self.calculate_object_size();
    }

    /// Returns the header version (1 or 2).
    pub fn version(&self) -> u16 {
        self.base.header_version
    }

    /// Returns the object type.
    pub fn object_type(&self) -> ObjectType {
        self.base.object_type
    }

    /// Returns the signature.
    pub fn signature(&self) -> u32 {
        self.base.signature
    }
}

// Provide convenience methods for accessing common fields
impl std::ops::Deref for ObjectHeader {
    type Target = ObjectHeaderBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ObjectHeader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

// Manual implementation of PartialEq for ObjectHeader
impl PartialEq for ObjectHeader {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base
            && self.object_flags == other.object_flags
            && self.client_index == other.client_index
            && self.object_version == other.object_version
            && self.object_time_stamp == other.object_time_stamp
            && self.original_time_stamp == other.original_time_stamp
            && self.time_stamp_status == other.time_stamp_status
            && self.reserved == other.reserved
    }
}

impl Eq for ObjectHeader {}

impl Default for ObjectHeader {
    fn default() -> Self {
        ObjectHeader {
            base: ObjectHeaderBase {
                signature: OBJECT_SIGNATURE,
                header_size: 32,
                header_version: 1,
                object_size: 32,
                object_type: ObjectType::Unknown,
            },
            object_flags: 0,
            client_index: 0,
            object_version: 0,
            object_time_stamp: 0,
            original_time_stamp: None,
            time_stamp_status: None,
            reserved: 0,
        }
    }
}

impl ObjectHeader {
    /// Validates the header consistency with detailed logging.
    pub fn validate(&self) -> BlfParseResult<()> {
        if self.signature != OBJECT_SIGNATURE {
            println!("ERROR: Invalid object signature: 0x{:08X}", self.signature);
            return Err(BlfParseError::InvalidContainerMagic);
        }

        if self.object_size < self.header_size as u32 {
            println!(
                "ERROR: Object size ({}) is smaller than header size ({})",
                self.object_size, self.header_size
            );
            return Err(BlfParseError::InvalidContainerMagic);
        }

        if self.header_version != 1 && self.header_version != 2 {
            println!(
                "ERROR: Unsupported header version: {} (supported: 1, 2)",
                self.header_version
            );
            return Err(BlfParseError::UnknownHeaderVersion(self.header_version));
        }

        // Additional consistency checks
        let expected_header_size = if self.header_version == 1 { 32 } else { 48 };
        if self.header_size != expected_header_size {
            println!(
                "WARN: Header size {} differs from expected {} for version {}",
                self.header_size, expected_header_size, self.header_version
            );
        }

        println!(
            "DEBUG: Header validation passed for object type {:?}",
            self.object_type
        );
        Ok(())
    }

    /// Returns a detailed debug string for troubleshooting.
    pub fn debug_info(&self) -> String {
        format!(
            "ObjectHeader {{ sig: 0x{:08X}, hdr_size: {}, hdr_ver: {}, obj_size: {}, obj_type: {:?} ({}), flags: 0x{:08X}, timestamp: {} }}",
            self.signature,
            self.header_size,
            self.header_version,
            self.object_size,
            self.object_type,
            self.object_type as u32,
            self.object_flags,
            self.object_time_stamp
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_object_header_base_calculate_size() {
        let base = ObjectHeaderBase::new(1, ObjectType::CanMessage);
        assert_eq!(base.calculate_header_size(), 16);
        assert_eq!(base.calculate_object_size(), 16);
    }

    #[test]
    fn test_object_header_v1_calculate_size() {
        let header = ObjectHeader::new_v1(ObjectType::CanMessage, 0);
        // V1: 16 (base) + 4 (flags) + 2 (client) + 2 (version) + 8 (timestamp) = 32
        assert_eq!(header.calculate_header_size(), 32);
        assert_eq!(header.calculate_object_size(), 32);
    }

    #[test]
    fn test_object_header_v2_calculate_size() {
        let header = ObjectHeader::new_v2(ObjectType::CanMessage2);
        // V2: 16 (base) + 4 (flags) + 1 (status) + 1 (reserved) + 2 (version) + 8 (timestamp) + 8 (original) = 40 bytes
        // This matches C++ ObjectHeader2::calculateHeaderSize()
        assert_eq!(header.calculate_header_size(), 40);
        assert_eq!(header.calculate_object_size(), 40);
    }

    #[test]
    fn test_object_header_v1_write_read_roundtrip() {
        let mut header = ObjectHeader::new_v1(ObjectType::CanMessage, 5);
        header.object_flags = ObjectFlags::TimeOneNans as u32;
        header.client_index = 42;
        header.object_version = 5;
        header.object_time_stamp = 123456789;

        // Prepare and write
        header.prepare_for_write();

        let mut buffer = Vec::new();
        header.write(&mut buffer).unwrap();

        // Read back
        let mut cursor = Cursor::new(buffer.as_slice());
        let header2 = ObjectHeader::read(&mut cursor).unwrap();

        // Verify all fields
        assert_eq!(header2.signature, OBJECT_SIGNATURE);
        assert_eq!(header2.header_version, 1);
        assert_eq!(header2.object_type, ObjectType::CanMessage);
        assert_eq!(header2.object_flags, ObjectFlags::TimeOneNans as u32);
        assert_eq!(header2.client_index, 42);
        assert_eq!(header2.object_version, 5);
        assert_eq!(header2.object_time_stamp, 123456789);
    }

    #[test]
    fn test_object_header_v2_write_read_roundtrip() {
        let mut header = ObjectHeader::new_v2(ObjectType::CanMessage2);
        header.object_flags = ObjectFlags::TimeOneNans as u32;
        header.object_version = 1;
        header.object_time_stamp = 987654321;
        header.original_time_stamp = Some(111111111);
        header.time_stamp_status = Some(0x01);
        header.reserved = 0;

        // Prepare and write
        header.prepare_for_write();

        let mut buffer = Vec::new();
        header.write(&mut buffer).unwrap();

        // Read back
        let mut cursor = Cursor::new(buffer.as_slice());
        let header2 = ObjectHeader::read(&mut cursor).unwrap();

        // Verify all fields
        assert_eq!(header2.signature, OBJECT_SIGNATURE);
        assert_eq!(header2.header_version, 2);
        assert_eq!(header2.object_type, ObjectType::CanMessage2);
        assert_eq!(header2.object_flags, ObjectFlags::TimeOneNans as u32);
        assert_eq!(header2.object_version, 1);
        assert_eq!(header2.object_time_stamp, 987654321);
        assert_eq!(header2.original_time_stamp, Some(111111111));
        assert_eq!(header2.time_stamp_status, Some(0x01));
    }

    #[test]
    fn test_object_header_v1_compact_header() {
        // Create a compact V1 header (16 bytes)
        let base = ObjectHeaderBase {
            signature: OBJECT_SIGNATURE,
            header_size: 16,
            header_version: 1,
            object_size: 16,
            object_type: ObjectType::CanMessage,
        };

        let mut buffer = Vec::new();
        base.write(&mut buffer).unwrap();

        // Read back
        let mut cursor = Cursor::new(buffer.as_slice());
        let header = ObjectHeader::read(&mut cursor).unwrap();

        // Verify compact header has zeros for extended fields
        assert_eq!(header.object_flags, 0);
        assert_eq!(header.client_index, 0);
        assert_eq!(header.object_version, 0);
        assert_eq!(header.object_time_stamp, 0);
    }

    #[test]
    fn test_object_header_constants() {
        assert_eq!(OBJECT_SIGNATURE, 0x4A424F4C);
        assert_eq!(ObjectFlags::TimeTenMics as u32, 0x00000001);
        assert_eq!(ObjectFlags::TimeOneNans as u32, 0x00000002);
        assert_eq!(TimeStampStatus::Orig as u8, 0x01);
        assert_eq!(TimeStampStatus::SwHw as u8, 0x02);
        assert_eq!(TimeStampStatus::User as u8, 0x10);
    }

    #[test]
    fn test_object_header_deref() {
        let header = ObjectHeader::new_v1(ObjectType::CanMessage, 0);

        // Test deref to base fields
        assert_eq!(header.signature, OBJECT_SIGNATURE);
        assert_eq!(header.header_version, 1);
        assert_eq!(header.object_type, ObjectType::CanMessage);

        // Test convenience methods
        assert_eq!(header.version(), 1);
        assert_eq!(header.object_type(), ObjectType::CanMessage);
        assert_eq!(header.signature(), OBJECT_SIGNATURE);
    }

    #[test]
    fn test_object_header_prepare_for_write() {
        let mut header = ObjectHeader::new_v1(ObjectType::CanMessage, 0);
        header.object_time_stamp = 1000000;

        // Before prepare, sizes might be incorrect
        header.prepare_for_write();

        // After prepare, sizes should be calculated
        assert_eq!(header.header_size, 32); // V1 header is 32 bytes
        assert_eq!(header.object_size, 32);
    }

    #[test]
    fn test_object_header_validate() {
        let mut header = ObjectHeader::new_v1(ObjectType::CanMessage, 0);
        header.prepare_for_write();

        // Valid header should pass
        assert!(header.validate().is_ok());

        // Invalid signature should fail
        header.base.signature = 0xDEADBEEF;
        assert!(header.validate().is_err());
    }
}
