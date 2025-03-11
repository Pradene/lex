use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter, Result};

type State = usize;

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

impl Symbol {
    pub fn matches(&self, c: char) -> bool {
        match self {
            Symbol::Epsilon => false,
            Symbol::Char(ch) => *ch == c,
            Symbol::CharClass(set) => set.contains(&c),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NFA {
    states: BTreeSet<State>,
    alphabet: BTreeSet<char>,
    transitions: BTreeMap<(State, Symbol), BTreeSet<State>>,
    start_state: State,
    finite_states: BTreeSet<State>,
}

impl Display for NFA {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "NFA Specification:")?;

        writeln!(f, "States: {:?}", self.states)?;

        let alphabet: String = self.alphabet.iter().collect();
        writeln!(f, "Alphabet: {}", alphabet)?;

        writeln!(f, "Start State: {:?}", self.start_state)?;

        writeln!(f, "Finite States: {:?}", self.finite_states)?;

        writeln!(f, "Transitions:")?;
        for ((state, symbol), next_states) in &self.transitions {
            let sorted_next: Vec<_> = next_states.iter().collect();
            writeln!(f, "  δ({:?}, {}) = {:?}", state, symbol, sorted_next)?;
        }

        Ok(())
    }
}

impl NFA {
    fn new() -> NFA {
        NFA {
            states: BTreeSet::new(),
            alphabet: BTreeSet::new(),
            transitions: BTreeMap::new(),
            start_state: 0,
            finite_states: BTreeSet::new(),
        }
    }

    fn add_state(&mut self) -> State {
        let state = if self.states.is_empty() {
            0
        } else {
            self.states.iter().max().unwrap() + 1
        };

        self.states.insert(state);

        state
    }

    fn add_transition(&mut self, from: State, symbol: Symbol, to: State) {
        self.transitions
            .entry((from, symbol.clone()))
            .or_insert(BTreeSet::new())
            .insert(to);

        match symbol {
            Symbol::Epsilon => {}
            Symbol::Char(c) => {
                self.alphabet.insert(c);
            }
            Symbol::CharClass(class) => {
                for &c in &class {
                    self.alphabet.insert(c);
                }
            }
        }
    }

    pub fn from_char(c: char) -> NFA {
        let mut nfa = NFA::new();
        let start = nfa.add_state();
        let end = nfa.add_state();

        nfa.start_state = start;
        nfa.finite_states.insert(end);
        nfa.add_transition(start, Symbol::Char(c), end);

        nfa
    }

    pub fn from_char_class(chars: BTreeSet<char>) -> NFA {
        let mut nfa = NFA::new();
        let start = nfa.add_state();
        let end = nfa.add_state();

        nfa.start_state = start;
        nfa.finite_states.insert(end);
        nfa.add_transition(start, Symbol::CharClass(chars), end);

        nfa
    }

    pub fn from_char_range(start_char: char, end_char: char) -> NFA {
        let mut chars = BTreeSet::new();

        let start_code = start_char as u32;
        let end_code = end_char as u32;

        for code in start_code..=end_code {
            if let Some(c) = char::from_u32(code) {
                chars.insert(c);
            }
        }

        Self::from_char_class(chars)
    }

    pub fn concat(first: NFA, second: NFA) -> NFA {
        let mut nfa = NFA::new();
        let start = nfa.add_state();
        nfa.start_state = start;

        let mut first_map = BTreeMap::new();
        for &state in &first.states {
            let new = nfa.add_state();
            first_map.insert(state, new);
        }

        let mut second_map = BTreeMap::new();
        for &state in &second.states {
            let new = nfa.add_state();
            second_map.insert(state, new);
        }

        for &finite in &second.finite_states {
            nfa.finite_states.insert(second_map[&finite]);
        }

        nfa.add_transition(
            nfa.start_state,
            Symbol::Epsilon,
            first_map[&first.start_state],
        );
        for (&(from, ref symbol), to_states) in &first.transitions {
            for &to in to_states {
                nfa.add_transition(first_map[&from], symbol.clone(), first_map[&to]);
            }
        }

        for (&(from, ref symbol), to_states) in &second.transitions {
            for &to in to_states {
                nfa.add_transition(second_map[&from], symbol.clone(), second_map[&to]);
            }
        }

        for &finite in &first.finite_states {
            nfa.add_transition(
                first_map[&finite],
                Symbol::Epsilon,
                second_map[&second.start_state],
            );
        }

        for &c in &first.alphabet {
            nfa.alphabet.insert(c);
        }
        for &c in &second.alphabet {
            nfa.alphabet.insert(c);
        }

        nfa
    }

    pub fn alternate(first: NFA, second: NFA) -> NFA {
        let mut nfa = NFA::new();

        let start = nfa.add_state();
        let end = nfa.add_state();

        nfa.start_state = start;
        nfa.finite_states.insert(end);

        let mut first_map = BTreeMap::new();
        for &state in &first.states {
            let new = nfa.add_state();
            first_map.insert(state, new);
        }

        let mut second_map = BTreeMap::new();
        for &state in &second.states {
            let new = nfa.add_state();
            second_map.insert(state, new);
        }

        nfa.add_transition(start, Symbol::Epsilon, first_map[&first.start_state]);
        nfa.add_transition(start, Symbol::Epsilon, second_map[&second.start_state]);

        for (&(from, ref symbol), to_states) in &first.transitions {
            for &to in to_states {
                nfa.add_transition(first_map[&from], symbol.clone(), first_map[&to]);
            }
        }

        for (&(from, ref symbol), to_states) in &second.transitions {
            for &to in to_states {
                nfa.add_transition(second_map[&from], symbol.clone(), second_map[&to]);
            }
        }

        for &finite in &first.finite_states {
            nfa.add_transition(first_map[&finite], Symbol::Epsilon, end);
        }
        for &finite in &second.finite_states {
            nfa.add_transition(second_map[&finite], Symbol::Epsilon, end);
        }

        for &c in &first.alphabet {
            nfa.alphabet.insert(c);
        }
        for &c in &second.alphabet {
            nfa.alphabet.insert(c);
        }

        nfa
    }

    pub fn kleene(inner: NFA) -> NFA {
        let mut nfa = NFA::new();

        let start = nfa.add_state();
        let end = nfa.add_state();

        nfa.start_state = start;
        nfa.finite_states.insert(end);

        let mut map = BTreeMap::new();
        for &state in &inner.states {
            let new = nfa.add_state();
            map.insert(state, new);
        }

        for (&(from, ref symbol), to_states) in &inner.transitions {
            for &to in to_states {
                nfa.add_transition(map[&from], symbol.clone(), map[&to]);
            }
        }

        nfa.add_transition(start, Symbol::Epsilon, end);
        nfa.add_transition(start, Symbol::Epsilon, map[&inner.start_state]);

        for &finite in &inner.finite_states {
            let mapped_finite = map[&finite];
            nfa.add_transition(mapped_finite, Symbol::Epsilon, end);
            nfa.add_transition(mapped_finite, Symbol::Epsilon, map[&inner.start_state]);
        }

        nfa.alphabet = inner.alphabet.clone();

        nfa
    }

    pub fn plus(inner: NFA) -> NFA {
        let inner_copy = inner.clone();
        let kleene_part = NFA::kleene(inner);
        NFA::concat(inner_copy, kleene_part)
    }

    pub fn optional(inner: NFA) -> NFA {
        let mut nfa = NFA::new();

        let start = nfa.add_state();
        let end = nfa.add_state();

        nfa.start_state = start;
        nfa.finite_states.insert(end);

        let mut map = BTreeMap::new();
        for &state in &inner.states {
            let new = nfa.add_state();
            map.insert(state, new);
        }

        for (&(from, ref symbol), to_states) in &inner.transitions {
            for &to in to_states {
                nfa.add_transition(map[&from], symbol.clone(), map[&to]);
            }
        }

        nfa.add_transition(start, Symbol::Epsilon, end);

        nfa.add_transition(start, Symbol::Epsilon, map[&inner.start_state]);

        for &finite in &inner.finite_states {
            nfa.add_transition(map[&finite], Symbol::Epsilon, end);
        }

        nfa.alphabet = inner.alphabet.clone();

        nfa
    }
}
