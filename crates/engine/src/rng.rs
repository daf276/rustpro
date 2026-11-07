// Port of RNG/Xoshiro256.hpp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Xoshiro256StarStar {
    s: [u64; 4],
}

impl Xoshiro256StarStar {
    pub fn new(seed: [u64; 4]) -> Self {
        Self { s: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        let [s0, s1, s2, s3] = &mut self.s;
        let result = s1.wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = *s1 << 17;
        *s2 ^= *s0;
        *s3 ^= *s1;
        *s1 ^= *s2;
        *s0 ^= *s3;
        *s2 ^= t;
        *s3 = s3.rotate_left(45);
        result
    }

    // Port of duel::get_next_integer, including its rejection of every draw up to and including `max % range`.
    pub fn get_next_integer(&mut self, low: i32, high: i32) -> i32 {
        let range = (i64::from(high) - i64::from(low) + 1) as u64;
        let limit = u64::MAX % range;
        let mut value = self.next_u64();
        while value <= limit {
            value = self.next_u64();
        }
        (value % range).wrapping_add(low as u64) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_match_reference_xoshiro256starstar_sequence() {
        let mut rng = Xoshiro256StarStar::new([1, 2, 3, 4]);
        assert_eq!([rng.next_u64(), rng.next_u64(), rng.next_u64()], [11520, 0, 1509978240]);
    }
}
