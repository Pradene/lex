use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter, Result};

use crate::State;
use crate::Symbol;

#[derive(Debug, Clone)]
pub struct NFA {
    pub states: BTreeSet<State>,
    pub alphabet: BTreeSet<char>,
    pub transitions: BTreeMap<(State, Symbol), BTreeSet<State>>,
    pub start_state: State,
    pub final_states: BTreeSet<State>,
}

impl Display for NFA {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "NFA Specification:")?;

        writeln!(f, "States: {:?}", self.states)?;

        let alphabet: String = self.alphabet.iter().collect();
        writeln!(f, "Alphabet: {}", alphabet)?;

        writeln!(f, "Start State: {:?}", self.start_state)?;

        writeln!(f, "Finite States: {:?}", self.final_states)?;

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
            final_states: BTreeSet::new(),
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

    pub fn empty() -> NFA {
        let mut nfa = NFA::new();
        let start = nfa.add_state();
        let end = nfa.add_state();

        nfa.start_state = start;
        nfa.final_states.insert(end);
        nfa.add_transition(start, Symbol::Epsilon, end);

        nfa
    }

    pub fn from_char(c: char) -> NFA {
        let mut nfa = NFA::new();
        let start = nfa.add_state();
        let end = nfa.add_state();

        nfa.start_state = start;
        nfa.final_states.insert(end);
        nfa.add_transition(start, Symbol::Char(c), end);

        nfa
    }

    pub fn from_char_class(chars: BTreeSet<char>) -> NFA {
        let mut nfa = NFA::new();
        let start = nfa.add_state();
        let end = nfa.add_state();

        nfa.start_state = start;
        nfa.final_states.insert(end);
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
    
        // Set the start state to be the start state of the first NFA.
        nfa.start_state = first_map[&first.start_state];
        
        // Copy transitions for the first NFA.
        for (&(from, ref symbol), to_states) in &first.transitions {
            for &to in to_states {
                nfa.add_transition(first_map[&from], symbol.clone(), first_map[&to]);
            }
        }
        
        // Copy transitions for the second NFA.
        for (&(from, ref symbol), to_states) in &second.transitions {
            for &to in to_states {
                nfa.add_transition(second_map[&from], symbol.clone(), second_map[&to]);
            }
        }
        
        // For each final state in the first NFA, add an epsilon transition to the start state of the second NFA.
        for &final_state in &first.final_states {
            nfa.add_transition(first_map[&final_state], Symbol::Epsilon, second_map[&second.start_state]);
        }
    
        // The final states of the new NFA are the final states of the second NFA.
        for &final_state in &second.final_states {
            nfa.final_states.insert(second_map[&final_state]);
        }
    
        nfa.alphabet.extend(first.alphabet.iter());
        nfa.alphabet.extend(second.alphabet.iter());
    
        nfa
    }
    

    pub fn union(first: NFA, second: NFA) -> NFA {
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
        let mut nfa = NFA::new();

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
        let mut nfa = NFA::new();

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

    pub fn epsilon_closure(&self, states: &BTreeSet<State>) -> BTreeSet<State> {
        let mut closure = states.clone();
        let mut stack: Vec<State> = states.iter().cloned().collect();

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

    // Compute the set of states reachable from `states` via transitions labeled with the given symbol.
    pub fn move_on_symbol(&self, states: &BTreeSet<State>, symbol: char) -> BTreeSet<State> {
        let mut set = BTreeSet::new();
        for &state in states {
            if let Some(targets) = self.transitions.get(&(state, Symbol::Char(symbol))) {
                set.extend(targets);
            }
        }

        set
    }
}
