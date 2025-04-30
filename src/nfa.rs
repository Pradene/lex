use std::collections::{BTreeMap, BTreeSet};
use std::default::Default;
use std::fmt;

use crate::Action;
use crate::Regex;
use crate::StateID;
use crate::Symbol;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NFA {
    pub states: BTreeSet<StateID>,
    pub alphabet: BTreeSet<char>,
    pub transitions: BTreeMap<(StateID, Symbol), BTreeSet<StateID>>,
    pub start_state: StateID,
    pub final_states: BTreeSet<StateID>,
    pub actions: BTreeMap<StateID, Action>,
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

        writeln!(f, "Actions:")?;
        for (state, action) in &self.actions {
            writeln!(f, "  {:?} ->  {}", state, action)?;
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
            actions: BTreeMap::new(),
        }
    }
}

impl From<Regex> for NFA {
    fn from(regex: Regex) -> NFA {
        match regex {
            Regex::Empty => NFA::empty(),
            Regex::Char(c) => NFA::char(c),
            Regex::CharClass(class) => NFA::char_class(class),
            Regex::NegatedCharClass(class) => NFA::negated_char_class(class),
            Regex::Dot => NFA::dot(),

            Regex::Concat(left, right) => NFA::concat(NFA::from(*left), NFA::from(*right)),
            Regex::Union(left, right) => NFA::union(NFA::from(*left), NFA::from(*right)),
            Regex::Kleene(inner) => NFA::kleene(NFA::from(*inner)),
            Regex::Option(inner) => NFA::optional(NFA::from(*inner)),
            Regex::Plus(inner) => NFA::plus(NFA::from(*inner)),
            Regex::Bounded(inner, min, max) => NFA::bounded(NFA::from(*inner), min, max),

            _ => panic!("Not implemented"),
        }
    }
}

impl NFA {
    pub fn new(string: &String) -> Result<NFA, String> {
        let nfa = NFA::from(
            Regex::new(string).map_err(|e| format!("{} : {}", string, e))?
        );

        Ok(nfa)
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

    pub fn add_action(&mut self, state: StateID, action: Action) {
        self.actions.insert(state, action);
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

    pub fn negated_char_class(class: BTreeSet<char>) -> NFA {
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
            let new_state = nfa.add_state();
            first_map.insert(state, new_state);
            if let Some(action) = first.actions.get(&state) {
                nfa.actions.insert(new_state, action.clone());
            }
        }

        let mut second_map = BTreeMap::new();
        for &state in &second.states {
            let new_state = nfa.add_state();
            second_map.insert(state, new_state);
            if let Some(action) = second.actions.get(&state) {
                nfa.actions.insert(new_state, action.clone());
            }
        }

        nfa.start_state = first_map[&first.start_state];

        for ((from, symbol), to_states) in &first.transitions {
            for &to in to_states {
                nfa.add_transition(first_map[from], symbol.clone(), first_map[&to]);
            }
        }

        for ((from, symbol), to_states) in &second.transitions {
            for &to in to_states {
                nfa.add_transition(second_map[from], symbol.clone(), second_map[&to]);
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
            if let Some(action) = second.actions.get(&final_state) {
                nfa.actions.insert(second_map[&final_state], action.clone());
            }
        }

        nfa.alphabet.extend(first.alphabet.iter());
        nfa.alphabet.extend(second.alphabet.iter());

        nfa
    }

    pub fn union_multiples(nfas: Vec<NFA>) -> NFA {
        match nfas.len() {
            0 => NFA::empty(),
            1 => nfas.into_iter().next().unwrap(),
            _ => {
                let mut iter = nfas.into_iter();
                let first = iter.next().unwrap();
                iter.fold(first, |acc, nfa| NFA::union(acc, nfa))
            }
        }
    }

    pub fn union(first: NFA, second: NFA) -> NFA {
        let mut nfa = NFA::default();
        let start = nfa.add_state();
        nfa.start_state = start;

        let mut first_map = BTreeMap::new();
        for &state in &first.states {
            let new_state = nfa.add_state();
            first_map.insert(state, new_state);
            if let Some(action) = first.actions.get(&state) {
                nfa.actions.insert(new_state, action.clone());
            }
        }

        let mut second_map = BTreeMap::new();
        for &state in &second.states {
            let new_state = nfa.add_state();
            second_map.insert(state, new_state);
            if let Some(action) = second.actions.get(&state) {
                nfa.actions.insert(new_state, action.clone());
            }
        }

        nfa.add_transition(start, Symbol::Epsilon, first_map[&first.start_state]);
        nfa.add_transition(start, Symbol::Epsilon, second_map[&second.start_state]);

        for ((from, symbol), to_states) in &first.transitions {
            for &to in to_states {
                nfa.add_transition(first_map[from], symbol.clone(), first_map[&to]);
            }
        }
        for ((from, symbol), to_states) in &second.transitions {
            for &to in to_states {
                nfa.add_transition(second_map[from], symbol.clone(), second_map[&to]);
            }
        }

        for &final_state in &first.final_states {
            nfa.final_states.insert(first_map[&final_state]);
        }
        for &final_state in &second.final_states {
            nfa.final_states.insert(second_map[&final_state]);
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

    pub fn bounded(inner: NFA, min: usize, max: Option<usize>) -> NFA {
        if min == 0 && max.is_none() {
            return NFA::kleene(inner);
        }

        let mut nfa = NFA::concat_multiples(vec![inner.clone(); min]);

        if let Some(max) = max {
            let mut optional_parts = Vec::new();
            for _ in min..max {
                optional_parts.push(NFA::optional(inner.clone()));
            }

            if !optional_parts.is_empty() {
                nfa = NFA::concat_multiples(vec![nfa, NFA::concat_multiples(optional_parts)]);
            }
        } else {
            let kleene = NFA::kleene(inner);
            nfa = NFA::concat(nfa, kleene);
        }

        nfa
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
}
