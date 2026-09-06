//! Fixed PatternSet acceptance corpus. Do not tune fixtures to benchmark results.

pub const FAMILIES: &[&str] = &["keywords", "fields", "captures", "mixed"];
pub const COUNTS: &[usize] = &[32, 128, 1024];
pub const LENGTHS: &[usize] = &[256, 4096, 65536];
pub const DENSITIES: &[&str] = &["none", "early", "late", "dense"];

pub fn patterns(family: &str, count: usize) -> Vec<String> {
    (0..count)
        .map(|id| match family {
            "keywords" => format!("event{id:04}"),
            "fields" => format!(r"key{id:04}=[0-9]+;"),
            "mixed" if id % 4 == 0 => format!(r"([a-z]+):([0-9]+);tag{id:04}"),
            _ => format!(r"key{id:04}=([a-z]+):([0-9]+);"),
        })
        .collect()
}

fn hit(family: &str, id: usize) -> String {
    match family {
        "keywords" => format!("event{id:04}"),
        "fields" => format!("key{id:04}=123;"),
        "mixed" if id % 4 == 0 => format!("alice:123;tag{id:04}"),
        _ => format!("key{id:04}=alice:123;"),
    }
}

pub fn haystack(family: &str, count: usize, length: usize, density: &str) -> String {
    let mut text = " ".repeat(length);
    let ids = [0, count / 2, count - 1];
    match density {
        "early" => {
            let value = hit(family, ids[0]);
            text.replace_range(..value.len(), &value);
        }
        "late" => {
            let value = hit(family, ids[2]);
            text.replace_range(length - value.len().., &value);
        }
        "dense" => {
            let mut offset = 0;
            let mut index = 0;
            loop {
                let value = hit(family, ids[index % ids.len()]);
                if offset + value.len() > length {
                    break;
                }
                text.replace_range(offset..offset + value.len(), &value);
                offset += value.len() + 1;
                index += 1;
            }
        }
        _ => {}
    }
    text
}
