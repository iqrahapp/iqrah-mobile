use super::error::NodeIdError;
use super::models::{KnowledgeAxis, NodeType};

pub type Result<T> = std::result::Result<T, NodeIdError>;

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

/// Parse knowledge node: "VERSE:1:1:memorization" -> ("VERSE:1:1", Memorization)
pub fn parse_knowledge(id: &str) -> Result<(String, KnowledgeAxis)> {
    let parts: Vec<&str> = id.split(':').collect();

    // Knowledge nodes have at least 3 parts: prefix:num:axis or prefix:n1:n2:axis
    if parts.len() < 3 {
        return Err(NodeIdError::InvalidFormat(id.to_string()));
    }

    // Last part is the axis
    let axis_str = parts.last().unwrap();
    let axis = KnowledgeAxis::from_str(axis_str)
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
        if KnowledgeAxis::from_str(last).is_ok() {
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
