use std::io;

const VOWELS: &[char] = &['a', 'A', 'e', 'E', 'i', 'I', 'o', 'O', 'u', 'U'];

fn main() -> io::Result<()> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    let mut count = 0;
    for ch in buffer.chars() {
        if VOWELS.contains(&ch) {
            count += 1;
        }
    }

    println!("{}", count);

    Ok(())
}
