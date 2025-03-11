use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter, Result};

type State = usize;

type Symbol = Option<char>;
const EPSILON: Symbol = None;

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
            writeln!(
                f,
                "  δ({:?}, {}) = {:?}",
                state,
                symbol.unwrap_or('_'),
                sorted_next
            )?;
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
            .entry((from, symbol))
            .or_insert(BTreeSet::new())
            .insert(to);

        if let Some(s) = symbol {
            self.alphabet.insert(s);
        }
    }

    pub fn from_char(c: char) -> NFA {
        let mut nfa = NFA::new();
        let start = nfa.add_state();
        let end = nfa.add_state();

        nfa.start_state = start;
        nfa.finite_states.insert(end);
        nfa.add_transition(start, Some(c), end);

        nfa
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

        nfa.add_transition(nfa.start_state, EPSILON, first_map[&first.start_state]);
        for (&(from, symbol), to_states) in &first.transitions {
            for &to in to_states {
                let from = first_map[&from];
                let to = first_map[&to];
                nfa.add_transition(from, symbol, to);
            }
        }

        for (&(from, symbol), to_states) in &second.transitions {
            for &to in to_states {
                let from = second_map[&from];
                let to = second_map[&to];
                nfa.add_transition(from, symbol, to);
            }
        }

        for &finite in &first.finite_states {
            nfa.add_transition(first_map[&finite], EPSILON, second_map[&second.start_state]);
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

        nfa.add_transition(start, EPSILON, first_map[&first.start_state]);
        nfa.add_transition(start, EPSILON, second_map[&second.start_state]);

        for (&(from, symbol), to_states) in &first.transitions {
            for &to in to_states {
                nfa.add_transition(first_map[&from], symbol, first_map[&to]);
            }
        }

        for (&(from, symbol), to_states) in &second.transitions {
            for &to in to_states {
                nfa.add_transition(second_map[&from], symbol, second_map[&to]);
            }
        }

        for &finite in &first.finite_states {
            nfa.add_transition(first_map[&finite], EPSILON, end);
        }
        for &finite in &second.finite_states {
            nfa.add_transition(second_map[&finite], EPSILON, end);
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

        for ((from, symbol), to_states) in &inner.transitions {
            for &to in to_states {
                let from = map[from];
                let to = map[&to];
                nfa.add_transition(from, *symbol, to);
            }
        }

        nfa.add_transition(start, EPSILON, end);
        nfa.add_transition(start, EPSILON, map[&inner.start_state]);

        for &finite in &inner.finite_states {
            let mapped_finite = map[&finite];
            nfa.add_transition(mapped_finite, EPSILON, end);
            nfa.add_transition(mapped_finite, EPSILON, map[&inner.start_state]);
        }

        nfa.alphabet = inner.alphabet.clone();

        nfa
    }
}
