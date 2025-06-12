pub fn nibble_key(bytes: &[u8]) -> Vec<u8> {
    bytes.iter()
        .flat_map(|byte| vec![byte >> 4, byte & 0x0F])
        .collect()
}

#[allow(dead_code)]
pub fn match_prefix(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
}