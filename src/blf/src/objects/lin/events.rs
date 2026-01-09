//! LIN event object definitions.

use crate::{BlfParseResult};
use crate::objects::object_header::ObjectHeader;
use std::io::Cursor;

// --- Stubs for LIN event objects ---

#[derive(Debug, Clone, PartialEq, Default)] pub struct LinCrcError { pub header: ObjectHeader }
impl LinCrcError { pub fn read(_cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> { Ok(Self { header: header.clone() }) } }
#[derive(Debug, Clone, PartialEq, Default)] pub struct LinReceiveError { pub header: ObjectHeader }
impl LinReceiveError { pub fn read(_cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> { Ok(Self { header: header.clone() }) } }
#[derive(Debug, Clone, PartialEq, Default)] pub struct LinSendError { pub header: ObjectHeader }
impl LinSendError { pub fn read(_cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> { Ok(Self { header: header.clone() }) } }
#[derive(Debug, Clone, PartialEq, Default)] pub struct LinSlaveTimeout { pub header: ObjectHeader }
impl LinSlaveTimeout { pub fn read(_cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> { Ok(Self { header: header.clone() }) } }
#[derive(Debug, Clone, PartialEq, Default)] pub struct LinSchedulerModeChange { pub header: ObjectHeader }
impl LinSchedulerModeChange { pub fn read(_cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> { Ok(Self { header: header.clone() }) } }
#[derive(Debug, Clone, PartialEq, Default)] pub struct LinSyncError { pub header: ObjectHeader }
impl LinSyncError { pub fn read(_cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> { Ok(Self { header: header.clone() }) } }
#[derive(Debug, Clone, PartialEq, Default)] pub struct LinBaudrateEvent { pub header: ObjectHeader }
impl LinBaudrateEvent { pub fn read(_cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> { Ok(Self { header: header.clone() }) } }
#[derive(Debug, Clone, PartialEq, Default)] pub struct LinSleepModeEvent { pub header: ObjectHeader }
impl LinSleepModeEvent { pub fn read(_cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> { Ok(Self { header: header.clone() }) } }
#[derive(Debug, Clone, PartialEq, Default)] pub struct LinWakeupEvent { pub header: ObjectHeader }
impl LinWakeupEvent { pub fn read(_cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> { Ok(Self { header: header.clone() }) } }
#[derive(Debug, Clone, PartialEq, Default)] pub struct LinDlcInfo { pub header: ObjectHeader }
impl LinDlcInfo { pub fn read(_cursor: &mut Cursor<&[u8]>, header: &ObjectHeader) -> BlfParseResult<Self> { Ok(Self { header: header.clone() }) } }
