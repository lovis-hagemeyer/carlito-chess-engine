use std::fmt::Debug;



#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score {
    pub s: i16
}

impl Score {
    pub const MAX_MATE_DISTANCE: i16 = 1000;

    pub const WINNING: Score = Score { s: i16::MAX - Score::MAX_MATE_DISTANCE-1 };
    pub const LOOSING: Score = Score { s: i16::MIN + Score::MAX_MATE_DISTANCE+2 };

    pub const POSITIVE_INFTY: Score = Score { s:i16::MAX };
    pub const NEGATIVE_INFTY: Score = Score { s:i16::MIN+1 };

    pub fn from_centi_pawns(s: i32) -> Score {
        Score { s: s.clamp(Score::LOOSING.s as i32, Score::WINNING.s as i32) as i16 }
    }

    pub fn from_mate_distance(m: i16) -> Score {
        if m > 0 {
            Score { s: i16::MAX - m }
        } else {
            Score { s: i16::MIN - m + 1 }
        }
    }

    pub fn mate(&self) -> Option<i16> {
        if self > &Score::WINNING {
            Some( Score::POSITIVE_INFTY.s - self.s)
        } else if self < &Score::LOOSING {
            Some( Score::NEGATIVE_INFTY.s - self.s)
        } else {
            None
        }
    }

    pub fn centi_pawns(&self) -> Option<i16> {
        if self <= &Score::WINNING && self >= &Score::LOOSING {
            Some(self.s)
        } else {
            None
        }
    }
}

impl core::ops::Neg for Score {
    type Output = Score;
    fn neg(self) -> Self::Output {
        Score { s: -self.s }
    }
}

impl core::ops::Add for Score {
    type Output = Score;
    fn add(self, rhs: Self) -> Self::Output {
        Score { s: self.s + rhs.s }
    }
}