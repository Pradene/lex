use std::collections::BTreeSet;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum TransitionSymbol {
    Epsilon,
    Char(char),
    CharClass(BTreeSet<char>),
}

impl Display for TransitionSymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            TransitionSymbol::Epsilon => write!(f, "ε"),
            TransitionSymbol::Char(c) => write!(f, "{}", c),
            TransitionSymbol::CharClass(set) => {
                write!(f, "[")?;
                for c in set {
                    write!(f, "{}", c)?;
                }
                write!(f, "]")
            }
        }
    }
}
