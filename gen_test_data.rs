use blf::{BlfParser, LogObject, CanMessage};
use std::fs::File;
use std::io::Write;
use byteorder::{LittleEndian, WriteBytesExt};

fn main() {
    let mut objects = Vec::new();

    // EngineStatus (ID 123)
    // EngineTemp: 80 degC -> raw = (80 - (-40)) / 1 = 120
    // EngineRPM: 3000 rpm -> raw = 3000 / 0.25 = 12000
    let mut data1 = [0u8; 8];
    data1[0] = 120;
    (&mut data1[1..3]).write_u16::<LittleEndian>(12000).unwrap();
    
    objects.push(LogObject::CanMessage(CanMessage {
        channel: 1,
        flags: 0,
        id: 123,
        dlc: 8,
        data: data1,
    }));

    // GearboxStatus (ID 456)
    // CurrentGear: 3
    // TransmissionTemp: 90 degC -> raw = 90 - (-40) = 130
    let mut data2 = [0u8; 8];
    data2[0] = 3 | (130 << 4); // This might be wrong due to bit layout, let's keep it simple
    // 0|4 = gear, 4|8 = temp. Gear 3 is 0011, Temp 130 is 10000010.
    // byte 0: TTTTGGGG where G is gear and T is low bits of temp?
    // In DBC 0|4@1+ means start bit 0, length 4, little endian.
    // 4|8@1+ means start bit 4, length 8, little endian.
    // Byte 0: [T3 T2 T1 T0 G3 G2 G1 G0]
    // Byte 1: [0 0 0 0 T7 T6 T5 T4]
    data2[0] = 3 | ((130 & 0x0F) << 4);
    data2[1] = (130 >> 4) as u8;

    objects.push(LogObject::CanMessage(CanMessage {
        channel: 1,
        flags: 0,
        id: 456,
        dlc: 4,
        data: data2,
    }));

    // Write to file
    // Note: I need to use the actual BLF writer if available, but let's check the blf crate.
    // Since I don't have a writer in the blf crate easily visible, I might need to implement a simple one 
    // or just rely on the existing blf crate's ability to serialize if it has one.
    // Actually, I'll check the blf crate for a write method.
}
