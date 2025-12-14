use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LdfSignal {
    pub name: String,
    pub size: u32,
    pub initial_value: u32,
    pub published_by: String,

    pub subscribed_by: Vec<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LdfSignalMapping {
    pub offset: u32,
    pub signal_name: String,
}

#[derive(Debug, Clone)]
pub struct LdfFrame {
    pub name: String,
    pub id: u32,
    pub published_by: String,
    pub size: u32,

    pub signals: Vec<LdfSignalMapping>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LdfDatabase {
    pub version: String,
    pub signals: HashMap<String, LdfSignal>,
    pub frames: HashMap<String, LdfFrame>,
}

pub struct LdfParser;

impl LdfParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, content: &str) -> Result<LdfDatabase, String> {
        let mut database = LdfDatabase {
            version: "".to_string(),
            signals: HashMap::new(),
            frames: HashMap::new(),
        };

        let mut section = "";
        
        // Simple line parser. Real LDF parsing is token-based and sensitive to braces.
        // We will approximate by looking for "Block {" lines.
        
        let lines: Vec<&str> = content.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
        let mut i = 0;
        
        while i < lines.len() {
            let raw_line = lines[i];
            // Split comment
            let (line, comment) = if let Some(idx) = raw_line.find("//") {
                (raw_line[..idx].trim(), Some(raw_line[idx+2..].trim().to_string()))
            } else {
                (raw_line, None)
            };
            
            if line.is_empty() {
                i += 1;
                continue;
            }
            
            if line.starts_with("LIN_description_file") {
                 // LIN_description_file = "2.1";
                 if let Some(start) = line.find('"') {
                     if let Some(end) = line[start+1..].find('"') {
                         database.version = line[start+1..start+1+end].to_string();
                     }
                 }
            } else if line.starts_with("Signals {") {
                section = "Signals";
            } else if line.starts_with("Frames {") {
                section = "Frames";
            } else if line == "}" {
                section = "";
            } else {
                match section {
                    "Signals" => {
                        // SigName: Size, InitValue, Publisher, Subscriber1, Subscriber2;
                        // Example: SysSt: 8, 0, BCM, IPC;
                         let clean_line = line.trim_end_matches(';');
                         if let Some(colon_idx) = clean_line.find(':') {
                             let name = clean_line[..colon_idx].trim().to_string();
                             let rest = &clean_line[colon_idx+1..];
                             let parts: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                             if parts.len() >= 3 {
                                 let size = parts[0].parse::<u32>().unwrap_or(0);
                                 let initial_value = parts[1].parse::<u32>().unwrap_or(0);
                                 let published_by = parts[2].to_string();
                                 let mut subscribed_by = Vec::new();
                                 for sub in parts.iter().skip(3) {
                                     subscribed_by.push(sub.to_string());
                                 }
                                
                                let signal = LdfSignal {
                                    name: name.clone(),
                                    size,
                                    initial_value,
                                    published_by,
                                    subscribed_by,
                                    comment: comment.clone(),
                                };
                                database.signals.insert(name, signal);
                            }
                        }
                    }
                    "Frames" => {
                        // FrameName: Id, Publisher, Size {
                        //    SigName, Offset;
                        // }
                        
                        // Note: Frames section has nested braces. 
                        // My simple line logic might break if we don't handle the internal block.
                        // Let's assume the syntax:
                        // FrameName: Id, Publisher, Size {
                        //    Mapping...
                        // }
                        
                        if line.ends_with("{") {
                            // Start of frame definition
                            // Example: IPC_Frame: 0x10, IPC, 4 {
                            let clean_line = line.trim_end_matches('{').trim();
                            if let Some(colon_idx) = clean_line.find(':') {
                                let name = clean_line[..colon_idx].trim().to_string();
                                let rest = &clean_line[colon_idx+1..];
                                let parts: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                                if parts.len() >= 3 {
                                    let id_str = parts[0];
                                    let id = if id_str.starts_with("0x") {
                                        u32::from_str_radix(&id_str[2..], 16).unwrap_or(0)
                                    } else {
                                        id_str.parse::<u32>().unwrap_or(0)
                                    };
                                    let published_by = parts[1].to_string();
                                    let size = parts[2].parse::<u32>().unwrap_or(0);
                                    
                                    let mut signals_map = Vec::new();
                                    
                                    // Read lines until "}"
                                    i += 1;
                                    while i < lines.len() {

                                        let raw_inner = lines[i];
                                         let (inner_line, _) = if let Some(idx) = raw_inner.find("//") {
                                            (raw_inner[..idx].trim(), Some(raw_inner[idx+2..].trim().to_string()))
                                        } else {
                                            (raw_inner, None)
                                        };
                                        
                                        if inner_line == "}" {
                                            break;
                                        }
                                        if inner_line.is_empty() {
                                            i += 1;
                                            continue;
                                        }
                                        // SigName, Offset;
                                        let inner_clean = inner_line.trim_end_matches(';');
                                        let inner_parts: Vec<&str> = inner_clean.split(',').map(|s| s.trim()).collect();
                                        if inner_parts.len() == 2 {
                                            let sig_name = inner_parts[0].to_string();
                                            let offset = inner_parts[1].parse::<u32>().unwrap_or(0);
                                            signals_map.push(LdfSignalMapping {
                                                offset,
                                                signal_name: sig_name,
                                            });
                                        }
                                        i += 1;
                                    }
                                    
                                    let frame = LdfFrame {
                                        name: name.clone(),
                                        id,
                                        published_by,
                                        size,

                                        signals: signals_map,
                                        comment: comment.clone(),
                                    };
                                    database.frames.insert(name, frame);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            
            i += 1;
        }

        Ok(database)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_ldf() {
        let ldf_content = r#"
LIN_description_file = "2.1";

Signals {
    SysSt: 8, 0, BCM, IPC; // Signal Comment
    VehSpd: 16, 65535, IPC, BCM;
}

Frames {
    BCM_St: 0x10, BCM, 2 { // Frame Comment
        SysSt, 0;
    }
    IPC_Spd: 0x11, IPC, 4 {
        VehSpd, 8;
    }
}
"#;

        let parser = LdfParser::new();
        let db = parser.parse(ldf_content).unwrap();
        
        assert_eq!(db.version, "2.1");
        assert_eq!(db.signals.len(), 2);
        assert_eq!(db.frames.len(), 2);
        
        let sig1 = db.signals.get("SysSt").unwrap();
        assert_eq!(sig1.size, 8);
        assert_eq!(sig1.initial_value, 0);
        assert_eq!(sig1.published_by, "BCM");
        assert_eq!(sig1.subscribed_by, vec!["IPC"]);
        assert_eq!(sig1.comment, Some("Signal Comment".to_string()));
        
        let frame1 = db.frames.get("BCM_St").unwrap();
        assert_eq!(frame1.id, 0x10);
        assert_eq!(frame1.published_by, "BCM");
        assert_eq!(frame1.size, 2);
        assert_eq!(frame1.signals.len(), 1);
        assert_eq!(frame1.signals[0].signal_name, "SysSt");
        assert_eq!(frame1.signals[0].offset, 0);
        assert_eq!(frame1.comment, Some("Frame Comment".to_string()));
    }
}
