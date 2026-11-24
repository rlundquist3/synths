use core::fmt::Error;

#[derive(Clone)]
pub struct Clock {
    pub bpm: u16,
    pub division: Note,
    pub time_signature: TimeSignature,
    pub tick_ms: u16,
}

#[derive(Clone)]
pub struct TimeSignature {
    pub per_measure: u8,
    pub base: Note,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Note {
    Quarter,
    Eighth,
    Triplet,
    Sixteenth,
    SixteenthTriplet,
    Thirtysecond,
}

impl Note {
    const NOTES: [Note; 6] = [
        Note::Quarter,
        Note::Eighth,
        Note::Triplet,
        Note::Sixteenth,
        Note::SixteenthTriplet,
        Note::Thirtysecond,
    ];

    fn index(self) -> usize {
        Note::NOTES.iter().position(|n| *n == self).unwrap()
    }

    fn next(self) -> Note {
        let i = self.index();
        Note::NOTES[(i + 1) % Note::NOTES.len()]
    }

    fn prev(self) -> Note {
        let i = self.index();
        Note::NOTES[(i + Note::NOTES.len() - 1) % Note::NOTES.len()]
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ClockError {
    InvalidDivision { base: Note, division: Note },
    InvalidTimeSignature(Note),
    ZeroBpm,
}

impl core::fmt::Display for ClockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ClockError::InvalidDivision { base, division } => write!(
                f,
                "invalid division {:?} for {:?} based time signature",
                division, base
            ),
            ClockError::InvalidTimeSignature(base) => {
                write!(f, "invalid time signature base {:?}", base)
            }
            ClockError::ZeroBpm => write!(f, "bpm must be greater than 0"),
        }
    }
}

impl core::error::Error for ClockError {}

const MINUTE_IN_MS: u16 = 60_000;

impl Clock {
    pub fn new() -> Self {
        let bpm = 120;
        Clock {
            bpm,
            division: Note::Quarter,
            time_signature: TimeSignature {
                per_measure: 4,
                base: Note::Quarter,
            },
            tick_ms: MINUTE_IN_MS / bpm,
        }
    }

    pub fn set_bpm(&mut self, bpm: u16) -> Result<(), ClockError> {
        self.bpm = bpm;
        self.calculate_tick_ms()
    }

    pub fn increment_bpm(&mut self) -> Result<(), ClockError> {
        if self.bpm < 180 {
            self.bpm += 1;
            return self.calculate_tick_ms();
        }
        Ok(())
    }
    pub fn decrement_bpm(&mut self) -> Result<(), ClockError> {
        if self.bpm > 40 {
            self.bpm -= 1;
            return self.calculate_tick_ms();
        }
        Ok(())
    }

    pub fn set_division(&mut self, division: Note) -> Result<(), ClockError> {
        self.division = division;
        self.calculate_tick_ms()
    }

    pub fn increment_division(&mut self) -> Result<(), ClockError> {
        self.division = self.division.next();
        self.calculate_tick_ms()
    }
    pub fn decrement_division(&mut self) -> Result<(), ClockError> {
        self.division = self.division.prev();
        self.calculate_tick_ms()
    }

    pub fn set_time_signature_per_measure(&mut self, per_measure: u8) -> Result<(), ClockError> {
        self.time_signature.per_measure = per_measure;
        self.calculate_tick_ms()
    }

    pub fn increment_time_signature_per_measure(&mut self) -> Result<(), ClockError> {
        if self.time_signature.per_measure <= 12 {
            self.time_signature.per_measure += 1;
            return self.calculate_tick_ms();
        }
        Ok(())
    }
    pub fn decrement_time_signature_per_measure(&mut self) -> Result<(), ClockError> {
        if self.time_signature.per_measure > 1 {
            self.time_signature.per_measure -= 1;
            return self.calculate_tick_ms();
        }
        Ok(())
    }

    pub fn set_time_signature_base(&mut self, base: Note) -> Result<(), ClockError> {
        self.time_signature.base = base;
        self.division = base;
        self.calculate_tick_ms()
    }

    pub fn increment_time_signature_base(&mut self) -> Result<(), ClockError> {
        if self.time_signature.base == Note::Quarter {
            self.time_signature.base = Note::Eighth;
            return self.calculate_tick_ms();
        }
        Ok(())
    }
    pub fn decrement_time_signature_base(&mut self) -> Result<(), ClockError> {
        if self.time_signature.base == Note::Eighth {
            self.time_signature.base = Note::Quarter;
            return self.calculate_tick_ms();
        }
        Ok(())
    }

    fn calculate_tick_ms(&mut self) -> Result<(), ClockError> {
        if self.bpm == 0 {
            return Err(ClockError::ZeroBpm);
        }
        let base = MINUTE_IN_MS / self.bpm;

        let divisor = match self.time_signature.base {
            Note::Quarter => match self.division {
                Note::Quarter => 1,
                Note::Eighth => 2,
                Note::Triplet => 3,
                Note::Sixteenth => 4,
                Note::SixteenthTriplet => 6,
                Note::Thirtysecond => 8,
            },
            Note::Eighth => match self.division {
                Note::Eighth => 1,
                Note::Sixteenth => 2,
                Note::SixteenthTriplet => 3,
                Note::Thirtysecond => 4,
                division => {
                    return Err(ClockError::InvalidDivision {
                        base: Note::Eighth,
                        division,
                    });
                }
            },
            base => return Err(ClockError::InvalidTimeSignature(base)),
        };

        self.tick_ms = base / divisor;

        Ok(())
    }

    pub fn get_tick_ms(&self) -> u16 {
        self.tick_ms
    }
}

#[cfg(test)]
mod tests {
    use crate::controls::clock::{Clock, ClockError, Note};
    use core::assert_eq;

    #[test]
    fn default() {
        let clock = Clock::new();
        assert_eq!(clock.get_tick_ms(), 500);
    }

    #[test]
    fn setting_base_resets_division() {
        let mut clock = Clock::new();
        clock.set_time_signature_base(Note::Eighth);
        assert_eq!(clock.division, Note::Eighth);
    }

    #[test]
    fn divisions_of_120_quarters() {
        let mut clock = Clock::new();
        clock.set_division(Note::Eighth);
        assert_eq!(clock.get_tick_ms(), 250);
        clock.set_division(Note::Triplet);
        assert_eq!(clock.get_tick_ms(), 166);
        clock.set_division(Note::Sixteenth);
        assert_eq!(clock.get_tick_ms(), 125);
        clock.set_division(Note::SixteenthTriplet);
        assert_eq!(clock.get_tick_ms(), 83);
        clock.set_division(Note::Thirtysecond);
        assert_eq!(clock.get_tick_ms(), 62);
    }

    #[test]
    fn divisions_of_120_eights() {
        let mut clock = Clock::new();
        clock.set_time_signature_per_measure(6);
        clock.set_time_signature_base(Note::Eighth);
        assert_eq!(clock.get_tick_ms(), 500);
    }

    #[test]
    fn base_other_than_quarter_or_eighth_invalid() {
        let mut clock = Clock::new();
        for base in [
            Note::Triplet,
            Note::Sixteenth,
            Note::SixteenthTriplet,
            Note::Thirtysecond,
        ] {
            let r = clock.set_time_signature_base(base);
            assert_eq!(r, Err(ClockError::InvalidTimeSignature(base)));
        }
    }
}
