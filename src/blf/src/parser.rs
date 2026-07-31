//! BLF parser implementation.
//! BLF parser module for parsing log objects from BLF files.

use crate::objects::*;
use crate::{BlfParseError, BlfParseResult, LogContainer, ObjectType};
use crate::BlfResultContext;

use rayon::prelude::*;
use std::io::{Cursor, Read};

// Log object enum for all supported BLF objects
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
    // Environment variables
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
    // Placeholder for unhandled objects
    Unhandled {
        object_type: u32,
        timestamp: u64,
        data: Vec<u8>,
    },
}

impl LogObject {
    /// Returns the timestamp of the log object
    pub fn timestamp(&self) -> u64 {
        match self {
            LogObject::CanMessage(msg) => msg.header.object_time_stamp,
            LogObject::CanMessage2(msg) => msg.header.object_time_stamp,
            LogObject::CanErrorFrame(msg) => msg.header.object_time_stamp,
            LogObject::CanFdMessage(msg) => msg.header.object_time_stamp,
            LogObject::CanFdMessage64(msg) => msg.header.object_time_stamp,
            LogObject::CanOverloadFrame(msg) => msg.header.object_time_stamp,
            LogObject::CanDriverStatistic(msg) => msg.header.object_time_stamp,
            LogObject::CanDriverError(msg) => msg.header.object_time_stamp,
            LogObject::LinMessage(msg) => msg.header.object_time_stamp,
            LogObject::LinMessage2(msg) => msg.header.object_time_stamp,
            LogObject::LinCrcError(msg) => msg.header.object_time_stamp,
            LogObject::LinDlcInfo(msg) => msg.header.object_time_stamp,
            LogObject::LinReceiveError(msg) => msg.header.object_time_stamp,
            LogObject::LinSendError(msg) => msg.header.object_time_stamp,
            LogObject::LinSlaveTimeout(msg) => msg.header.object_time_stamp,
            LogObject::LinSchedulerModeChange(msg) => msg.header.object_time_stamp,
            LogObject::LinSyncError(msg) => msg.header.object_time_stamp,
            LogObject::LinBaudrateEvent(msg) => msg.header.object_time_stamp,
            LogObject::LinSleepModeEvent(msg) => msg.header.object_time_stamp,
            LogObject::LinWakeupEvent(msg) => msg.header.object_time_stamp,
            LogObject::FlexRayData(msg) => msg.timestamp,
            LogObject::FlexRaySync(msg) => msg.timestamp,
            LogObject::FlexRayV6Message(msg) => msg.timestamp,
            LogObject::FlexRayV6StartCycleEvent(msg) => msg.timestamp,
            LogObject::FlexRayStatusEvent(msg) => msg.timestamp,
            LogObject::FlexRayVFrError(msg) => msg.timestamp,
            LogObject::FlexRayVFrStatus(msg) => msg.timestamp,
            LogObject::FlexRayVFrStartCycle(msg) => msg.timestamp,
            LogObject::FlexRayVFrReceiveMsg(msg) => msg.timestamp,
            LogObject::FlexRayVFrReceiveMsgEx(msg) => msg.timestamp,
            LogObject::EthernetFrame(msg) => msg.timestamp,
            LogObject::AppTrigger(msg) => msg.timestamp,
            LogObject::EventComment(msg) => msg.timestamp,
            LogObject::GlobalMarker(msg) => msg.timestamp,
            LogObject::MostSpy(msg) => msg.timestamp,
            LogObject::MostCtrl(msg) => msg.timestamp,
            LogObject::MostPkt2(msg) => msg.timestamp,
            LogObject::MostLightLock(msg) => msg.timestamp,
            LogObject::MostStatistic(msg) => msg.timestamp,
            LogObject::MostHwMode(msg) => msg.timestamp,
            LogObject::MostReg(msg) => msg.timestamp,
            LogObject::MostGenReg(msg) => msg.timestamp,
            LogObject::MostNetState(msg) => msg.timestamp,
            LogObject::MostDataLost(msg) => msg.timestamp,
            LogObject::MostTrigger(msg) => msg.timestamp,
            LogObject::Unhandled { timestamp, .. } => *timestamp,
        }
    }

    /// Overwrite the timestamp stored in this log object. Used by
    /// multi-file merge to rewrite each message's relative timestamp
    /// to a global absolute timestamp (in nanoseconds since Unix epoch)
    /// so that messages from different files sort and display uniformly.
    pub fn set_timestamp(&mut self, ts: u64) {
        match self {
            LogObject::CanMessage(msg) => msg.header.object_time_stamp = ts,
            LogObject::CanMessage2(msg) => msg.header.object_time_stamp = ts,
            LogObject::CanErrorFrame(msg) => msg.header.object_time_stamp = ts,
            LogObject::CanFdMessage(msg) => msg.header.object_time_stamp = ts,
            LogObject::CanFdMessage64(msg) => msg.header.object_time_stamp = ts,
            LogObject::CanOverloadFrame(msg) => msg.header.object_time_stamp = ts,
            LogObject::CanDriverStatistic(msg) => msg.header.object_time_stamp = ts,
            LogObject::CanDriverError(msg) => msg.header.object_time_stamp = ts,
            LogObject::LinMessage(msg) => msg.header.object_time_stamp = ts,
            LogObject::LinMessage2(msg) => msg.header.object_time_stamp = ts,
            LogObject::LinCrcError(msg) => msg.header.object_time_stamp = ts,
            LogObject::LinDlcInfo(msg) => msg.header.object_time_stamp = ts,
            LogObject::LinReceiveError(msg) => msg.header.object_time_stamp = ts,
            LogObject::LinSendError(msg) => msg.header.object_time_stamp = ts,
            LogObject::LinSlaveTimeout(msg) => msg.header.object_time_stamp = ts,
            LogObject::LinSchedulerModeChange(msg) => msg.header.object_time_stamp = ts,
            LogObject::LinSyncError(msg) => msg.header.object_time_stamp = ts,
            LogObject::LinBaudrateEvent(msg) => msg.header.object_time_stamp = ts,
            LogObject::LinSleepModeEvent(msg) => msg.header.object_time_stamp = ts,
            LogObject::LinWakeupEvent(msg) => msg.header.object_time_stamp = ts,
            LogObject::FlexRayData(msg) => msg.timestamp = ts,
            LogObject::FlexRaySync(msg) => msg.timestamp = ts,
            LogObject::FlexRayV6Message(msg) => msg.timestamp = ts,
            LogObject::FlexRayV6StartCycleEvent(msg) => msg.timestamp = ts,
            LogObject::FlexRayStatusEvent(msg) => msg.timestamp = ts,
            LogObject::FlexRayVFrError(msg) => msg.timestamp = ts,
            LogObject::FlexRayVFrStatus(msg) => msg.timestamp = ts,
            LogObject::FlexRayVFrStartCycle(msg) => msg.timestamp = ts,
            LogObject::FlexRayVFrReceiveMsg(msg) => msg.timestamp = ts,
            LogObject::FlexRayVFrReceiveMsgEx(msg) => msg.timestamp = ts,
            LogObject::EthernetFrame(msg) => msg.timestamp = ts,
            LogObject::AppTrigger(msg) => msg.timestamp = ts,
            LogObject::EventComment(msg) => msg.timestamp = ts,
            LogObject::GlobalMarker(msg) => msg.timestamp = ts,
            LogObject::MostSpy(msg) => msg.timestamp = ts,
            LogObject::MostCtrl(msg) => msg.timestamp = ts,
            LogObject::MostPkt2(msg) => msg.timestamp = ts,
            LogObject::MostLightLock(msg) => msg.timestamp = ts,
            LogObject::MostStatistic(msg) => msg.timestamp = ts,
            LogObject::MostHwMode(msg) => msg.timestamp = ts,
            LogObject::MostReg(msg) => msg.timestamp = ts,
            LogObject::MostGenReg(msg) => msg.timestamp = ts,
            LogObject::MostNetState(msg) => msg.timestamp = ts,
            LogObject::MostDataLost(msg) => msg.timestamp = ts,
            LogObject::MostTrigger(msg) => msg.timestamp = ts,
            LogObject::Unhandled { timestamp, .. } => *timestamp = ts,
        }
    }

    /// Returns the `object_flags` of the log object (BLF spec: 0x1=TimeTenMics,
    /// 0x2=TimeOneNans). For object types that don't have an ObjectHeader
    /// (FlexRay, Ethernet, Most, AppTrigger, EventComment, GlobalMarker, Unhandled),
    /// returns 0x2 (TimeOneNans) as a safe default — these types already store
    /// an absolute-nanosecond timestamp in their `timestamp` field (per
    /// `timestamp()` impl above).
    pub fn flags(&self) -> u32 {
        match self {
            LogObject::CanMessage(msg) => msg.header.object_flags,
            LogObject::CanMessage2(msg) => msg.header.object_flags,
            LogObject::CanErrorFrame(msg) => msg.header.object_flags,
            LogObject::CanFdMessage(msg) => msg.header.object_flags,
            LogObject::CanFdMessage64(msg) => msg.header.object_flags,
            LogObject::CanOverloadFrame(msg) => msg.header.object_flags,
            LogObject::CanDriverStatistic(msg) => msg.header.object_flags,
            LogObject::CanDriverError(msg) => msg.header.object_flags,
            LogObject::LinMessage(msg) => msg.header.object_flags,
            LogObject::LinMessage2(msg) => msg.header.object_flags,
            LogObject::LinCrcError(msg) => msg.header.object_flags,
            LogObject::LinDlcInfo(msg) => msg.header.object_flags,
            LogObject::LinReceiveError(msg) => msg.header.object_flags,
            LogObject::LinSendError(msg) => msg.header.object_flags,
            LogObject::LinSlaveTimeout(msg) => msg.header.object_flags,
            LogObject::LinSchedulerModeChange(msg) => msg.header.object_flags,
            LogObject::LinSyncError(msg) => msg.header.object_flags,
            LogObject::LinBaudrateEvent(msg) => msg.header.object_flags,
            LogObject::LinSleepModeEvent(msg) => msg.header.object_flags,
            LogObject::LinWakeupEvent(msg) => msg.header.object_flags,
            _ => 0x2, // FlexRay/Ethernet/Most/etc. store abs_ns already
        }
    }

    /// Returns the timestamp converted to **nanoseconds** based on
    /// `object_flags` (per BLF spec):
    /// - `TimeTenMics` (0x1): raw ticks × 10µs = ticks × 10000 ns
    /// - `TimeOneNans` (0x2) or default: raw ticks = ns
    pub fn timestamp_nanos(&self) -> u64 {
        let raw = self.timestamp();
        let flags = self.flags();
        if flags & 0x1 != 0 {
            // TimeTenMics: 10µs per tick → × 10000 ns
            raw.saturating_mul(10_000)
        } else {
            // TimeOneNans (default): 1ns per tick
            raw
        }
    }

    /// Returns the channel ID of the log object (if applicable)
    pub fn channel(&self) -> Option<u16> {
        match self {
            LogObject::CanMessage(msg) => Some(msg.channel),
            LogObject::CanMessage2(msg) => Some(msg.channel),
            LogObject::CanFdMessage(msg) => Some(msg.channel),
            LogObject::CanFdMessage64(msg) => Some(msg.channel as u16),
            LogObject::LinMessage(msg) => Some(msg.channel),
            LogObject::LinMessage2(_msg) => None, // LinMessage2 doesn't have a direct channel field
            _ => None,
        }
    }
}

/// BLF parser for handling log objects
#[derive(Debug, Default)]
pub struct BlfParser {
    /// Enable debug logging
    pub debug: bool,
}

impl BlfParser {
    /// Creates a new BlfParser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new BlfParser with debug logging enabled.
    pub fn with_debug() -> Self {
        Self { debug: true }
    }

    /// Parses the data slice and returns a vector of log objects and a vector of errors.
    pub fn parse(&self, data: &[u8]) -> BlfParseResult<(Vec<LogObject>, Vec<BlfParseError>, u64)> {
        let mut cursor = Cursor::new(data);
        let mut all_objects = Vec::new();
        let mut all_errors = Vec::new();
        let data_len = cursor.get_ref().len();

        if self.debug {
            println!("Starting to parse {} bytes of BLF data", data_len);
        }

        // The top-level of a BLF file (after the initial FileStatistics header, which is handled elsewhere)
        // consists of a series of LogContainer objects. LogContainers are independent:
        // each has its own zlib stream and inner objects that don't reference other containers.
        // We exploit this by collecting container descriptors in a single pass, then
        // decompressing + parsing their inner objects in parallel with rayon. Per-container
        // results are collected back in original file order so timestamps stay monotonic.
        let mut pending_containers: Vec<(usize, ObjectHeaderBase)> = Vec::new();

        while (cursor.position() as usize) < data_len {
            let start_pos = cursor.position();

            // Check if we have enough data for a header
            if (cursor.position() as usize) + 32 > data_len {
                if self.debug {
                    println!(
                        "Not enough data remaining for a complete header at position {}",
                        cursor.position()
                    );
                }
                break;
            }

            let header_result = ObjectHeaderBase::read(&mut cursor).context("BlfParser.ObjectHeaderBase");
            let header = match header_result {
                Ok(h) => h,
                Err(e) => {
                    if self.debug {
                        println!(
                            "Failed to read object header at position {}: {:?}",
                            start_pos, e
                        );
                    }
                    all_errors.push(e);
                    // Try to skip some bytes and continue
                    cursor.set_position(start_pos + 4);
                    continue;
                }
            };

            if self.debug {
                println!(
                    "Read object header: type={:?}, size={}",
                    header.object_type, header.object_size,
                );
            }

            // Validate object size
            if header.object_size < header.header_size as u32 {
                if self.debug {
                    println!(
                        "Invalid object size: {} < header size: {}",
                        header.object_size, header.header_size
                    );
                }
                // We don't have a specific error for this, maybe just log debug or add a custom error if strict?
                // For now, let's treat it as a skip, but maybe we should report?
                // all_errors.push(BlfParseError::InvalidObjectSize); // Logic for error handling
                self.advance_cursor_to_next_object(&mut cursor, start_pos, 32);
                continue;
            }

            if header.object_type != ObjectType::LogContainer {
                if self.debug {
                    println!(
                        "Non-container object at top level: {:?}, skipping",
                        header.object_type
                    );
                }
            } else {
                if self.debug {
                    println!("Parsing container {}", header.object_size);
                }
                // Defer the actual decompress+parse to the parallel phase. Capture
                // the byte range of this container so the worker can re-read it
                // from the original `data` slice without copying.
                pending_containers.push((start_pos as usize, header.clone()));
            }
            self.advance_cursor_to_next_object(&mut cursor, start_pos, header.object_size);
        }

        let consumed = cursor.position().min(data_len as u64);

        // Parallel phase: decompress + parse_inner_objects for each container.
        // Each task returns (container_index, Result<Vec<LogObject>, BlfParseError>).
        // The original `data` slice is shared (read-only) across threads via &data[..].
        let per_container_results: Vec<(usize, BlfParseResult<Vec<LogObject>>)> = pending_containers
            .into_par_iter()
            .map(|(start_pos, header)| {
                // Worker cursor starts at the LogContainer payload (after the
                // ObjectHeaderBase), since LogContainer::read expects to read
                // the compression_method field next, not the header.
                let payload_start = start_pos + header.header_size as usize;
                let mut worker_cursor = Cursor::new(&data[payload_start..]);
                let inner_result = LogContainer::read(&mut worker_cursor, header)
                    .context("BlfParser.LogContainer")
                    .and_then(|container| {
                        let mut inner_cursor = Cursor::new(&container.uncompressed_data[..]);
                        self.parse_inner_objects(&mut inner_cursor)
                            .context("BlfParser.parse_inner_objects")
                    });
                (start_pos, inner_result)
            })
            .collect();

        // Sequential merge phase: preserve original container order. The
        // pending_containers Vec was built in file order, and `into_par_iter()`
        // preserves order through `collect()`, so iterating results in order
        // gives us the same sequence as the old sequential code.
        for (_start_pos, result) in per_container_results {
            match result {
                Ok(objects) => {
                    if self.debug {
                        println!(
                            "Successfully parsed {} objects from container",
                            objects.len()
                        );
                    }
                    all_objects.extend(objects);
                }
                Err(e) => {
                    if self.debug {
                        println!("Error parsing inner objects: {:?}", e);
                    }
                    all_errors.push(e);
                }
            }
        }

        if self.debug {
            println!(
                "Parsing complete, found {} objects total, {} errors",
                all_objects.len(),
                all_errors.len()
            );
        }

        Ok((all_objects, all_errors, consumed))
    }

    fn parse_can_object(
        &self,
        cursor: &mut Cursor<&[u8]>,
        header: &ObjectHeader,
        object_data_size: usize,
    ) -> BlfParseResult<Option<LogObject>> {
        match header.object_type {
            ObjectType::CanMessage => Ok(Some(LogObject::CanMessage(CanMessage::read(
                cursor, header,
            )?))),
            ObjectType::CanMessage2 => Ok(Some(LogObject::CanMessage2(CanMessage2::read(
                cursor,
                header,
                object_data_size,
            )?))),
            ObjectType::CanError => Ok(Some(LogObject::CanErrorFrame(CanErrorFrame::read(
                cursor, header,
            )?))),
            ObjectType::CanFdMessage => Ok(Some(LogObject::CanFdMessage(CanFdMessage::read(
                cursor, header,
            )?))),
            ObjectType::CanFdMessage64 => Ok(Some(LogObject::CanFdMessage64(
                CanFdMessage64::read(cursor, header)?,
            ))),
            ObjectType::CanOverload => Ok(Some(LogObject::CanOverloadFrame(
                CanOverloadFrame::read(cursor, header)?,
            ))),
            ObjectType::CanStatistic => Ok(Some(LogObject::CanDriverStatistic(
                CanDriverStatistic::read(cursor, header)?,
            ))),
            ObjectType::CanDriverError => Ok(Some(LogObject::CanDriverError(
                CanDriverError::read(cursor, header)?,
            ))),
            _ => self.parse_lin_object(cursor, header, object_data_size),
        }
    }

    fn parse_lin_object(
        &self,
        cursor: &mut Cursor<&[u8]>,
        header: &ObjectHeader,
        object_data_size: usize,
    ) -> BlfParseResult<Option<LogObject>> {
        match header.object_type {
            ObjectType::LinMessage => Ok(Some(LogObject::LinMessage(LinMessage::read(
                cursor, header,
            )?))),
            ObjectType::LinCrcError => Ok(Some(LogObject::LinCrcError(LinCrcError::read(
                cursor, header,
            )?))),
            ObjectType::LinDlcInfo => Ok(Some(LogObject::LinDlcInfo(LinDlcInfo::read(
                cursor, header,
            )?))),
            ObjectType::LinReceiveError => Ok(Some(LogObject::LinReceiveError(
                LinReceiveError::read(cursor, header)?,
            ))),
            ObjectType::LinSendError => Ok(Some(LogObject::LinSendError(LinSendError::read(
                cursor, header,
            )?))),
            ObjectType::LinSlaveTimeout => Ok(Some(LogObject::LinSlaveTimeout(
                LinSlaveTimeout::read(cursor, header)?,
            ))),
            ObjectType::LinSchedulerModeChange => Ok(Some(LogObject::LinSchedulerModeChange(
                LinSchedulerModeChange::read(cursor, header)?,
            ))),
            ObjectType::LinSyncError => Ok(Some(LogObject::LinSyncError(LinSyncError::read(
                cursor, header,
            )?))),
            ObjectType::LinBaudrate => Ok(Some(LogObject::LinBaudrateEvent(
                LinBaudrateEvent::read(cursor, header)?,
            ))),
            ObjectType::LinSleep => Ok(Some(LogObject::LinSleepModeEvent(
                LinSleepModeEvent::read(cursor, header)?,
            ))),
            ObjectType::LinWakeup => Ok(Some(LogObject::LinWakeupEvent(LinWakeupEvent::read(
                cursor, header,
            )?))),
            ObjectType::LinMessage2 => Ok(Some(LogObject::LinMessage2(LinMessage2::read(
                cursor,
                header,
                object_data_size,
            )?))),
            _ => self.parse_flexray_object(cursor, &header.clone(), object_data_size),
        }
    }
    fn parse_flexray_object(
        &self,
        cursor: &mut Cursor<&[u8]>,
        header: &ObjectHeader,
        object_data_size: usize,
    ) -> BlfParseResult<Option<LogObject>> {
        match header.object_type {
            ObjectType::FlexRayData => Ok(Some(LogObject::FlexRayData(FlexRayData::read(
                cursor, header,
            )?))),
            ObjectType::FlexRaySync => Ok(Some(LogObject::FlexRaySync(FlexRaySync::read(
                cursor, header,
            )?))),
            ObjectType::FlexRayMessage => Ok(Some(LogObject::FlexRayV6Message(
                FlexRayV6Message::read(cursor, header)?,
            ))),
            ObjectType::FlexRayV6StartCycleEvent => Ok(Some(LogObject::FlexRayV6StartCycleEvent(
                FlexRayV6StartCycleEvent::read(cursor, header)?,
            ))),
            ObjectType::FlexRayStatusEvent => Ok(Some(LogObject::FlexRayStatusEvent(
                FlexRayStatusEvent::read(cursor, header)?,
            ))),
            ObjectType::FlexRayVFrError => Ok(Some(LogObject::FlexRayVFrError(
                FlexRayVFrError::read(cursor, header)?,
            ))),
            ObjectType::FlexRayVFrStatus => Ok(Some(LogObject::FlexRayVFrStatus(
                FlexRayVFrStatus::read(cursor, header)?,
            ))),
            ObjectType::FlexRayVFrStartCycle => Ok(Some(LogObject::FlexRayVFrStartCycle(
                FlexRayVFrStartCycle::read(cursor, header)?,
            ))),
            ObjectType::FlexRayVFrReceiveMsg => Ok(Some(LogObject::FlexRayVFrReceiveMsg(
                FlexRayVFrReceiveMsg::read(cursor, header)?,
            ))),
            ObjectType::FlexRayVFrReceiveMsgEx => Ok(Some(LogObject::FlexRayVFrReceiveMsgEx(
                FlexRayVFrReceiveMsgEx::read(cursor, header)?,
            ))),
            _ => self.parse_other_object(cursor, &header.clone(), object_data_size),
        }
    }
    fn parse_other_object(
        &self,
        cursor: &mut Cursor<&[u8]>,
        header: &ObjectHeader,
        object_data_size: usize,
    ) -> BlfParseResult<Option<LogObject>> {
        match header.object_type {
            ObjectType::EthernetFrame => Ok(Some(LogObject::EthernetFrame(EthernetFrame::read(
                cursor, header,
            )?))),
            ObjectType::AppTrigger => Ok(Some(LogObject::AppTrigger(AppTrigger::read(
                cursor, header,
            )?))),
            ObjectType::EventComment => Ok(Some(LogObject::EventComment(EventComment::read(
                cursor, header,
            )?))),
            ObjectType::GlobalMarker => Ok(Some(LogObject::GlobalMarker(GlobalMarker::read(
                cursor, header,
            )?))),
            // Temporarily comment out missing types
            // ObjectType::SystemVariable => Ok(Some(LogObject::SystemVariable(SystemVariable::read(cursor, header)?))),
            // ObjectType::EnvInteger => Ok(Some(LogObject::EnvInteger(EnvInteger::read(cursor, header)?))),
            // ObjectType::EnvDouble => Ok(Some(LogObject::EnvDouble(EnvDouble::read(cursor, header)?))),
            _ => self.parse_most_object(cursor, header.clone(), object_data_size),
        }
    }
    fn parse_most_object(
        &self,
        cursor: &mut Cursor<&[u8]>,
        header: ObjectHeader,
        object_data_size: usize,
    ) -> BlfParseResult<Option<LogObject>> {
        match header.object_type {
            // Temporarily comment out EnvString since it's not yet implemented
            // ObjectType::EnvString => Ok(Some(LogObject::EnvString(EnvString::read(cursor, &header, object_data_size)?))),
            ObjectType::MostSpy => Ok(Some(LogObject::MostSpy(MostSpy::read(cursor, &header)?))),
            ObjectType::MostCtrl => Ok(Some(LogObject::MostCtrl(MostCtrl::read(cursor, &header)?))),
            ObjectType::MostPkt2 => Ok(Some(LogObject::MostPkt2(MostPkt2::read(cursor, &header)?))),
            ObjectType::MostLightLock => Ok(Some(LogObject::MostLightLock(MostLightLock::read(
                cursor, &header,
            )?))),
            ObjectType::MostStatistic => Ok(Some(LogObject::MostStatistic(MostStatistic::read(
                cursor, &header,
            )?))),
            ObjectType::MostHwMode => Ok(Some(LogObject::MostHwMode(MostHwMode::read(
                cursor, &header,
            )?))),
            ObjectType::MostReg => Ok(Some(LogObject::MostReg(MostReg::read(cursor, &header)?))),
            ObjectType::MostGenReg => Ok(Some(LogObject::MostGenReg(MostGenReg::read(
                cursor, &header,
            )?))),
            ObjectType::MostNetState => Ok(Some(LogObject::MostNetState(MostNetState::read(
                cursor, &header,
            )?))),
            ObjectType::MostDataLost => Ok(Some(LogObject::MostDataLost(MostDataLost::read(
                cursor, &header,
            )?))),
            ObjectType::MostTrigger => Ok(Some(LogObject::MostTrigger(MostTrigger::read(
                cursor, &header,
            )?))),
            _ => {
                // Create an Unhandled object for unknown types
                let mut data = vec![0u8; object_data_size];
                cursor.read_exact(&mut data)?;
                Ok(Some(LogObject::Unhandled {
                    object_type: header.object_type as u32,
                    timestamp: header.object_time_stamp,
                    data,
                }))
            }
        }
    }

    /// Parses the actual log objects contained within a (decompressed) LogContainer.
    fn parse_inner_objects(&self, cursor: &mut Cursor<&[u8]>) -> BlfParseResult<Vec<LogObject>> {
        let mut all_objects = Vec::new();
        let mut all_errors = Vec::new();
        let data_len = cursor.get_ref().len();

        while (cursor.position() as usize) < data_len {
            let start_pos = cursor.position();

            // Check if we have enough bytes to read the signature
            if (data_len as u64 - start_pos) < 4 {
                break;
            }

            // Try to read the header, but handle the case where there's no valid object left
            let header = match ObjectHeader::read(cursor).context("BlfParser.ObjectHeader") {
                Ok(header) => header,
                Err(BlfParseError::InvalidContainerMagic) => {
                    // If we can't read a valid header due to magic number, skip one byte and try again
                    cursor.set_position(start_pos + 1);
                    continue;
                }
                Err(BlfParseError::UnexpectedEof) => {
                    // End of file reached, return what we have
                    break;
                }
                Err(e) => {
                    all_errors.push(e);
                    // Try to skip some bytes and continue
                    cursor.set_position(start_pos + 4);
                    continue;
                }
            };

            // LogContainers should not be nested. If they are, we skip them to avoid infinite recursion.
            if header.object_type != ObjectType::LogContainer {
                let object_body_size = (header.object_size as usize)
                    .saturating_sub(header.calculate_header_size() as usize);

                match self.parse_can_object(cursor, &header, object_body_size) {
                    Ok(Some(object)) => {
                        all_objects.push(object);
                    }
                    Ok(None) => {
                        // Object type not handled, skip
                    }
                    Err(BlfParseError::UnexpectedEof) => {
                        // End of file reached while parsing object body
                        all_errors.push(BlfParseError::UnexpectedEof);
                        break;
                    }
                    Err(e) => {
                        all_errors.push(e);
                        // Try to advance and continue
                        let next_pos = start_pos + header.object_size as u64;
                        cursor.set_position(next_pos.min(data_len as u64));
                        continue;
                    }
                }
            } else {
                // For LogContainer objects, we skip them but still need to advance the cursor
                if self.debug {
                    println!("Skipping LogContainer object");
                }
            }

            // 在LogContainer内部，对象已经通过add_padding进行了4字节对齐
            // 所以我们直接使用对象大小，而不是进行额外的对齐
            let next_pos = start_pos + header.object_size as u64;
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
            let aligned_pos = (current_pos + 3) & !3; // Round up to next multiple of 4
            cursor.set_position(aligned_pos.min(data_len as u64));
        }

        // If we have errors but also successfully parsed objects, return the objects
        // Otherwise, if we have no objects but have errors, return the first error
        if !all_objects.is_empty() {
            Ok(all_objects)
        } else if !all_errors.is_empty() {
            Err(all_errors.into_iter().next().unwrap())
        } else {
            Ok(all_objects)
        }
    }

    /// Advances the cursor to the start of the next object, including padding.
    /// This logic is based on `is.seekg(objectSize % 4, std::ios_base::cur);` from the C++ source.
    /// The C++ implementation `objectSize % 4` is a non-standard way to achieve 4-byte alignment
    /// that happens to work in most cases but is conceptually incorrect when `objectSize` is a multiple of 4.
    /// We use a more robust and idiomatic method `(object_size + 3) & !3` to round up to the next
    /// multiple of 4, which correctly handles all cases and clarifies the intent of alignment.
    fn advance_cursor_to_next_object(
        &self,
        cursor: &mut Cursor<&[u8]>,
        start_pos: u64,
        object_size: u32,
    ) {
        let data_len = cursor.get_ref().len() as u64;
        let padded_size = (object_size as u64 + 3) & !3;
        let next_pos = start_pos + padded_size;
        // Clamp to data_len so consumed bytes never exceed file size
        // (avoids StatusBar showing >100%).
        let clamped = next_pos.min(data_len);
        cursor.set_position(clamped);
    }

    #[allow(dead_code)]
    fn parse_unhandled_object(&self) -> Option<LogObject> {
        // The `parse_objects` loop handles advancing the cursor to the next object.
        // For unhandled or deprecated object types, we simply do nothing and return `None`.
        //
        // This includes:
        // - ObjectType::MostPkt: Deprecated and should be skipped.
        // - Any other unknown ObjectType.
        None
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
                base: crate::objects::object_header::ObjectHeaderBase {
                    signature: 0x4A424F4C, // "LOBJ"
                    header_size: 32,
                    header_version: 1,
                    object_size: 48, // header + can_msg_fields + data
                    object_type: ObjectType::CanMessage,
                },
                object_flags: 0,
                client_index: 0,
                object_version: 0,
                object_time_stamp: 1000,
                original_time_stamp: None,
                time_stamp_status: None,
                reserved: 0,
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
                base: crate::objects::object_header::ObjectHeaderBase {
                    signature: 0x4A424F4C, // "LOBJ"
                    header_size: 32,
                    header_version: 1,
                    object_size: 48,
                    object_type: ObjectType::CanMessage,
                },
                object_flags: 0,
                client_index: 0,
                object_version: 0,
                object_time_stamp: 1000,
                original_time_stamp: None,
                time_stamp_status: None,
                reserved: 0,
            },
            channel: 1,
            flags: 0,
            dlc: 8,
            id: 0x111,
            data: [0; 8],
        };
        let can_message2 = CanMessage {
            header: ObjectHeader {
                base: crate::objects::object_header::ObjectHeaderBase {
                    signature: 0x4A424F4C, // "LOBJ"
                    header_size: 32,
                    header_version: 1,
                    object_size: 48,
                    object_type: ObjectType::CanMessage,
                },
                object_flags: 0,
                client_index: 0,
                object_version: 0,
                object_time_stamp: 2000,
                original_time_stamp: None,
                time_stamp_status: None,
                reserved: 0,
            },
            channel: 2,
            flags: 0,
            dlc: 8,
            id: 0x222,
            data: [1; 8],
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
        use crate::ObjectType;
        use std::io::Write;

        let parser = BlfParser::new();

        // Create a LogContainer to wrap the unknown object
        let unknown_header = ObjectHeader {
            base: crate::objects::object_header::ObjectHeaderBase {
                signature: 0x4A424F4C, // "LOBJ"
                header_size: 32,
                header_version: 1,
                object_size: 38, // Does not need to be a multiple of 4
                object_type: ObjectType::Unknown,
            },
            object_flags: 0,
            client_index: 0,
            object_version: 0,
            object_time_stamp: 1000,
            original_time_stamp: None,
            time_stamp_status: None,
            reserved: 0,
        };

        let mut unknown_object_bytes = Vec::new();
        serialize_object_header(&unknown_header, &mut unknown_object_bytes);
        unknown_object_bytes
            .write_all(&vec![
                0;
                (unknown_header.object_size - unknown_header.header_size as u32)
                    as usize
            ])
            .unwrap();
        add_padding(&mut unknown_object_bytes);

        let mut cursor = Cursor::new(&unknown_object_bytes[..]);
        let result = parser.parse_inner_objects(&mut cursor).unwrap();

        // The parser should gracefully skip the unknown object and return an empty list.
        assert!(result.is_empty());
    }
}
