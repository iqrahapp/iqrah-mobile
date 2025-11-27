use super::error::NodeIdError;
use super::models::{KnowledgeAxis, NodeType};

pub type Result<T> = std::result::Result<T, NodeIdError>;

// ============================================================================
// CONSTANTS FOR I64 ENCODING
// ============================================================================

const TYPE_SHIFT: u8 = 56;
const TYPE_MASK: i64 = 0xFF << TYPE_SHIFT;

const TYPE_CHAPTER: i64 = 1;
const TYPE_VERSE: i64 = 2;
const TYPE_WORD: i64 = 3;
const TYPE_WORD_INSTANCE: i64 = 4;
const TYPE_KNOWLEDGE: i64 = 5;

// ============================================================================
// BUILDER FUNCTIONS (Infallible)
// ============================================================================

/// Build a chapter node ID: "CHAPTER:1"
pub fn chapter(num: u8) -> String {
    debug_assert!((1..=114).contains(&num), "Chapter must be 1-114");
    format!("CHAPTER:{}", num)
}

/// Build a verse node ID: "VERSE:1:1"
pub fn verse(chapter: u8, verse: u16) -> String {
    debug_assert!((1..=114).contains(&chapter), "Chapter must be 1-114");
    debug_assert!(verse >= 1, "Verse must be >= 1");
    format!("VERSE:{}:{}", chapter, verse)
}

/// Build a word node ID: "WORD:123"
pub fn word(word_id: i64) -> String {
    debug_assert!(word_id > 0, "Word ID must be positive");
    format!("WORD:{}", word_id)
}

/// Build a word instance node ID: "WORD_INSTANCE:1:1:3"
pub fn word_instance(chapter: u8, verse: u16, position: u8) -> String {
    debug_assert!((1..=114).contains(&chapter), "Chapter must be 1-114");
    debug_assert!(verse >= 1, "Verse must be >= 1");
    debug_assert!(position >= 1, "Position must be >= 1");
    format!("WORD_INSTANCE:{}:{}:{}", chapter, verse, position)
}

/// Build a knowledge node ID: "VERSE:1:1:memorization"
pub fn knowledge(base_id: &str, axis: KnowledgeAxis) -> String {
    format!("{}:{}", base_id, axis.as_ref())
}

// ============================================================================
// ENCODER FUNCTIONS (I64)
// ============================================================================

pub fn encode_chapter(num: u8) -> i64 {
    (TYPE_CHAPTER << TYPE_SHIFT) | (num as i64)
}

pub fn encode_verse(chapter: u8, verse: u16) -> i64 {
    (TYPE_VERSE << TYPE_SHIFT) | ((chapter as i64) << 16) | (verse as i64)
}

pub fn encode_word(word_id: i64) -> i64 {
    (TYPE_WORD << TYPE_SHIFT) | word_id
}

pub fn encode_word_instance(chapter: u8, verse: u16, position: u8) -> i64 {
    (TYPE_WORD_INSTANCE << TYPE_SHIFT)
        | ((chapter as i64) << 32)
        | ((verse as i64) << 16)
        | (position as i64)
}

pub fn encode_knowledge(base_id: i64, axis: KnowledgeAxis) -> i64 {
    let axis_id = match axis {
        KnowledgeAxis::Memorization => 1,
        KnowledgeAxis::Translation => 2,
        KnowledgeAxis::Tafsir => 3,
        KnowledgeAxis::Tajweed => 4,
        KnowledgeAxis::ContextualMemorization => 5,
        KnowledgeAxis::Meaning => 6,
    };

    // Extract the base type from the high bits (56-63)
    let base_type = (base_id >> TYPE_SHIFT) & 0xFF;

    // We only have 4 bits for base type in the new scheme (bits 52-55),
    // so ensure it fits. Current types are small integers (1-5).
    debug_assert!(base_type <= 0xF, "Base type must fit in 4 bits");

    // Layout:
    // Bits 56-63 (8): TYPE_KNOWLEDGE
    // Bits 52-55 (4): Base Type
    // Bits 48-51 (4): Knowledge Axis
    // Bits 0-47 (48): Base Payload (base_id without type prefix)
    (TYPE_KNOWLEDGE << TYPE_SHIFT)
        | (base_type << 52)
        | (axis_id << 48)
        | (base_id & 0x0000FFFFFFFFFFFF)
}

// ============================================================================
// PARSER FUNCTIONS (Fallible)
// ============================================================================

/// Parse a chapter ID: "CHAPTER:1" -> 1
pub fn parse_chapter(id: &str) -> Result<u8> {
    let parts: Vec<&str> = id.split(':').collect();

    if parts.len() != 2 || parts[0] != "CHAPTER" {
        return Err(NodeIdError::InvalidFormat(id.to_string()));
    }

    let num = parts[1]
        .parse::<u8>()
        .map_err(|_| NodeIdError::Malformed(id.to_string()))?;

    if !(1..=114).contains(&num) {
        return Err(NodeIdError::InvalidChapter(num));
    }

    Ok(num)
}

/// Parse a verse ID: "VERSE:1:1" -> (1, 1)
pub fn parse_verse(id: &str) -> Result<(u8, u16)> {
    let parts: Vec<&str> = id.split(':').collect();

    // Only accept prefixed format "VERSE:1:1"
    if parts.len() != 3 || parts[0] != "VERSE" {
        return Err(NodeIdError::InvalidFormat(id.to_string()));
    }

    let chapter_str = parts[1];
    let verse_str = parts[2];

    let chapter = chapter_str
        .parse::<u8>()
        .map_err(|_| NodeIdError::Malformed(id.to_string()))?;

    let verse = verse_str
        .parse::<u16>()
        .map_err(|_| NodeIdError::Malformed(id.to_string()))?;

    if !(1..=114).contains(&chapter) {
        return Err(NodeIdError::InvalidChapter(chapter));
    }

    if verse < 1 {
        return Err(NodeIdError::InvalidVerse(verse));
    }

    Ok((chapter, verse))
}

/// Parse word ID: "WORD:123" -> 123
pub fn parse_word(id: &str) -> Result<i64> {
    let parts: Vec<&str> = id.split(':').collect();

    if parts.len() != 2 || parts[0] != "WORD" {
        return Err(NodeIdError::InvalidFormat(id.to_string()));
    }

    parts[1]
        .parse::<i64>()
        .map_err(|_| NodeIdError::Malformed(id.to_string()))
}

/// Parse word instance: "WORD_INSTANCE:1:1:3" -> (1, 1, 3)
pub fn parse_word_instance(id: &str) -> Result<(u8, u16, u8)> {
    let parts: Vec<&str> = id.split(':').collect();

    if parts.len() != 4 || parts[0] != "WORD_INSTANCE" {
        return Err(NodeIdError::InvalidFormat(id.to_string()));
    }

    let chapter = parts[1]
        .parse::<u8>()
        .map_err(|_| NodeIdError::Malformed(id.to_string()))?;

    let verse = parts[2]
        .parse::<u16>()
        .map_err(|_| NodeIdError::Malformed(id.to_string()))?;

    let position = parts[3]
        .parse::<u8>()
        .map_err(|_| NodeIdError::Malformed(id.to_string()))?;

    if !(1..=114).contains(&chapter) {
        return Err(NodeIdError::InvalidChapter(chapter));
    }

    Ok((chapter, verse, position))
}

/// Decode a knowledge node ID (i64) into its base ID and axis components.
/// 
/// Example: decode_knowledge_id(encoded_id) -> Some((base_id, KnowledgeAxis::Memorization))
pub fn decode_knowledge_id(id: i64) -> Option<(i64, KnowledgeAxis)> {
    if decode_type(id) != Some(NodeType::Knowledge) {
        return None;
    }

    // Extract components
    // Bits 52-55: Base Type
    let base_type = (id >> 52) & 0x0F;
    // Bits 48-51: Knowledge Axis
    let axis_id = (id >> 48) & 0x0F;
    // Bits 0-47: Payload
    let payload = id & 0x0000FFFFFFFFFFFF;

    // Reconstruct base_id
    // Original base_id had type in bits 56-63
    let base_id = (base_type << 56) | payload;

    // Map axis_id back to enum
    let axis = match axis_id {
        1 => KnowledgeAxis::Memorization,
        2 => KnowledgeAxis::Translation,
        3 => KnowledgeAxis::Tafsir,
        4 => KnowledgeAxis::Tajweed,
        5 => KnowledgeAxis::ContextualMemorization,
        6 => KnowledgeAxis::Meaning,
        _ => return None,
    };

    Some((base_id, axis))
}

pub fn parse_knowledge(id: &str) -> Result<(String, KnowledgeAxis)> {
    let parts: Vec<&str> = id.split(':').collect();

    // Knowledge nodes have at least 3 parts: prefix:num:axis or prefix:n1:n2:axis
    if parts.len() < 3 {
        return Err(NodeIdError::InvalidFormat(id.to_string()));
    }

    // Last part is the axis
    let axis_str = parts.last().unwrap();
    let axis = KnowledgeAxis::parse(axis_str)
        .map_err(|_| NodeIdError::InvalidAxis(axis_str.to_string()))?;

    // Everything before the last part is the base ID
    let base_id = parts[..parts.len() - 1].join(":");

    Ok((base_id, axis))
}

/// Detect node type from ID string
pub fn node_type(id: &str) -> Result<NodeType> {
    let parts: Vec<&str> = id.split(':').collect();

    if parts.is_empty() {
        return Err(NodeIdError::InvalidFormat(id.to_string()));
    }

    // Check if it's a knowledge node (ends with axis)
    if let Some(last) = parts.last() {
        if KnowledgeAxis::parse(last).is_ok() {
            return Ok(NodeType::Knowledge);
        }
    }

    // Check prefix
    match parts[0] {
        "CHAPTER" => Ok(NodeType::Chapter),
        "VERSE" => Ok(NodeType::Verse),
        "WORD" => Ok(NodeType::Word),
        "WORD_INSTANCE" => Ok(NodeType::WordInstance),
        _ => Err(NodeIdError::InvalidPrefix(parts[0].to_string())),
    }
}

// ============================================================================
// DECODER FUNCTIONS (I64)
// ============================================================================

pub fn decode_type(id: i64) -> Option<NodeType> {
    let type_id = (id & TYPE_MASK) >> TYPE_SHIFT;
    match type_id {
        TYPE_CHAPTER => Some(NodeType::Chapter),
        TYPE_VERSE => Some(NodeType::Verse),
        TYPE_WORD => Some(NodeType::Word),
        TYPE_WORD_INSTANCE => Some(NodeType::WordInstance),
        TYPE_KNOWLEDGE => Some(NodeType::Knowledge),
        _ => None,
    }
}

pub fn decode_chapter(id: i64) -> Option<u8> {
    if decode_type(id) != Some(NodeType::Chapter) {
        return None;
    }
    Some((id & 0xFF) as u8)
}

pub fn decode_verse(id: i64) -> Option<(u8, u16)> {
    if decode_type(id) != Some(NodeType::Verse) {
        return None;
    }
    let chapter = ((id >> 16) & 0xFF) as u8;
    let verse = (id & 0xFFFF) as u16;
    Some((chapter, verse))
}

pub fn decode_word(id: i64) -> Option<i64> {
    if decode_type(id) != Some(NodeType::Word) {
        return None;
    }
    Some(id & !TYPE_MASK)
}

pub fn decode_word_instance(id: i64) -> Option<(u8, u16, u8)> {
    if decode_type(id) != Some(NodeType::WordInstance) {
        return None;
    }
    let chapter = ((id >> 32) & 0xFF) as u8;
    let verse = ((id >> 16) & 0xFFFF) as u16;
    let position = (id & 0xFF) as u8;
    Some((chapter, verse, position))
}

// ============================================================================
// CONVERSION HELPERS
// ============================================================================

pub fn to_ukey(id: i64) -> Option<String> {
    match decode_type(id)? {
        NodeType::Chapter => {
            let num = decode_chapter(id)?;
            Some(chapter(num))
        }
        NodeType::Verse => {
            let (ch, v) = decode_verse(id)?;
            Some(verse(ch, v))
        }
        NodeType::Word => {
            let wid = decode_word(id)?;
            Some(word(wid))
        }
        NodeType::WordInstance => {
            let (ch, v, pos) = decode_word_instance(id)?;
            Some(word_instance(ch, v, pos))
        }
        NodeType::Knowledge => {
            let (base_id, axis) = decode_knowledge_id(id)?;
            let base_ukey = to_ukey(base_id)?;
            Some(knowledge(&base_ukey, axis))
        }
        _ => None,
    }
}

pub fn from_ukey(ukey: &str) -> Option<i64> {
    match node_type(ukey).ok()? {
        NodeType::Chapter => {
            let num = parse_chapter(ukey).ok()?;
            Some(encode_chapter(num))
        }
        NodeType::Verse => {
            let (ch, v) = parse_verse(ukey).ok()?;
            Some(encode_verse(ch, v))
        }
        NodeType::Word => {
            let wid = parse_word(ukey).ok()?;
            Some(encode_word(wid))
        }
        NodeType::WordInstance => {
            let (ch, v, pos) = parse_word_instance(ukey).ok()?;
            Some(encode_word_instance(ch, v, pos))
        }
        NodeType::Knowledge => {
            let (base_ukey, axis) = parse_knowledge(ukey).ok()?;
            let base_id = from_ukey(&base_ukey)?;
            Some(encode_knowledge(base_id, axis))
        }
        _ => None,
    }
}

// ============================================================================
// VALIDATION
// ============================================================================

/// Validate a node ID string
pub fn validate(id: &str) -> Result<()> {
    match node_type(id)? {
        NodeType::Chapter => parse_chapter(id).map(|_| ()),
        NodeType::Verse => parse_verse(id).map(|_| ()),
        NodeType::Word => parse_word(id).map(|_| ()),
        NodeType::WordInstance => parse_word_instance(id).map(|_| ()),
        NodeType::Knowledge => parse_knowledge(id).map(|_| ()),
        _ => Err(NodeIdError::InvalidPrefix("Unknown".to_string())),
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Builder tests
    #[test]
    fn test_build_chapter() {
        assert_eq!(chapter(1), "CHAPTER:1");
        assert_eq!(chapter(114), "CHAPTER:114");
    }

    #[test]
    fn test_build_verse() {
        assert_eq!(verse(1, 1), "VERSE:1:1");
        assert_eq!(verse(2, 286), "VERSE:2:286");
    }

    #[test]
    fn test_build_word() {
        assert_eq!(word(123), "WORD:123");
    }

    #[test]
    fn test_build_word_instance() {
        assert_eq!(word_instance(1, 1, 3), "WORD_INSTANCE:1:1:3");
    }

    #[test]
    fn test_build_knowledge() {
        assert_eq!(
            knowledge("VERSE:1:1", KnowledgeAxis::Memorization),
            "VERSE:1:1:memorization"
        );
    }

    // Encoder/Decoder tests
    #[test]
    fn test_encode_decode_chapter() {
        let id = encode_chapter(114);
        assert_eq!(decode_type(id), Some(NodeType::Chapter));
        assert_eq!(decode_chapter(id), Some(114));
    }

    #[test]
    fn test_encode_decode_verse() {
        let id = encode_verse(2, 255);
        assert_eq!(decode_type(id), Some(NodeType::Verse));
        assert_eq!(decode_verse(id), Some((2, 255)));
    }

    #[test]
    fn test_encode_decode_word() {
        let id = encode_word(123456);
        assert_eq!(decode_type(id), Some(NodeType::Word));
        assert_eq!(decode_word(id), Some(123456));
    }

    #[test]
    fn test_encode_decode_word_instance() {
        let id = encode_word_instance(2, 255, 5);
        assert_eq!(decode_type(id), Some(NodeType::WordInstance));
        assert_eq!(decode_word_instance(id), Some((2, 255, 5)));
    }

    #[test]
    fn test_encode_decode_knowledge() {
        // 1. Create a base node (VERSE:2:255)
        let _base_ukey = "VERSE:2:255";
        let base_id = encode_verse(2, 255);
        let axis = KnowledgeAxis::Translation;

        // 2. Encode knowledge node
        let k_id = encode_knowledge(base_id, axis);
        assert_eq!(decode_type(k_id), Some(NodeType::Knowledge));

        // 3. Decode components
        let (decoded_base_id, decoded_axis) = decode_knowledge_id(k_id).unwrap();
        assert_eq!(decoded_base_id, base_id);
        assert!(matches!(decoded_axis, KnowledgeAxis::Translation));

        // 4. Verify UKEY round trip
        let k_ukey = to_ukey(k_id).unwrap();
        assert_eq!(k_ukey, "VERSE:2:255:translation");

        // 5. Verify parsing round trip
        let parsed_id = from_ukey(&k_ukey).unwrap();
        assert_eq!(parsed_id, k_id);
    }

    #[test]
    fn test_knowledge_node_complex_base() {
        // Test with WordInstance as base (uses more bits)
        let base_ukey = "WORD_INSTANCE:114:6:5";
        let base_id = encode_word_instance(114, 6, 5);
        let axis = KnowledgeAxis::Tajweed;

        let k_id = encode_knowledge(base_id, axis);
        let (decoded_base, decoded_axis) = decode_knowledge_id(k_id).unwrap();

        assert_eq!(decoded_base, base_id);
        assert_eq!(to_ukey(decoded_base).unwrap(), base_ukey);
        assert!(matches!(decoded_axis, KnowledgeAxis::Tajweed));
    }

    // Parser tests
    #[test]
    fn test_parse_chapter() {
        assert_eq!(parse_chapter("CHAPTER:1").unwrap(), 1);
        assert_eq!(parse_chapter("CHAPTER:114").unwrap(), 114);
        assert!(parse_chapter("CHAPTER:115").is_err());
        assert!(parse_chapter("VERSE:1:1").is_err());
    }

    #[test]
    fn test_parse_verse() {
        assert_eq!(parse_verse("VERSE:1:1").unwrap(), (1, 1));
        assert_eq!(parse_verse("VERSE:2:286").unwrap(), (2, 286));
        assert!(parse_verse("VERSE:1").is_err());
        assert!(parse_verse("CHAPTER:1").is_err());
        assert!(parse_verse("1:1").is_err()); // Unprefixed format not supported
    }

    #[test]
    fn test_parse_word() {
        assert_eq!(parse_word("WORD:123").unwrap(), 123);
        assert!(parse_word("VERSE:1:1").is_err());
    }

    #[test]
    fn test_parse_word_instance() {
        assert_eq!(
            parse_word_instance("WORD_INSTANCE:1:1:3").unwrap(),
            (1, 1, 3)
        );
        assert!(parse_word_instance("WORD:123").is_err());
    }

    #[test]
    fn test_parse_knowledge() {
        let (base, axis) = parse_knowledge("VERSE:1:1:memorization").unwrap();
        assert_eq!(base, "VERSE:1:1");
        assert!(matches!(axis, KnowledgeAxis::Memorization));

        let (base, axis) = parse_knowledge("WORD_INSTANCE:1:1:3:translation").unwrap();
        assert_eq!(base, "WORD_INSTANCE:1:1:3");
        assert!(matches!(axis, KnowledgeAxis::Translation));
    }

    #[test]
    fn test_node_type_detection() {
        assert!(matches!(node_type("CHAPTER:1").unwrap(), NodeType::Chapter));
        assert!(matches!(node_type("VERSE:1:1").unwrap(), NodeType::Verse));
        assert!(node_type("1:1").is_err()); // Unprefixed format not supported
        assert!(matches!(node_type("WORD:123").unwrap(), NodeType::Word));
        assert!(matches!(
            node_type("WORD_INSTANCE:1:1:3").unwrap(),
            NodeType::WordInstance
        ));
        assert!(matches!(
            node_type("VERSE:1:1:memorization").unwrap(),
            NodeType::Knowledge
        ));
    }

    #[test]
    fn test_roundtrip() {
        // Build then parse should return original values
        let chapter_id = chapter(5);
        assert_eq!(parse_chapter(&chapter_id).unwrap(), 5);

        let verse_id = verse(2, 255);
        assert_eq!(parse_verse(&verse_id).unwrap(), (2, 255));

        let word_id = word(999);
        assert_eq!(parse_word(&word_id).unwrap(), 999);
    }

    // Validation tests
    #[test]
    fn test_validate_happy_path() {
        assert!(validate("CHAPTER:1").is_ok());
        assert!(validate("VERSE:114:6").is_ok());
        assert!(validate("WORD:12345").is_ok());
        assert!(validate("WORD_INSTANCE:2:286:1").is_ok());
        assert!(validate("VERSE:1:1:memorization").is_ok());
    }

    #[test]
    fn test_validate_error_cases() {
        // Malformed
        assert!(validate("CHAPTER").is_err());
        assert!(validate("VERSE:1").is_err());
        assert!(validate("WORD:").is_err());
        assert!(validate("WORD_INSTANCE:1:1").is_err());
        assert!(validate("VERSE:1:1:").is_err());
        assert!(validate("VERSE:1:1:unknown_axis").is_err());

        // Invalid chapter/verse numbers
        assert!(validate("CHAPTER:0").is_err());
        assert!(validate("CHAPTER:115").is_err());
        assert!(validate("VERSE:0:1").is_err());
        assert!(validate("VERSE:1:0").is_err());

        // Unprefixed
        assert!(validate("1:1").is_err());
    }
}
