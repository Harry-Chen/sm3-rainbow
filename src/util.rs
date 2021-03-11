use std::ops::Range;

pub fn generate_cumulative_lengths(len_range: &Range<usize>, charset_len: usize) -> Vec<u64> {
    let mut lens = Vec::new();
    // calculate key space (cumulative)
    lens.push(0);
    for i in 0..len_range.end {
        let prefix_sum = *lens.last().unwrap();
        lens.push(
            prefix_sum
                + if len_range.start <= i + 1 {
                charset_len.pow((i + 1) as u32) as u64
            } else {
                0
            },
        );
    }
    lens
}
