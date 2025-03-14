use std::collections::BTreeSet;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Symbol {
    Epsilon,
    Char(char),
    CharClass(BTreeSet<char>),
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Symbol::Epsilon => write!(f, "ε"),
            Symbol::Char(c) => write!(f, "{}", c),
            Symbol::CharClass(set) => {
                write!(f, "[")?;
                for c in set {
                    write!(f, "{}", c)?;
                }
                write!(f, "]")
            }
        }
    }
}
