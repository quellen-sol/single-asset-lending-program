pub fn mul_u64_by_f32(a: u64, b: f32) -> u64 {
  let result = (a as f64) * (b as f64);
  result as u64
}