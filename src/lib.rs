// k = 32
// p = 7
// emax = 96
// emin = -95
// bias = 101
// comb = 11
// sigf = 20

// our view is (spaces denote seps between S, G, T):
// x xxxxxxxxxxx xxxxxxxxxxxxxxxxxxxx
// 1 11122223333 44445555666677778888
#[derive(Clone, Copy)]
pub struct d32(u32);

pub enum Class {
    QuietNaN,
    SignalingNaN,
    NegativeInf,
    NegativeNormal,
    NegativeSubnormal,
    NegativeZero,
    PositiveZero,
    PositiveSubnormal,
    PositiveNormal,
    PositiveInf,
}

impl d32 {
    /* ********************************************** *
     *                HELPER FUNCTIONS                *
     * ********************************************** */

    fn exponent_form_one(&self) -> bool {
        // first exponent encoding requires the first
        // two bits of G be 00, 01, or 10
        self.is_finite() && (self.0 & 0x60000000 != 0x60000000)
    }

    fn significand(&self) -> u32 {
        // if exponent is of first form, then signif
        // is last 23 bits; otherwise, it's last 21
        // bits, with 100 leading for a total of 24
        if self.exponent_form_one() {
            self.0 & 0x007fffff
        } else {
            (self.0 & 0x001fffff) | 0x00800000
        }
    }

    fn exponent(&self) -> u32 {
        // if exponent is of first form, then it is
        // G0 through G7; otherwise, it's G2 through
        // G9 (inclusive both times)
        if !self.is_finite() {
            0
        } else if self.exponent_form_one() {
            self.0 & 0x7f800000 >> 23
        } else {
            self.0 & 0x1fe00000 >> 21
        }
    }

    fn is_quiet(&self) -> bool {
        self.is_nan() && !self.is_signaling()
    }

    /* ********************************************** *
     *             GENERAL-COMP FUNCTIONS             *
     * ********************************************** */

    pub fn quantum(&self) -> Self {
        // if it's a NaN, we want to canonicalize it
        // and propagate its payload
        if self.is_signaling() {
            d32(self.0 & 0x7e07ffff)
        } else if self.is_nan() {
            d32(self.0 & 0x7c07ffff)
        // if it's +/-inf, its quantum is +inf
        } else if self.is_infinite() {
            d32(0x78000000)
        // for finite numbers, we clear the sign bit,
        // leave the exponent untouched, and wipe out
        // the significand, leaving 1
        } else if self.exponent_form_one() {
            d32(self.0 & 0x7f800001)
        } else {
            d32(self.0 & 0x7fe00001)
        }
    }

    /* ********************************************** *
     *              QUIET-COMP FUNCTIONS              *
     * ********************************************** */

    pub fn negate(&self) -> Self {
        d32(self.0 ^ 0x80000000)
    }

    pub fn abs(&self) -> Self {
        d32(self.0 & 0x7fffffff)
    }

    pub fn copy_sign(&self, y: &d32) -> Self {
        d32(self.abs().0 | (y.0 & 0x80000000))
    }

    pub fn encode_binary(&self) -> Self {
        d32(self.0)
    }

    pub fn decode_binary(&self) -> Self {
        d32(self.0)
    }

    /* ********************************************** *
     *               NON-COMP FUNCTIONS               *
     * ********************************************** */

    pub fn is_754_version_1985() -> bool {
        todo!()
    }

    pub fn is_754_version_2008() -> bool {
        todo!()
    }

    pub fn is_754_version_2019() -> bool {
        todo!()
    }

    pub fn class(&self) -> Class {
        if self.is_signaling() {
            Class::SignalingNaN
        } else if self.is_nan() {
            Class::QuietNaN
        } else if self.is_sign_minus() {
            if self.is_infinite() {
                Class::NegativeInf
            } else if self.is_normal() {
                Class::NegativeNormal
            } else if self.is_subnormal() {
                Class::NegativeSubnormal
            } else {
                Class::NegativeZero
            }
        } else if self.is_infinite() {
            Class::PositiveInf
        } else if self.is_normal() {
            Class::PositiveNormal
        } else if self.is_subnormal() {
            Class::PositiveSubnormal
        } else {
            Class::PositiveZero
        }
    }

    pub fn is_sign_minus(&self) -> bool {
        self.0 & 0x80000000 == 0x80000000
    }

    pub fn is_normal(&self) -> bool {
        self.is_finite() && !self.is_zero() && !self.is_subnormal()
    }

    pub fn is_finite(&self) -> bool {
        !(self.is_infinite() || self.is_nan())
    }

    pub fn is_zero(&self) -> bool {
        // need combination field to not indicate inf or nan,
        self.is_finite()
        // and need significand to be zero
        && (self.significand() == 0 || !self.is_canonical())
    }

    pub fn is_subnormal(&self) -> bool {
        self.is_finite()
            && !self.is_zero()
            && self.exponent() < 6
            && self.significand() * 10u32.pow(self.exponent()) < 1000000
    }

    pub fn is_infinite(&self) -> bool {
        !self.is_nan() && (self.0 & 0x78000000 == 0x78000000)
    }

    pub fn is_nan(&self) -> bool {
        self.0 & 0x7c000000 == 0x7c000000
    }

    pub fn is_signaling(&self) -> bool {
        self.0 & 0x7e000000 == 0x7e000000
    }

    pub fn is_canonical(&self) -> bool {
        (self.is_nan() && self.0 & 0x7df00000 == 0x7c000000)
            || (self.is_infinite() && self.0 & 0x7fffffff == 0x78000000)
            || (self.is_finite() && self.significand() <= 9999999)
    }

    pub fn radix(&self) -> u32 {
        10
    }

    pub fn total_order(&self, y: &d32) -> bool {
        match (self.class(), y.class()) {
            // TODO: handle NaN payloads---reps of same payload for
            // same NaN class should spit out true
            (Class::QuietNaN, Class::QuietNaN) => self.is_sign_minus() && !y.is_sign_minus(),
            (Class::SignalingNaN, Class::SignalingNaN) => {
                self.is_sign_minus() && !y.is_sign_minus()
            }
            (Class::QuietNaN, Class::SignalingNaN) => self.is_sign_minus(),
            (Class::SignalingNaN, Class::QuietNaN) => !y.is_sign_minus(),
            (Class::QuietNaN, _) => self.is_sign_minus(),
            (Class::SignalingNaN, _) => self.is_sign_minus(),
            (_, Class::QuietNaN) => !y.is_sign_minus(),
            (_, Class::SignalingNaN) => !y.is_sign_minus(),
            (Class::NegativeInf, Class::NegativeInf) => true,
            (Class::NegativeInf, _) => true,
            (_, Class::NegativeInf) => false,
            (Class::PositiveInf, Class::PositiveInf) => true,
            (Class::PositiveInf, _) => false,
            (_, Class::PositiveInf) => true,
            (Class::NegativeNormal, Class::NegativeNormal) => {
                (self.significand() > y.significand() && self.exponent() >= y.exponent())
                    || (self.exponent() - y.exponent() > 6)
                    || (self.exponent() >= y.exponent()
                        && (self.significand() as u64 * 10u64.pow(self.exponent() - y.exponent())
                            >= y.significand() as u64))
                    || (self.exponent() <= y.exponent()
                        && (self.significand() as u64
                            > y.significand() as u64 * 10u64.pow(y.exponent() - self.exponent())))
            }
            (Class::NegativeNormal, _) => true,
            (_, Class::NegativeNormal) => false,
            (Class::NegativeSubnormal, Class::NegativeSubnormal) => {
                (self.significand() > y.significand() && self.exponent() >= y.exponent())
                    || (self.exponent() - y.exponent() > 6)
                    || (self.exponent() >= y.exponent()
                        && (self.significand() as u64 * 10u64.pow(self.exponent() - y.exponent())
                            >= y.significand() as u64))
                    || (self.exponent() < y.exponent()
                        && (self.significand() as u64
                            > y.significand() as u64 * 10u64.pow(y.exponent() - self.exponent())))
            }
            (Class::NegativeSubnormal, _) => true,
            (_, Class::NegativeSubnormal) => false,
            (Class::NegativeZero, Class::NegativeZero) => self.exponent() >= y.exponent(),
            (Class::NegativeZero, _) => true,
            (_, Class::NegativeZero) => false,
            (Class::PositiveZero, Class::PositiveZero) => self.exponent() <= y.exponent(),
            (Class::PositiveZero, _) => true,
            (_, Class::PositiveZero) => false,
            (Class::PositiveSubnormal, Class::PositiveSubnormal) => {
                (self.significand() < y.significand() && self.exponent() <= y.exponent())
                    || (y.exponent() - self.exponent() > 6)
                    || (self.exponent() <= y.exponent()
                        && (self.significand() as u64
                            <= y.significand() as u64 * 10u64.pow(y.exponent() - self.exponent())))
                    || (self.exponent() > y.exponent()
                        && (self.significand() as u64 * 10u64.pow(self.exponent() - y.exponent())
                            < y.significand() as u64))
            }
            (Class::PositiveSubnormal, _) => true,
            (_, Class::PositiveSubnormal) => false,
            (Class::PositiveNormal, Class::PositiveNormal) => {
                (self.significand() < y.significand() && self.exponent() <= y.exponent())
                    || (y.exponent() - self.exponent() > 6)
                    || (self.exponent() <= y.exponent()
                        && (self.significand() as u64
                            <= y.significand() as u64 * 10u64.pow(y.exponent() - self.exponent())))
                    || (self.exponent() > y.exponent()
                        && (self.significand() as u64 * 10u64.pow(self.exponent() - y.exponent())
                            < y.significand() as u64))
            }
        }
    }

    pub fn total_order_mag(&self, y: &d32) -> bool {
        self.abs().total_order(&y.abs())
    }

    pub fn same_quantum(&self, y: &d32) -> bool {
        (self.is_nan() && y.is_nan())
            || (self.is_infinite() && y.is_infinite())
            || (self.is_finite() && y.is_finite() && self.exponent() == y.exponent())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    #[test]
    fn nan_test() {
        let nan = d32(0x7c000000);
        let snan = d32(0x7e000000);
        let inf = d32(0x78000000);
        let zero = d32(0);
        let thirty = d32(0x00800003);

        assert!(nan.is_nan());
        assert!(snan.is_nan());
        assert!(!inf.is_nan());
        assert!(!zero.is_nan());
        assert!(!thirty.is_nan());

        let now = Instant::now();
        nan.is_nan();
        snan.is_nan();
        inf.is_nan();
        zero.is_nan();
        thirty.is_nan();
        let elapsed = now.elapsed();
        println!("Total call time: {} ns", elapsed.as_nanos());
    }
}
