pub fn nibble_key(bytes: &[u8]) -> Vec<u8> {
    let mut nibbles = Vec::with_capacity(bytes.len() * 2);
    for byte in bytes {
        nibbles.push(byte >> 4);
        nibbles.push(byte & 0x0F);
    }
    nibbles
}

pub fn match_prefix(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b).take_while(|(x, y)| x == y).count()
}