struct Score {
    s: i16
}

impl Score {
    pub const max_mate_distance: i16 = 1000;

    const winning: i16 = i16::MAX - max_mate_distance;
    const loosing: i16 = -winning;

    pub fn from_centi_pawns(s: i16) -> Score {
        Score { s.clamp(loosing, winning) }
    }

    pub fn from_mate_distance(m: i16) -> Score {
        if (m > 0) {
            
        }
    }

    pub fn mate(&self) -> Option<i16> {

    }

    pub fn centi_pawns(&self) -> Option<i16> {

    }
}