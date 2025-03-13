use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::convert::From;
use std::default::Default;
use std::fmt;

use crate::{StateID, Symbol, NFA};

#[derive(Debug, Clone)]
pub struct DFA {
    pub states: BTreeSet<StateID>,
    pub alphabet: BTreeSet<char>,
    pub transitions: BTreeMap<(StateID, Symbol), StateID>,
    pub start_state: StateID,
    pub final_states: BTreeSet<StateID>,
}

impl Default for DFA {
    fn default() -> Self {
        DFA {
            states: BTreeSet::new(),
            alphabet: BTreeSet::new(),
            transitions: BTreeMap::new(),
            start_state: 0,
            final_states: BTreeSet::new(),
        }
    }
}

impl DFA {
    pub fn new(regex: String) -> Result<DFA, String> {
        let nfa = NFA::new(regex).unwrap();
        let dfa = DFA::from(nfa);

        Ok(dfa)
    }
}

impl fmt::Display for DFA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "DFA Specification:")?;

        writeln!(f, "States: {:?}", self.states)?;

        let alphabet: String = self.alphabet.iter().collect();
        writeln!(f, "Alphabet: {}", alphabet)?;

        writeln!(f, "Start StateID: {:?}", self.start_state)?;

        writeln!(f, "Finite States: {:?}", self.final_states)?;

        writeln!(f, "Transitions:")?;
        for ((state, symbol), next_state) in &self.transitions {
            writeln!(f, "  δ({:?}, {}) = {:?}", state, symbol, next_state)?;
        }

        Ok(())
    }
}

impl From<NFA> for DFA {
    fn from(nfa: NFA) -> DFA {
        let mut dfa = DFA::default();
        dfa.alphabet.extend(nfa.alphabet.iter());

        let start_set = [nfa.start_state].iter().cloned().collect();
        let start_closure = nfa.epsilon_closure(&start_set);

        let mut state_map = BTreeMap::new();
        let mut dfa_state_counter = 0;

        state_map.insert(start_closure.clone(), dfa_state_counter);
        dfa.states.insert(dfa_state_counter);
        dfa.start_state = dfa_state_counter;
        if start_closure.iter().any(|s| nfa.final_states.contains(s)) {
            dfa.final_states.insert(dfa_state_counter);
        }

        dfa_state_counter += 1;

        let mut queue = VecDeque::new();
        queue.push_back(start_closure);

        while let Some(current_set) = queue.pop_front() {
            let current_dfa_state = state_map[&current_set];

            // Process single character transitions
            for &symbol in &dfa.alphabet {
                let mut next_set = BTreeSet::new();
                
                // Check all NFA states in current subset for transitions on this symbol
                for &state in &current_set {
                    // Check for Char transitions
                    if let Some(targets) = nfa.transitions.get(&(state, Symbol::Char(symbol))) {
                        next_set.extend(targets);
                    }
                    
                    // Check for CharClass transitions
                    for ((src_state, src_symbol), targets) in &nfa.transitions {
                        if *src_state == state {
                            if let Symbol::CharClass(char_set) = src_symbol {
                                if char_set.contains(&symbol) {
                                    next_set.extend(targets);
                                }
                            }
                        }
                    }
                }
                
                // Apply epsilon closure to the resulting set
                let next_set = nfa.epsilon_closure(&next_set);
                if next_set.is_empty() {
                    continue;
                }

                let target_state = if let Some(&state) = state_map.get(&next_set) {
                    state
                } else {
                    let new_state = dfa_state_counter;
                    dfa_state_counter += 1;
                    state_map.insert(next_set.clone(), new_state);
                    dfa.states.insert(new_state);
                    if next_set.iter().any(|s| nfa.final_states.contains(s)) {
                        dfa.final_states.insert(new_state);
                    }

                    queue.push_back(next_set.clone());

                    new_state
                };

                dfa.transitions
                    .insert((current_dfa_state, Symbol::Char(symbol)), target_state);
            }
        }

        dfa
    }
}