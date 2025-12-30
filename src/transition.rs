use std::collections::BTreeSet;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum TransitionSymbol {
    Epsilon,
    Literal(char),
    Set(BTreeSet<char>),
    NegatedSet(BTreeSet<char>),
}

impl Display for TransitionSymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            TransitionSymbol::Epsilon => write!(f, "ε")?,
            TransitionSymbol::Literal(c) => write!(f, "{}", c)?,
            TransitionSymbol::Set(set) => {
                write!(f, "[")?;
                for c in set {
                    write!(f, "{}", c)?;
                }
                write!(f, "]")?;
            }
            TransitionSymbol::NegatedSet(set) => {
                write!(f, "[^")?;
                for c in set {
                    write!(f, "{}", c)?;
                }
                write!(f, "]")?;
            }
        }

        Ok(())
    }
}
