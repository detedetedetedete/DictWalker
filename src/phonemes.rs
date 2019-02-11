use std::fmt;
use std::fmt::Formatter;
use std::fmt::Error;
use std::str::FromStr;
use serde::Serializer;

#[derive(Debug)]
pub struct Phoneme {
    pub symbol: String,
    pub ordinal: i8,
    pub accented: bool,
    pub valid: bool
}

impl Phoneme {
    pub fn from_symbol(symbol: &str, accent: bool) -> Phoneme {
        match symbol {
            " " => Phoneme {
                symbol: String::from(" "),
                ordinal: 0,
                accented: false,
                valid: true
            },
            "A" => Phoneme {
                symbol: String::from("A"),
                ordinal: 1,
                accented: accent,
                valid: true
            },
            "A_" => Phoneme {
                symbol: String::from("A_"),
                ordinal: 2,
                accented: accent,
                valid: true
            },
            "B" => Phoneme {
                symbol: String::from("B"),
                ordinal: 3,
                accented: accent,
                valid: true
            },
            "C" => Phoneme {
                symbol: String::from("C"),
                ordinal: 4,
                accented: accent,
                valid: true
            },
            "C2" => Phoneme {
                symbol: String::from("C2"),
                ordinal: 5,
                accented: accent,
                valid: true
            },
            "CH" => Phoneme {
                symbol: String::from("CH"),
                ordinal: 6,
                accented: accent,
                valid: true
            },
            "D" => Phoneme {
                symbol: String::from("D"),
                ordinal: 7,
                accented: accent,
                valid: true
            },
            "DZ" => Phoneme {
                symbol: String::from("DZ"),
                ordinal: 8,
                accented: accent,
                valid: true
            },
            "DZ2" => Phoneme {
                symbol: String::from("DZ2"),
                ordinal: 9,
                accented: accent,
                valid: true
            },
            "E" => Phoneme {
                symbol: String::from("E"),
                ordinal: 10,
                accented: accent,
                valid: true
            },
            "E_" => Phoneme {
                symbol: String::from("E_"),
                ordinal: 11,
                accented: accent,
                valid: true
            },
            "E3_" => Phoneme {
                symbol: String::from("E3_"),
                ordinal: 12,
                accented: accent,
                valid: true
            },
            "F" => Phoneme {
                symbol: String::from("F"),
                ordinal: 13,
                accented: accent,
                valid: true
            },
            "G" => Phoneme {
                symbol: String::from("G"),
                ordinal: 14,
                accented: accent,
                valid: true
            },
            "H" => Phoneme {
                symbol: String::from("H"),
                ordinal: 15,
                accented: accent,
                valid: true
            },
            "I" => Phoneme {
                symbol: String::from("I"),
                ordinal: 16,
                accented: accent,
                valid: true
            },
            "I_" => Phoneme {
                symbol: String::from("I_"),
                ordinal: 17,
                accented: accent,
                valid: true
            },
            "IO_" => Phoneme {
                symbol: String::from("IO_"),
                ordinal: 18,
                accented: accent,
                valid: true
            },
            "IU" => Phoneme {
                symbol: String::from("IU"),
                ordinal: 19,
                accented: accent,
                valid: true
            },
            "IU_" => Phoneme {
                symbol: String::from("IU_"),
                ordinal: 20,
                accented: accent,
                valid: true
            },
            "J." => Phoneme {
                symbol: String::from("J."),
                ordinal: 21,
                accented: accent,
                valid: true
            },
            "K" => Phoneme {
                symbol: String::from("K"),
                ordinal: 22,
                accented: accent,
                valid: true
            },
            "L" => Phoneme {
                symbol: String::from("L"),
                ordinal: 23,
                accented: accent,
                valid: true
            },
            "M" => Phoneme {
                symbol: String::from("M"),
                ordinal: 24,
                accented: accent,
                valid: true
            },
            "N" => Phoneme {
                symbol: String::from("N"),
                ordinal: 25,
                accented: accent,
                valid: true
            },
            "O_" => Phoneme {
                symbol: String::from("O_"),
                ordinal: 26,
                accented: accent,
                valid: true
            },
            "P" => Phoneme {
                symbol: String::from("P"),
                ordinal: 27,
                accented: accent,
                valid: true
            },
            "R" => Phoneme {
                symbol: String::from("R"),
                ordinal: 28,
                accented: accent,
                valid: true
            },
            "S" => Phoneme {
                symbol: String::from("S"),
                ordinal: 29,
                accented: accent,
                valid: true
            },
            "S2" => Phoneme {
                symbol: String::from("S2"),
                ordinal: 30,
                accented: accent,
                valid: true
            },
            "T" => Phoneme {
                symbol: String::from("T"),
                ordinal: 31,
                accented: accent,
                valid: true
            },
            "U" => Phoneme {
                symbol: String::from("U"),
                ordinal: 32,
                accented: accent,
                valid: true
            },
            "U_" => Phoneme {
                symbol: String::from("U_"),
                ordinal: 33,
                accented: accent,
                valid: true
            },
            "V" => Phoneme {
                symbol: String::from("V"),
                ordinal: 34,
                accented: accent,
                valid: true
            },
            "Z" => Phoneme {
                symbol: String::from("Z"),
                ordinal: 35,
                accented: accent,
                valid: true
            },
            "Z2" => Phoneme {
                symbol: String::from("Z2"),
                ordinal: 36,
                accented: accent,
                valid: true
            },
            "[PAUSE]" => Phoneme {
                symbol: String::from("PAUSE"),
                ordinal: 37,
                accented: accent,
                valid: true
            },
            "[INHALE]" => Phoneme {
                symbol: String::from("INHALE"),
                ordinal: 38,
                accented: accent,
                valid: true
            },
            "[EXHALE]" => Phoneme {
                symbol: String::from("EXHALE"),
                ordinal: 39,
                accented: accent,
                valid: true
            },
            "[SWALLOW]" => Phoneme {
                symbol: String::from("SWALLOW"),
                ordinal: 40,
                accented: accent,
                valid: true
            },
            "[SMACK]" => Phoneme {
                symbol: String::from("SMACK"),
                ordinal: 41,
                accented: accent,
                valid: true
            },
            "[CHAIR]" => Phoneme {
                symbol: String::from("CHAIR"),
                ordinal: 42,
                accented: accent,
                valid: true
            },
            "[STOMACH]" => Phoneme {
                symbol: String::from("STOMACH"),
                ordinal: 43,
                accented: accent,
                valid: true
            },
            "[PAGE]" => Phoneme {
                symbol: String::from("PAGE"),
                ordinal: 44,
                accented: accent,
                valid: true
            },
            "[DOOR]" => Phoneme {
                symbol: String::from("DOOR"),
                ordinal: 45,
                accented: accent,
                valid: true
            },
            "[EH]" => Phoneme {
                symbol: String::from("EH"),
                ordinal: 46,
                accented: accent,
                valid: true
            },
            "[MIDWORDPAUSE]" => Phoneme {
                symbol: String::from("MIDWORDPAUSE"),
                ordinal: 47,
                accented: accent,
                valid: true
            },
            "[NOISE]" => Phoneme {
                symbol: String::from("NOISE"),
                ordinal: 48,
                accented: accent,
                valid: true
            },
            _ => Phoneme {
                symbol: format!("ERR-{}", symbol),
                ordinal: -1,
                accented: accent,
                valid: false
            }
        }
    }
}

impl FromStr for Phoneme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        Ok(Phoneme::from_symbol(s, false))
    }
}

impl fmt::Display for Phoneme {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if self.ordinal == 0 {
            write!(f, "{}", self.symbol)
        } else if self.accented {
            write!(f, "{{{}}}", self.symbol)
        } else {
            write!(f, "[{}]", self.symbol)
        }
    }
}

impl Clone for Phoneme {
    fn clone(&self) -> Self {
        Phoneme {
            symbol: self.symbol.clone(),
            ordinal: self.ordinal,
            accented: self.accented,
            valid: self.valid
        }
    }
}

impl serde::Serialize for Phoneme {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
        S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}
