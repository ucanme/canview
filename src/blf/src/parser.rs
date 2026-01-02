//! BLF parser implementation.

use std::io::Cursor;
use crate::blf_core::{BlfParseResult, ObjectHeader, ObjectType, BlfParseError};
use crate::objects::*;
use crate::objects::most::{MostSpy, MostCtrl, MostPkt2, MostLightLock, MostStatistic, MostHwMode, MostReg, MostGenReg, MostNetState, MostDataLost, MostTrigger};

/// An enumeration of all possible log objects that can be parsed from a BLF file.
#[derive(Debug, Clone, PartialEq)] 
pub enum LogObject {
    CanMessage(CanMessage),
    CanMessage2(CanMessage2),
    CanErrorFrame(CanErrorFrame),
    CanFdMessage(CanFdMessage),
    CanFdMessage64(CanFdMessage64),
    CanOverloadFrame(CanOverloadFrame),
    CanDriverStatistic(CanDriverStatistic),
    CanDriverError(CanDriverError),
    LinMessage(LinMessage),
    LinMessage2(LinMessage2),
    LinCrcError(LinCrcError),
    LinDlcInfo(LinDlcInfo),
    LinReceiveError(LinReceiveError),
    LinSendError(LinSendError),
    LinSlaveTimeout(LinSlaveTimeout),
    LinSchedulerModeChange(LinSchedulerModeChange),
    LinSyncError(LinSyncError),
    LinBaudrateEvent(LinBaudrateEvent),
    LinSleepModeEvent(LinSleepModeEvent),
    LinWakeupEvent(LinWakeupEvent),
    FlexRayData(FlexRayData),
    FlexRaySync(FlexRaySync),
    FlexRayV6Message(FlexRayV6Message),
    FlexRayV6StartCycleEvent(FlexRayV6StartCycleEvent),
    FlexRayStatusEvent(FlexRayStatusEvent),
    FlexRayVFrError(FlexRayVFrError),
    FlexRayVFrStatus(FlexRayVFrStatus),
    FlexRayVFrStartCycle(FlexRayVFrStartCycle),
    FlexRayVFrReceiveMsg(FlexRayVFrReceiveMsg),
    FlexRayVFrReceiveMsgEx(FlexRayVFrReceiveMsgEx),
    EthernetFrame(EthernetFrame),
    // Temporarily comment out missing types
    // SystemVariable(SystemVariable),
    // EnvInteger(EnvInteger),
    // EnvDouble(EnvDouble),
    // EnvString(EnvString),
    AppTrigger(AppTrigger),
    EventComment(EventComment),
    GlobalMarker(GlobalMarker),
    MostSpy(MostSpy),
    MostCtrl(MostCtrl),
    MostPkt2(MostPkt2),
    MostLightLock(MostLightLock),
    MostStatistic(MostStatistic),
    MostHwMode(MostHwMode),
    MostReg(MostReg),
    MostGenReg(MostGenReg),
    MostNetState(MostNetState),
    MostDataLost(MostDataLost),
    MostTrigger(MostTrigger),
}

/// The main parser for BLF files.
#[derive(Default)]
pub struct BlfParser {}

impl BlfParser {
    /// Creates a new `BlfParser`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses a byte slice containing BLF data into a list of `LogObject`s.
    pub fn parse(&self, data: &[u8]) -> BlfParseResult<Vec<LogObject>> {
        let mut cursor = Cursor::new(data);
        let mut all_objects = Vec::new();
        let data_len = cursor.get_ref().len();

        // The top-level of a BLF file (after the initial FileStatistics header, which is handled elsewhere)
        // consists of a series of LogContainer objects. This loop iterates through them.
        while (cursor.position() as usize) < data_len {
            let start_pos = cursor.position();
            let header = ObjectHeader::read(&mut cursor)?;

            if header.object_type != ObjectType::LogContainer {
                // According to the C++ source (File.cpp -> compressedFile2UncompressedFile),
                // only LogContainers are expected at the top level of the object stream.
                // We will skip any other object types at this level to be robust.
            } else {
                let container = LogContainer::read(&mut cursor, header.clone())?;
                let mut container_cursor = Cursor::new(&container.uncompressed_data[..]);
                all_objects.extend(self.parse_inner_objects(&mut container_cursor)?);
            }
            self.advance_cursor_to_next_object(&mut cursor, start_pos, header.object_size);
        }
        Ok(all_objects)
    }

    fn parse_can_object(&self, cursor: &mut Cursor<&[u8]>, header: &ObjectHeader, object_data_size: usize) -> BlfParseResult<Option<LogObject>> {
        match header.object_type {
            ObjectType::CanMessage => Ok(Some(LogObject::CanMessage(CanMessage::read(cursor, header)?))),
            ObjectType::CanMessage2 => Ok(Some(LogObject::CanMessage2(CanMessage2::read(cursor, header, object_data_size)?))),
            ObjectType::CanError => Ok(Some(LogObject::CanErrorFrame(CanErrorFrame::read(cursor, header)?))),
            ObjectType::CanFdMessage => Ok(Some(LogObject::CanFdMessage(CanFdMessage::read(cursor, header)?))),
            ObjectType::CanFdMessage64 => Ok(Some(LogObject::CanFdMessage64(CanFdMessage64::read(cursor, header)?))),
            ObjectType::CanOverload => Ok(Some(LogObject::CanOverloadFrame(CanOverloadFrame::read(cursor, header)?))),
            ObjectType::CanStatistic => Ok(Some(LogObject::CanDriverStatistic(CanDriverStatistic::read(cursor, header)?))),
            ObjectType::CanDriverError => Ok(Some(LogObject::CanDriverError(CanDriverError::read(cursor, header)?))),
            _ => self.parse_lin_object(cursor, header, object_data_size)
        }
    }

    fn parse_lin_object(&self, cursor: &mut Cursor<&[u8]>, header: &ObjectHeader, object_data_size: usize) -> BlfParseResult<Option<LogObject>> {
        match header.object_type {
            ObjectType::LinMessage => Ok(Some(LogObject::LinMessage(LinMessage::read(cursor, header)?))),
            ObjectType::LinCrcError => Ok(Some(LogObject::LinCrcError(LinCrcError::read(cursor, header)?))),
            ObjectType::LinDlcInfo => Ok(Some(LogObject::LinDlcInfo(LinDlcInfo::read(cursor, header)?))),
            ObjectType::LinReceiveError => Ok(Some(LogObject::LinReceiveError(LinReceiveError::read(cursor, header)?))),
            ObjectType::LinSendError => Ok(Some(LogObject::LinSendError(LinSendError::read(cursor, header)?))),
            ObjectType::LinSlaveTimeout => Ok(Some(LogObject::LinSlaveTimeout(LinSlaveTimeout::read(cursor, header)?))),
            ObjectType::LinSchedulerModeChange => Ok(Some(LogObject::LinSchedulerModeChange(LinSchedulerModeChange::read(cursor, header)?))),
            ObjectType::LinSyncError => Ok(Some(LogObject::LinSyncError(LinSyncError::read(cursor, header)?))),
            ObjectType::LinBaudrate => Ok(Some(LogObject::LinBaudrateEvent(LinBaudrateEvent::read(cursor, header)?))),
            ObjectType::LinSleep => Ok(Some(LogObject::LinSleepModeEvent(LinSleepModeEvent::read(cursor, header)?))),
            ObjectType::LinWakeup => Ok(Some(LogObject::LinWakeupEvent(LinWakeupEvent::read(cursor, header)?))),
            ObjectType::LinMessage2 => Ok(Some(LogObject::LinMessage2(LinMessage2::read(cursor, header, object_data_size)?))),
            _ => self.parse_flexray_object(cursor, &header.clone(), object_data_size)
        }
    }
    fn parse_flexray_object(&self, cursor: &mut Cursor<&[u8]>, header: &ObjectHeader, object_data_size: usize) -> BlfParseResult<Option<LogObject>> {
        match header.object_type {
            ObjectType::FlexRayData => Ok(Some(LogObject::FlexRayData(FlexRayData::read(cursor, header)?))),
            ObjectType::FlexRaySync => Ok(Some(LogObject::FlexRaySync(FlexRaySync::read(cursor, header)?))),
            ObjectType::FlexRayMessage => Ok(Some(LogObject::FlexRayV6Message(FlexRayV6Message::read(cursor, header)?))),
            ObjectType::FlexRayV6StartCycleEvent => Ok(Some(LogObject::FlexRayV6StartCycleEvent(FlexRayV6StartCycleEvent::read(cursor, header)?))),
            ObjectType::FlexRayStatusEvent => Ok(Some(LogObject::FlexRayStatusEvent(FlexRayStatusEvent::read(cursor, header)?))),
            ObjectType::FlexRayVFrError => Ok(Some(LogObject::FlexRayVFrError(FlexRayVFrError::read(cursor, header)?))),
            ObjectType::FlexRayVFrStatus => Ok(Some(LogObject::FlexRayVFrStatus(FlexRayVFrStatus::read(cursor, header)?))),
            ObjectType::FlexRayVFrStartCycle => Ok(Some(LogObject::FlexRayVFrStartCycle(FlexRayVFrStartCycle::read(cursor, header)?))),
            ObjectType::FlexRayVFrReceiveMsg => Ok(Some(LogObject::FlexRayVFrReceiveMsg(FlexRayVFrReceiveMsg::read(cursor, header)?))),
            ObjectType::FlexRayVFrReceiveMsgEx => Ok(Some(LogObject::FlexRayVFrReceiveMsgEx(FlexRayVFrReceiveMsgEx::read(cursor, header)?))),
            _ => self.parse_other_object(cursor, &header.clone(), object_data_size)
        }
    }
    fn parse_other_object(&self, cursor: &mut Cursor<&[u8]>, header: &ObjectHeader, object_data_size: usize) -> BlfParseResult<Option<LogObject>> {
        match header.object_type {
            ObjectType::EthernetFrame => Ok(Some(LogObject::EthernetFrame(EthernetFrame::read(cursor, header)?))),
            ObjectType::AppTrigger => Ok(Some(LogObject::AppTrigger(AppTrigger::read(cursor, header)?))),
            ObjectType::EventComment => Ok(Some(LogObject::EventComment(EventComment::read(cursor, header)?))),
            ObjectType::GlobalMarker => Ok(Some(LogObject::GlobalMarker(GlobalMarker::read(cursor, header)?))),
            // Temporarily comment out missing types
            // ObjectType::SystemVariable => Ok(Some(LogObject::SystemVariable(SystemVariable::read(cursor, header)?))),
            // ObjectType::EnvInteger => Ok(Some(LogObject::EnvInteger(EnvInteger::read(cursor, header)?))),
            // ObjectType::EnvDouble => Ok(Some(LogObject::EnvDouble(EnvDouble::read(cursor, header)?))),
            _ => self.parse_most_object(cursor, header.clone(), object_data_size)
        }
    }
    fn parse_most_object(&self, cursor: &mut Cursor<&[u8]>, header: ObjectHeader, _object_data_size: usize) -> BlfParseResult<Option<LogObject>> {
        match header.object_type {
            // Temporarily comment out EnvString since it's not yet implemented
            // ObjectType::EnvString => Ok(Some(LogObject::EnvString(EnvString::read(cursor, &header, object_data_size)?))),
            ObjectType::MostSpy => Ok(Some(LogObject::MostSpy(MostSpy::read(cursor, &header)?))),
            ObjectType::MostCtrl => Ok(Some(LogObject::MostCtrl(MostCtrl::read(cursor, &header)?))),
            ObjectType::MostPkt2 => Ok(Some(LogObject::MostPkt2(MostPkt2::read(cursor, &header)?))),
            ObjectType::MostLightLock => Ok(Some(LogObject::MostLightLock(MostLightLock::read(cursor, &header)?))),
            ObjectType::MostStatistic => Ok(Some(LogObject::MostStatistic(MostStatistic::read(cursor, &header)?))),
            ObjectType::MostHwMode => Ok(Some(LogObject::MostHwMode(MostHwMode::read(cursor, &header)?))),
            ObjectType::MostReg => Ok(Some(LogObject::MostReg(MostReg::read(cursor, &header)?))),
            ObjectType::MostGenReg => Ok(Some(LogObject::MostGenReg(MostGenReg::read(cursor, &header)?))),
            ObjectType::MostNetState => Ok(Some(LogObject::MostNetState(MostNetState::read(cursor, &header)?))),
            ObjectType::MostDataLost => Ok(Some(LogObject::MostDataLost(MostDataLost::read(cursor, &header)?))),
            ObjectType::MostTrigger => Ok(Some(LogObject::MostTrigger(MostTrigger::read(cursor, &header)?))),
            _ => self.parse_unhandled_object()
        }
    }

    /// Parses the actual log objects contained within a (decompressed) LogContainer.
    fn parse_inner_objects(&self, cursor: &mut Cursor<&[u8]>) -> BlfParseResult<Vec<LogObject>> {
        let mut all_objects = Vec::new();
        let data_len = cursor.get_ref().len();
        println!("Parsing inner objects, data_len: {}", data_len);

        while (cursor.position() as usize) < data_len {
            let start_pos = cursor.position();
            println!("Reading object at position: {}", start_pos);
            
            // Check if we have enough bytes to read the signature
            if (data_len as u64 - start_pos) < 4 {
                break;
            }
            
            // Try to read the header, but handle the case where there's no valid object left
            let header = match ObjectHeader::read(cursor) {
                Ok(header) => header,
                Err(BlfParseError::InvalidContainerMagic) => {
                    // If we can't read a valid header due to magic number, skip one byte and try again
                    cursor.set_position(start_pos + 1);
                    continue;
                }
                Err(e) => return Err(e),
            };

            println!("Parsing object {:?} with size {}", header.object_type, header.object_size);
            // LogContainers should not be nested. If they are, we skip them to avoid infinite recursion.
            if header.object_type != ObjectType::LogContainer {
                let object_body_size = (header.object_size as usize).saturating_sub(header.calculate_header_size() as usize);
                println!("Object body size: {}", object_body_size);
                if let Some(object) = self.parse_can_object(cursor, &header, object_body_size)? {
                    all_objects.push(object);
                }
            } else {
                // For LogContainer objects, we skip them but still need to advance the cursor
                println!("Skipping LogContainer object");
            }
            
            // 在LogContainer内部，对象已经通过add_padding进行了4字节对齐
            // 所以我们直接使用对象大小，而不是进行额外的对齐
            let next_pos = start_pos + header.object_size as u64;
            println!("Advancing to next position: {} (current: {})", next_pos, start_pos);
            
            // Make sure we advance the cursor position to avoid infinite loops
            if next_pos <= start_pos {
                // If object_size is 0 or invalid, advance by 1 byte to avoid infinite loop
                cursor.set_position(start_pos + 1);
            } else {
                cursor.set_position(next_pos.min(data_len as u64));
            }
            
            // Additional safeguard to prevent infinite loops
            if header.object_size == 0 {
                cursor.set_position(start_pos + 1);
            }
            
            // Ensure 4-byte alignment
            let current_pos = cursor.position();
            let aligned_pos = (current_pos + 3) & !3;  // Round up to next multiple of 4
            cursor.set_position(aligned_pos.min(data_len as u64));
        }
        println!("Finished parsing, found {} objects", all_objects.len());
        Ok(all_objects)
    }

    /// Advances the cursor to the start of the next object, including padding.
    /// This logic is based on `is.seekg(objectSize % 4, std::ios_base::cur);` from the C++ source.
    /// The C++ implementation `objectSize % 4` is a non-standard way to achieve 4-byte alignment
    /// that happens to work in most cases but is conceptually incorrect when `objectSize` is a multiple of 4.
    /// We use a more robust and idiomatic method `(object_size + 3) & !3` to round up to the next
    /// multiple of 4, which correctly handles all cases and clarifies the intent of alignment.
    fn advance_cursor_to_next_object(&self, cursor: &mut Cursor<&[u8]>, start_pos: u64, object_size: u32) {
        let padded_size = (object_size as u64 + 3) & !3;
        let next_pos = start_pos + padded_size;
        cursor.set_position(next_pos);
    }

    fn parse_unhandled_object(&self) -> BlfParseResult<Option<LogObject>> {
        // The `parse_objects` loop handles advancing the cursor to the next object.
        // For unhandled or deprecated object types, we simply do nothing and return `None`.
        // The cursor will be advanced correctly regardless of whether we parsed the object or not.
        //
        // This includes:
        // - ObjectType::MostPkt: Deprecated and should be skipped.
        // - Any other unknown ObjectType.
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_parse_inner_objects_single_can_message() {
        let parser = BlfParser::new();
        let can_message = CanMessage {
            header: ObjectHeader {
                signature: 0x4A424F4C, // "LOBJ"
                header_size: 32,
                header_version: 1,
                object_size: 48, // header + can_msg_fields + data
                object_type: ObjectType::CanMessage,
                object_flags: 0,
                object_time_stamp: 1000,
                original_time_stamp: None,
                time_stamp_status: None,
            },
            channel: 1,
            flags: 0,
            dlc: 8,
            id: 0x123,
            data: [1, 2, 3, 4, 5, 6, 7, 8],
        };

        // Create a LogContainer to wrap the CAN message
        let mut can_msg_bytes = serialize_can_message(&can_message);
        println!("can_msg_bytes before padding: {:?}", can_msg_bytes);
        add_padding(&mut can_msg_bytes);
        println!("can_msg_bytes after padding: {:?}", can_msg_bytes);
        
        let mut cursor = Cursor::new(&can_msg_bytes[..]);
        let result = parser.parse_inner_objects(&mut cursor).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], LogObject::CanMessage(can_message));
    }

    #[test]
    fn test_parse_inner_objects_multiple_objects() {
        let parser = BlfParser::new();
        let can_message1 = CanMessage {
            header: ObjectHeader {
                signature: 0x4A424F4C, // "LOBJ"
                header_size: 32,
                header_version: 1,
                object_size: 48,
                object_type: ObjectType::CanMessage,
                object_flags: 0,
                object_time_stamp: 1000,
                original_time_stamp: None,
                time_stamp_status: None,
            },
            channel: 1,
            flags: 0,
            dlc: 8,
            id: 0x111,
            data: [0; 8],
        };
        let can_message2 = CanMessage {
            header: ObjectHeader {
                signature: 0x4A424F4C, // "LOBJ"
                header_size: 32,
                header_version: 1,
                object_size: 48,
                object_type: ObjectType::CanMessage,
                object_flags: 0,
                object_time_stamp: 1000,
                original_time_stamp: None,
                time_stamp_status: None,
            },
            channel: 1,
            flags: 0,
            dlc: 8,
            id: 0x222,
            data: [0; 8],
        };

        // Create a LogContainer to wrap the CAN messages
        let mut bytes1 = serialize_can_message(&can_message1);
        println!("bytes1 before padding: {:?}", bytes1);
        add_padding(&mut bytes1);
        println!("bytes1 after padding: {:?}", bytes1);
        let mut bytes2 = serialize_can_message(&can_message2);
        println!("bytes2 before padding: {:?}", bytes2);
        add_padding(&mut bytes2);
        println!("bytes2 after padding: {:?}", bytes2);

        let combined_bytes = [bytes1, bytes2].concat();
        println!("combined_bytes: {:?}", combined_bytes);
        
        let mut cursor = Cursor::new(&combined_bytes[..]);
        let result = parser.parse_inner_objects(&mut cursor).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], LogObject::CanMessage(can_message1));
        assert_eq!(result[1], LogObject::CanMessage(can_message2));
    }

    #[test]
    fn test_parse_inner_objects_skips_unknown_object() {
        use std::io::Write;
        use crate::ObjectType;
        
        let parser = BlfParser::new();
        
        // Create a LogContainer to wrap the unknown object
        let unknown_header = ObjectHeader {
            signature: 0x4A424F4C, // "LOBJ"
            header_size: 32,
            header_version: 1,
            object_size: 38, // Does not need to be a multiple of 4
            object_type: ObjectType::Unknown,
            object_flags: 0,
            object_time_stamp: 1000,
            original_time_stamp: None,
            time_stamp_status: None,
        };

        let mut unknown_object_bytes = Vec::new();
        serialize_object_header(&unknown_header, &mut unknown_object_bytes);
        unknown_object_bytes.write_all(&vec![0; (unknown_header.object_size - unknown_header.header_size as u32) as usize]).unwrap();
        add_padding(&mut unknown_object_bytes);
        
        let mut cursor = Cursor::new(&unknown_object_bytes[..]);
        let result = parser.parse_inner_objects(&mut cursor).unwrap();

        // The parser should gracefully skip the unknown object and return an empty list.
        assert!(result.is_empty());
    }
}
