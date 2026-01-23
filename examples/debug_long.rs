use rexile::Pattern;
fn main() {
    let text_long = &("ab12 ".repeat(200) + "target xyz999 end");
    let pat = Pattern::new("[a-z]+.+[0-9]+").unwrap();

    let start = std::time::Instant::now();
    let result = pat.find(text_long);
    let elapsed = start.elapsed();

    println!("text length: {} bytes", text_long.len());
    println!("find result: {:?}", result);
    if let Some((s, e)) = result {
        println!("matched: {:?}", &text_long[s..e]);
    }
    println!("time: {:?}", elapsed);
}
