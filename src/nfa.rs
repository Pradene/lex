use std::collections::{BTreeMap, BTreeSet};
use std::default::Default;
use std::fmt;

use crate::ParseError;
use crate::RegexParser;
use crate::StateID;
use crate::Symbol;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NFA {
    pub states: BTreeSet<StateID>,
    pub alphabet: BTreeSet<char>,
    pub transitions: BTreeMap<(StateID, Symbol), BTreeSet<StateID>>,
    pub start_state: StateID,
    pub final_states: BTreeSet<StateID>,
}

impl fmt::Display for NFA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "NFA Specification:")?;

        writeln!(f, "States: {:?}", self.states)?;

        let alphabet: String = self.alphabet.iter().collect();
        writeln!(f, "Alphabet: {}", alphabet)?;

        writeln!(f, "Start StateID: {:?}", self.start_state)?;

        writeln!(f, "Finite States: {:?}", self.final_states)?;

        writeln!(f, "Transitions:")?;
        for ((state, symbol), next_states) in &self.transitions {
            let sorted_next: Vec<_> = next_states.iter().collect();
            writeln!(f, "  δ({:?}, {}) = {:?}", state, symbol, sorted_next)?;
        }

        Ok(())
    }
}

impl Default for NFA {
    fn default() -> NFA {
        NFA {
            states: BTreeSet::new(),
            alphabet: BTreeSet::new(),
            transitions: BTreeMap::new(),
            start_state: 0,
            final_states: BTreeSet::new(),
        }
    }
}

impl NFA {
    pub fn new(regex: String) -> Result<NFA, ParseError> {
        RegexParser::new(&regex).parse()
    }

    pub fn is_empty(&self) -> bool {
        *self == NFA::empty()
    }

    fn add_state(&mut self) -> StateID {
        let state = if self.states.is_empty() {
            0
        } else {
            self.states.iter().max().unwrap() + 1
        };

        self.states.insert(state);

        state
    }

    fn add_transition(&mut self, from: StateID, symbol: Symbol, to: StateID) {
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

    pub fn empty() -> NFA {
        let mut nfa = NFA::default();
        let start = nfa.add_state();
        let end = nfa.add_state();

        nfa.start_state = start;
        nfa.final_states.insert(end);
        nfa.add_transition(start, Symbol::Epsilon, end);

        nfa
    }

    pub fn char(c: char) -> NFA {
        let mut nfa = NFA::default();
        let start = nfa.add_state();
        let end = nfa.add_state();

        nfa.start_state = start;
        nfa.final_states.insert(end);
        nfa.add_transition(start, Symbol::Char(c), end);

        nfa
    }

    pub fn char_class(chars: BTreeSet<char>) -> NFA {
        let mut nfa = NFA::default();
        let start = nfa.add_state();
        let end = nfa.add_state();

        nfa.start_state = start;
        nfa.final_states.insert(end);
        nfa.add_transition(start, Symbol::CharClass(chars), end);

        nfa
    }

    pub fn char_class_negated(class: BTreeSet<char>) -> NFA {
        let mut negated = BTreeSet::new();
        for c in (0..128).map(|i| i as u8 as char) {
            if !class.contains(&c) {
                negated.insert(c);
            }
        }
        NFA::char_class(negated)
    }

    pub fn concat_multiples(nfas: Vec<NFA>) -> NFA {
        match nfas.len() {
            0 => NFA::empty(),
            1 => nfas.into_iter().next().unwrap(),
            _ => {
                let mut iter = nfas.into_iter();
                let first = iter.next().unwrap();
                iter.fold(first, |acc, nfa| NFA::concat(acc, nfa))
            }
        }
    }

    pub fn concat(first: NFA, second: NFA) -> NFA {
        let mut nfa = NFA::default();

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

        nfa.start_state = first_map[&first.start_state];

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

        for &final_state in &first.final_states {
            nfa.add_transition(
                first_map[&final_state],
                Symbol::Epsilon,
                second_map[&second.start_state],
            );
        }

        for &final_state in &second.final_states {
            nfa.final_states.insert(second_map[&final_state]);
        }

        nfa.alphabet.extend(first.alphabet.iter());
        nfa.alphabet.extend(second.alphabet.iter());

        nfa
    }

    pub fn union(first: NFA, second: NFA) -> NFA {
        let mut nfa = NFA::default();

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

        let end = nfa.add_state();
        nfa.final_states.insert(end);

        for &finite in &first.final_states {
            nfa.add_transition(first_map[&finite], Symbol::Epsilon, end);
        }
        for &finite in &second.final_states {
            nfa.add_transition(second_map[&finite], Symbol::Epsilon, end);
        }

        nfa.alphabet.extend(first.alphabet.iter());
        nfa.alphabet.extend(second.alphabet.iter());

        nfa
    }

    pub fn kleene(inner: NFA) -> NFA {
        let mut nfa = NFA::default();

        let start = nfa.add_state();
        nfa.start_state = start;

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

        let end = nfa.add_state();
        nfa.final_states.insert(end);

        nfa.add_transition(start, Symbol::Epsilon, map[&inner.start_state]);
        nfa.add_transition(start, Symbol::Epsilon, end);

        for &finite in &inner.final_states {
            nfa.add_transition(map[&finite], Symbol::Epsilon, map[&inner.start_state]);
            nfa.add_transition(map[&finite], Symbol::Epsilon, end);
        }

        nfa.alphabet.extend(inner.alphabet.iter());

        nfa
    }

    pub fn plus(inner: NFA) -> NFA {
        let inner_copy = inner.clone();
        let kleene_part = NFA::kleene(inner);
        NFA::concat(inner_copy, kleene_part)
    }

    pub fn optional(inner: NFA) -> NFA {
        let mut nfa = NFA::default();

        let start = nfa.add_state();

        nfa.start_state = start;

        let mut map = BTreeMap::new();
        for &state in &inner.states {
            let new = nfa.add_state();
            map.insert(state, new);
        }

        let end = nfa.add_state();
        nfa.final_states.insert(end);

        for (&(from, ref symbol), to_states) in &inner.transitions {
            for &to in to_states {
                nfa.add_transition(map[&from], symbol.clone(), map[&to]);
            }
        }

        nfa.add_transition(start, Symbol::Epsilon, end);
        nfa.add_transition(start, Symbol::Epsilon, map[&inner.start_state]);

        for &finite in &inner.final_states {
            nfa.add_transition(map[&finite], Symbol::Epsilon, end);
        }

        nfa.alphabet.extend(inner.alphabet.iter());

        nfa
    }

    pub fn dot() -> NFA {
        let mut chars = BTreeSet::new();
        for c in 0..128u8 {
            chars.insert(c as char);
        }

        NFA::char_class(chars)
    }

    pub fn epsilon_closure(&self, states: &BTreeSet<StateID>) -> BTreeSet<StateID> {
        let mut closure = states.clone();
        let mut stack: Vec<StateID> = states.iter().cloned().collect();

        while let Some(state) = stack.pop() {
            if let Some(next_states) = self.transitions.get(&(state, Symbol::Epsilon)) {
                for &next in next_states {
                    if closure.insert(next) {
                        stack.push(next);
                    }
                }
            }
        }

        closure
    }

    pub fn move_on_symbol(&self, states: &BTreeSet<StateID>, symbol: char) -> BTreeSet<StateID> {
        let mut set = BTreeSet::new();
        for &state in states {
            if let Some(targets) = self.transitions.get(&(state, Symbol::Char(symbol))) {
                set.extend(targets);
            }
        }

        set
    }
}
