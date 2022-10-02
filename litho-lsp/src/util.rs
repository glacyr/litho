pub fn line_col_to_offset(source: &str, line: u32, col: u32) -> usize {
    let line_offset = source
        .split_inclusive("\n")
        .take(line as usize)
        .fold(0, |sum, line| sum + line.len());
    line_offset + col as usize
}
