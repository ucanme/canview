impl ObjectHeader {
    /// Reads an `ObjectHeader` (V1 or V2) from a byte stream.
    pub fn read(cursor: &mut Cursor<&[u8]>) -> BlfParseResult<Self> {
        let signature = cursor.read_u32::<LittleEndian>()?;
        if signature != 0x4A424F4C {
            return Err(BlfParseError::InvalidContainerMagic);
        }
        let header_size = cursor.read_u16::<LittleEndian>()?;
        let header_version = cursor.read_u16::<LittleEndian>()?;
        let object_size = cursor.read_u32::<LittleEndian>()?;
        let object_type = ObjectType::from(cursor.read_u32::<LittleEndian>()?);

        let object_flags;
        let object_time_stamp;
        let mut original_time_stamp = None;
        let mut time_stamp_status = None;

        if header_version == 1 {
            object_flags = cursor.read_u32::<LittleEndian>()?;
            let _client_index = cursor.read_u16::<LittleEndian>()?;
            let _object_version = cursor.read_u16::<LittleEndian>()?;
            object_time_stamp = cursor.read_u64::<LittleEndian>()?;
        } else if header_version == 2 {
            object_flags = cursor.read_u32::<LittleEndian>()?;
            time_stamp_status = Some(cursor.read_u8()?);
            let _reserved = cursor.read_u8()?;
            let _object_version = cursor.read_u16::<LittleEndian>()?;
            object_time_stamp = cursor.read_u64::<LittleEndian>()?;
            original_time_stamp = Some(cursor.read_u64::<LittleEndian>()?);
        } else {
            return Err(BlfParseError::UnknownHeaderVersion(header_version));
        }

        Ok(ObjectHeader {
            signature,
            header_size,
            header_version,
            object_size,
            object_type,
            object_flags,
            object_time_stamp,
            original_time_stamp,
            time_stamp_status,
        })
    }

    /// Calculate the header size in bytes for this ObjectHeader
    pub fn calculate_header_size(&self) -> u16 {
        if self.header_version == 1 {
            self.header_size
        } else if self.header_version == 2 {
            self.header_size
        } else {
            // Fallback - should not happen in valid files
            24
        }
    }

    /// Writes an `ObjectHeader` to a byte stream.
    pub fn write<W: Write>(&self, writer: &mut W) -> BlfParseResult<()> {
        writer.write_u32::<LittleEndian>(self.signature)?;
        writer.write_u16::<LittleEndian>(self.header_size)?;
        writer.write_u16::<LittleEndian>(self.header_version)?;
        writer.write_u32::<LittleEndian>(self.object_size)?;
        writer.write_u32::<LittleEndian>(self.object_type as u32)?;
        writer.write_u32::<LittleEndian>(self.object_flags)?;

        if self.header_version == 1 {
            writer.write_u16::<LittleEndian>(0)?; // _client_index
            writer.write_u16::<LittleEndian>(0)?; // _object_version
            writer.write_u64::<LittleEndian>(self.object_time_stamp)?;
        } else if self.header_version == 2 {
            writer.write_u8(self.time_stamp_status.unwrap_or(0))?;
            writer.write_u8(0)?; // _reserved
            writer.write_u16::<LittleEndian>(0)?; // _object_version
            writer.write_u64::<LittleEndian>(self.object_time_stamp)?;
            writer.write_u64::<LittleEndian>(self.original_time_stamp.unwrap_or(0))?;
        } else {
            return Err(BlfParseError::UnknownHeaderVersion(self.header_version));
        }
        Ok(())
    }
}