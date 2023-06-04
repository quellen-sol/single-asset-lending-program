pub fn calculate_interest(amount: u64, interest_rate: f32) -> u64 {
  let to_repay = (amount as f64) * (interest_rate as f64);
  to_repay as u64
}