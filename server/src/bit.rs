pub fn test_bit(s: usize, i: usize) -> bool {
  s & (1 << i) != 0
}

pub fn popcnt(s: isize) -> u32 {
  s.count_ones()
}

pub fn lowbit(s: isize) -> isize {
  s & (-s)
}

pub fn clear_lowbit(s: isize) -> isize {
  s & (s - 1)
}
