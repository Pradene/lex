use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::convert::From;
use std::fmt::{Display, Formatter, Result};

use crate::{State, Symbol, NFA};

#[derive(Debug, Clone)]
pub struct DFA {
    pub states: BTreeSet<State>,
    pub alphabet: BTreeSet<char>,
    pub transitions: BTreeMap<(State, Symbol), State>,
    pub start_state: State,
    pub finite_states: BTreeSet<State>,
}

impl DFA {
    pub fn new() -> DFA {
        DFA {
            states: BTreeSet::new(),
            alphabet: BTreeSet::new(),
            transitions: BTreeMap::new(),
            start_state: 0,
            finite_states: BTreeSet::new(),
        }
    }
}

impl Display for DFA {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "DFA Specification:")?;

        writeln!(f, "States: {:?}", self.states)?;

        let alphabet: String = self.alphabet.iter().collect();
        writeln!(f, "Alphabet: {}", alphabet)?;

        writeln!(f, "Start State: {:?}", self.start_state)?;

        writeln!(f, "Finite States: {:?}", self.finite_states)?;

        writeln!(f, "Transitions:")?;
        for ((state, symbol), next_state) in &self.transitions {
            writeln!(f, "  δ({:?}, {}) = {:?}", state, symbol, next_state)?;
        }

        Ok(())
    }
}

impl From<NFA> for DFA {
    fn from(nfa: NFA) -> DFA {
        let mut dfa = DFA::new();
        // Copy the NFA alphabet (note: it should not include epsilon)
        dfa.alphabet = nfa.alphabet.clone();

        // Compute the epsilon closure of the NFA's start state.
        let start_set: BTreeSet<State> = [nfa.start_state].iter().cloned().collect();
        let start_closure = nfa.epsilon_closure(&start_set);

        // Mapping from sets of NFA states to a DFA state.
        let mut state_map: BTreeMap<BTreeSet<State>, State> = BTreeMap::new();
        let mut dfa_state_counter = 0;

        state_map.insert(start_closure.clone(), dfa_state_counter);
        dfa.states.insert(dfa_state_counter);
        dfa.start_state = dfa_state_counter;
        if start_closure.iter().any(|s| nfa.finite_states.contains(s)) {
            dfa.finite_states.insert(dfa_state_counter);
        }

        dfa_state_counter += 1;

        let mut queue = VecDeque::new();
        queue.push_back(start_closure);

        // Process each set of NFA states (each representing one DFA state)
        while let Some(current_set) = queue.pop_front() {
            let current_dfa_state = state_map[&current_set];

            for &symbol in &dfa.alphabet {
                // Compute all NFA states reachable by reading `symbol` from any state in `current_set`.
                let move_set = nfa.move_on_symbol(&current_set, symbol);
                // Then take the epsilon closure of those states.
                let next_set = nfa.epsilon_closure(&move_set);
                if next_set.is_empty() {
                    continue;
                }

                // If we haven't seen this set, add it as a new DFA state.
                let target_state = if let Some(&state) = state_map.get(&next_set) {
                    state
                } else {
                    let new_state = dfa_state_counter;
                    dfa_state_counter += 1;
                    state_map.insert(next_set.clone(), new_state);
                    dfa.states.insert(new_state);
                    if next_set.iter().any(|s| nfa.finite_states.contains(s)) {
                        dfa.finite_states.insert(new_state);
                    }
                    queue.push_back(next_set.clone());
                    new_state
                };

                // Record the deterministic transition.
                dfa.transitions
                    .insert((current_dfa_state, Symbol::Char(symbol)), target_state);
            }
        }

        dfa
    }
}
