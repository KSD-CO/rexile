// Test the boundary logic directly without going through Pattern

fn main() {
    let text = "hello world";

    println!("Text: {:?}", text);
    println!("Bytes: {} chars", text.len());

    // Test is_at_boundary manually
    println!("\nManual boundary check:");
    for i in 0..=text.len() {
        let is_boundary = is_at_boundary(text, i);
        let char_at = if i < text.len() {
            format!("{:?}", text.as_bytes()[i] as char)
        } else {
            "END".to_string()
        };
        if is_boundary {
            println!("  Position {}: {} - BOUNDARY", i, char_at);
        }
    }

    println!("\nExpected boundaries: [0, 5, 6, 11]");
}

fn is_at_boundary(text: &str, pos: usize) -> bool {
    let bytes = text.as_bytes();

    // Check characters before and after position
    let before_is_word = if pos == 0 {
        false
    } else {
        is_word_byte(bytes[pos - 1])
    };

    let after_is_word = if pos >= bytes.len() {
        false
    } else {
        is_word_byte(bytes[pos])
    };

    // Boundary = transition between word/non-word
    before_is_word != after_is_word
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_lowercase() || b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_'
}
