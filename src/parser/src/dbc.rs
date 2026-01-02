use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Signal {
    pub name: String,
    pub start_bit: u32,
    pub signal_size: u32,
    pub byte_order: u8, // 0 = BigEndian/Motorola, 1 = LittleEndian/Intel
    pub value_type: char, // '+' = unsigned, '-' = signed
    pub factor: f64,
    pub offset: f64,
    pub min: f64,
    pub max: f64,
    pub unit: String,
    pub receivers: Vec<String>,
    pub comment: Option<String>,
}

impl Signal {
    pub fn decode(&self, data: &[u8]) -> f64 {
        let mut raw_value: u64 = 0;
        
        if self.byte_order == 1 { // Intel / Little Endian
            for i in 0..self.signal_size {
                let bit_pos = self.start_bit + i;
                let byte_idx = (bit_pos / 8) as usize;
                let bit_in_byte = bit_pos % 8;
                
                if byte_idx < data.len() {
                    let bit = (data[byte_idx] >> bit_in_byte) & 1;
                    raw_value |= (bit as u64) << i;
                }
            }
        } else { // Motorola / Big Endian
             // In Vector DBC, Motorola start_bit is the MSB of the signal.
             // We need to decode backwards.
             let mut current_bit = self.start_bit as i32;
             for i in 0..self.signal_size {
                 let byte_idx = (current_bit / 8) as usize;
                 let bit_in_byte = current_bit % 8;
                 
                 if byte_idx < data.len() {
                     let bit = (data[byte_idx] >> bit_in_byte) & 1;
                     raw_value |= (bit as u64) << (self.signal_size - 1 - i);
                 }
                 
                 // Move to next bit in Motorola order
                 if current_bit % 8 == 0 {
                     current_bit += 15;
                 } else {
                     current_bit -= 1;
                 }
             }
        }
        
        // Handle signed
        let value = if self.value_type == '-' {
            let sign_bit = 1u64 << (self.signal_size - 1);
            if (raw_value & sign_bit) != 0 {
                // Sign extend
                let mask = (1u64 << self.signal_size) - 1;
                let extended = raw_value | !mask;
                extended as i64 as f64
            } else {
                raw_value as f64
            }
        } else {
            raw_value as f64
        };
        
        value * self.factor + self.offset
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub id: u32,
    pub name: String,
    pub dlc: u8,
    pub transmitter: String,

    pub signals: HashMap<String, Signal>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DbcDatabase {
    pub messages: HashMap<u32, Message>,

    pub version: String,
    pub description: Option<String>,
}

pub struct DbcParser;

impl DbcParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, content: &str) -> Result<DbcDatabase, String> {

        let mut database = DbcDatabase {
            messages: HashMap::new(),
            version: "".to_string(),
            description: None,
        };

        let mut current_message_id: Option<u32> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("VERSION") {
                if let Some(version) = line.split('"').nth(1) {
                    database.version = version.to_string();
                }
            } else if line.starts_with("BO_") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    if let Ok(id) = parts[1].parse::<u32>() {
                        let name = parts[2].trim_end_matches(':').to_string();
                        let dlc = parts[3].parse::<u8>().unwrap_or(0);
                        let transmitter = parts[4].to_string();


                        let message = Message {
                            id,
                            name,
                            dlc,
                            transmitter,
                            signals: HashMap::new(),
                            comment: None,
                        };
                        database.messages.insert(id, message);
                        current_message_id = Some(id);
                    }
                }
            } else if line.starts_with("SG_") {
                if let Some(msg_id) = current_message_id {
                    // Format: SG_ Name : StartBit|Size@ByteOrderValueType (Factor,Offset) [Min|Max] "Unit" Receivers
                    // Example: SG_ SignalName : 0|8@1+ (1,0) [0|255] "unit" Vector__XXX
                    
                    // Simple parsing using splits (robust parsing would use regex or a parser combinator)
                    // Part 1: "SG_" "SignalName" ":" ...
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() < 4 { continue; } // Basic check

                    let name = parts[1].to_string();
                    
                    // Remainder string for complex parsing
                    if let Some(rest_idx) = line.find(':') {
                        let rest = &line[rest_idx+1..].trim();
                        // 0|8@1+ (1,0) [0|255] "unit" Vector__XXX
                        
                        // Split by whitespace to get chunks
                        // Chunk 0: 0|8@1+
                        // Chunk 1: (1,0)
                        // Chunk 2: [0|255]
                        // Chunk 3: "unit"
                        // Chunk 4: Receivers
                        
                        // We need to be careful about spaces inside quotes/brackets, but let's assume standard format for now.
                        let chunks: Vec<&str> = rest.split_whitespace().collect();
                        if chunks.len() >= 1 {
                             // Parse bit definition: 0|8@1+
                             let bit_def = chunks[0];
                             let pipe_parts: Vec<&str> = bit_def.split('|').collect();
                             if pipe_parts.len() == 2 {
                                 let start_bit = pipe_parts[0].parse::<u32>().unwrap_or(0);
                                 let size_type_split: Vec<&str> = pipe_parts[1].split('@').collect();
                                 if size_type_split.len() == 2 {
                                     let size = size_type_split[0].parse::<u32>().unwrap_or(0);
                                     let order_type = size_type_split[1]; // 1+ or 0-
                                     let byte_order = if order_type.contains('1') { 1 } else { 0 }; 
                                     let value_type = if order_type.contains('-') { '-' } else { '+' };
                                     
                                     // Factors (1,0)
                                     let mut factor = 1.0;
                                     let mut offset = 0.0;
                                     if chunks.len() >= 2 {
                                         let fact_str = chunks[1].trim_matches(|c| c == '(' || c == ')');
                                         let fact_parts: Vec<&str> = fact_str.split(',').collect();
                                         if fact_parts.len() == 2 {
                                             factor = fact_parts[0].parse::<f64>().unwrap_or(1.0);
                                             offset = fact_parts[1].parse::<f64>().unwrap_or(0.0);
                                         }
                                     }

                                     // Min/Max [0|255]
                                     let mut min = 0.0;
                                     let mut max = 0.0;
                                     if chunks.len() >= 3 {
                                         let minmax_str = chunks[2].trim_matches(|c| c == '[' || c == ']');
                                         let minmax_parts: Vec<&str> = minmax_str.split('|').collect();
                                          if minmax_parts.len() == 2 {
                                             min = minmax_parts[0].parse::<f64>().unwrap_or(0.0);
                                             max = minmax_parts[1].parse::<f64>().unwrap_or(0.0);
                                         }
                                     }
                                     
                                     // Unit "unit"
                                     let mut unit = "".to_string();
                                     if chunks.len() >= 4 {
                                         unit = chunks[3].trim_matches('"').to_string();
                                     }

                                     // Receivers
                                     let mut receivers = Vec::new();
                                     if chunks.len() >= 5 {
                                         receivers.push(chunks[4].to_string()); // Simplification: just one or first receiver chunk
                                     }


                                     let signal = Signal {
                                         name: name.clone(),
                                         start_bit,
                                         signal_size: size,
                                         byte_order,
                                         value_type,
                                         factor,
                                         offset,
                                         min,
                                         max,
                                         unit,
                                         receivers,
                                         comment: None,
                                     };
                                     
                                     if let Some(msg) = database.messages.get_mut(&msg_id) {
                                         msg.signals.insert(name, signal);
                                     }
                                 }
                             }
                        }
                    }
                }

            } else if line.starts_with("CM_") {
                // CM_ "Global Comment";
                // CM_ BO_ 123 "Message Comment";
                // CM_ SG_ 123 SigName "Signal Comment";
                
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                   if parts[1].starts_with('"') {
                        // Global comment
                        // CM_ "Description"
                        let desc = line.trim_start_matches("CM_").trim();
                        // Remove leading/trailing quotes and optional semicolon
                        let desc = desc.trim_matches(|c| c == '"' || c == ';' || c == ' ');
                        database.description = Some(desc.to_string());
                   } else if parts[1] == "BO_" && parts.len() >= 4 {
                       // CM_ BO_ 123 "Comment"
                       if let Ok(id) = parts[2].parse::<u32>() {
                           if let Some(msg) = database.messages.get_mut(&id) {
                               if let Some(first_quote) = line.find('"') {
                                   let comment = line[first_quote..].trim_matches(|c| c == '"' || c == ';' || c == ' ').to_string();
                                   msg.comment = Some(comment);
                               }
                           }
                       }
                   } else if parts[1] == "SG_" && parts.len() >= 5 {
                       // CM_ SG_ 123 SigName "Comment"
                       if let Ok(id) = parts[2].parse::<u32>() {
                           let sig_name = parts[3];
                           if let Some(msg) = database.messages.get_mut(&id) {
                               if let Some(sig) = msg.signals.get_mut(sig_name) {
                                   if let Some(first_quote) = line.find('"') {
                                       let comment = line[first_quote..].trim_matches(|c| c == '"' || c == ';' || c == ' ').to_string();
                                       sig.comment = Some(comment);
                                   }
                               }
                           }
                       }
                   }
                }
            }
        }

        Ok(database)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_dbc() {
        let dbc_content = r#"
VERSION ""


NS_ : 
	NS_DESC_
	CM_
	BA_DEF_
	BA_
	VAL_
	CAT_DEF_
	CAT_
	FILTER
	BA_DEF_DEF_
	EV_DATA_
	ENVVAR_DATA_
	SGTYPE_
	SGTYPE_VAL_
	BA_DEF_SGTYPE_
	BA_SGTYPE_
	SIG_TYPE_REF_
	VAL_TABLE_
	SIG_GROUP_
	SIG_VALTYPE_
	SIGTYPE_VALTYPE_
	BO_TX_BU_
	BA_DEF_REL_
	BA_REL_
	BA_DEF_DEF_REL_
	BU_SG_REL_
	BU_EV_REL_
	BU_BO_REL_
	SG_MUL_VAL_

BS_:

BU_: Vector__XXX


BO_ 12345 TestMessage: 8 Vector__XXX
 SG_ TestSignal1 : 0|8@1+ (1,0) [0|255] "unit1" Vector__XXX
 SG_ TestSignal2 : 8|16@1- (0.5,10) [-100|100] "unit2" Vector__XXX

CM_ "Global Description";
CM_ BO_ 12345 "Message Comment";
CM_ SG_ 12345 TestSignal1 "Signal Comment";
"#;

        let parser = DbcParser::new();
        let db = parser.parse(dbc_content).unwrap();

        assert_eq!(db.description, Some("Global Description".to_string()));
        
        let msg = db.messages.get(&12345).unwrap();
        assert_eq!(msg.name, "TestMessage");
        assert_eq!(msg.dlc, 8);
        assert_eq!(msg.comment, Some("Message Comment".to_string()));
        assert_eq!(msg.signals.len(), 2);

        let sig1 = msg.signals.get("TestSignal1").unwrap();
        assert_eq!(sig1.start_bit, 0);
        assert_eq!(sig1.signal_size, 8);
        assert_eq!(sig1.byte_order, 1);
        assert_eq!(sig1.value_type, '+');
        assert_eq!(sig1.factor, 1.0);
        assert_eq!(sig1.offset, 0.0);
        assert_eq!(sig1.min, 0.0);
        assert_eq!(sig1.max, 255.0);
        assert_eq!(sig1.unit, "unit1");
        assert_eq!(sig1.comment, Some("Signal Comment".to_string()));

        let sig2 = msg.signals.get("TestSignal2").unwrap();
        assert_eq!(sig2.start_bit, 8);
        assert_eq!(sig2.signal_size, 16);
        assert_eq!(sig2.byte_order, 1);
        assert_eq!(sig2.value_type, '-');
        assert_eq!(sig2.factor, 0.5);
        assert_eq!(sig2.offset, 10.0);
        assert_eq!(sig2.min, -100.0);
        assert_eq!(sig2.max, 100.0);
        assert_eq!(sig2.unit, "unit2");
    }
}
