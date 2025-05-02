use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::convert::From;
use std::default::Default;
use std::fmt;

use crate::{Action, StateID, TransitionSymbol, NFA};

#[derive(Debug, Clone)]
pub struct DFA {
    pub states: BTreeSet<StateID>,
    pub alphabet: BTreeSet<char>,
    pub transitions: BTreeMap<(StateID, TransitionSymbol), StateID>,
    pub start_state: StateID,
    pub final_states: BTreeSet<StateID>,
    pub actions: BTreeMap<StateID, Action>,
}

impl Default for DFA {
    fn default() -> Self {
        DFA {
            states: BTreeSet::new(),
            alphabet: BTreeSet::new(),
            transitions: BTreeMap::new(),
            start_state: 0,
            final_states: BTreeSet::new(),
            actions: BTreeMap::new(),
        }
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

        writeln!(f, "Actions:")?;
        for (state, action) in &self.actions {
            writeln!(f, "  {:?} ->  {}", state, action)?;
        }

        Ok(())
    }
}

impl From<NFA> for DFA {
    fn from(nfa: NFA) -> DFA {
        let mut dfa = DFA::default();
        dfa.alphabet.extend(nfa.alphabet.iter());

        let start_set = nfa.epsilon_closure(&BTreeSet::from([nfa.start_state]));
        let mut state_map = BTreeMap::new(); // Maps NFA state subsets to DFA StateIDs
        let mut dfa_state_counter = 0;

        state_map.insert(start_set.clone(), dfa_state_counter);
        dfa.states.insert(dfa_state_counter);
        dfa.start_state = dfa_state_counter;

        dfa_state_counter += 1;
        let mut queue = VecDeque::new();
        queue.push_back(start_set);

        while let Some(current_nfa_states) = queue.pop_front() {
            let current_dfa_state = state_map[&current_nfa_states];

            for &symbol in &dfa.alphabet {
                let mut next_nfa_states = BTreeSet::new();

                for &nfa_state in &current_nfa_states {
                    if let Some(targets) = nfa.transitions.get(&(nfa_state, TransitionSymbol::Char(symbol))) {
                        next_nfa_states.extend(targets);
                    }
                    for ((src, sym), targets) in &nfa.transitions {
                        if *src == nfa_state {
                            if let TransitionSymbol::CharClass(char_set) = sym {
                                if char_set.contains(&symbol) {
                                    next_nfa_states.extend(targets);
                                }
                            }
                        }
                    }
                }

                let next_nfa_states = nfa.epsilon_closure(&next_nfa_states);
                if next_nfa_states.is_empty() {
                    continue;
                }

                let target_dfa_state = match state_map.get(&next_nfa_states) {
                    Some(&id) => id,
                    None => {
                        let new_id = dfa_state_counter;
                        dfa_state_counter += 1;
                        state_map.insert(next_nfa_states.clone(), new_id);
                        dfa.states.insert(new_id);

                        let mut highest_priority_state: Option<StateID> = None;
                        for &nfa_state in &next_nfa_states {
                            if nfa.final_states.contains(&nfa_state) {
                                if highest_priority_state.is_none()
                                    || nfa_state < highest_priority_state.unwrap()
                                {
                                    highest_priority_state = Some(nfa_state);
                                }
                            }
                        }

                        if let Some(state) = highest_priority_state {
                            if let Some(action) = nfa.actions.get(&state) {
                                dfa.final_states.insert(new_id);
                                dfa.actions.insert(new_id, action.clone());
                            }
                        }

                        queue.push_back(next_nfa_states.clone());
                        new_id
                    }
                };

                dfa.transitions
                    .insert((current_dfa_state, TransitionSymbol::Char(symbol)), target_dfa_state);
            }
        }

        dfa
    }
}

impl DFA {
    pub fn simulate(&self, input: &str) -> Vec<(String, Action)> {
        let mut tokens = Vec::new();
        let mut remaining = input.to_string();

        while !remaining.is_empty() {
            let (token, action, rest) = self.scan_next_token(&remaining);
            if token.is_empty() {
                break;
            }

            tokens.push((token, action));
            remaining = rest;
        }

        tokens
    }

    fn scan_next_token(&self, input: &str) -> (String, Action, String) {
        let mut current_state = self.start_state;
        let mut last_accepting_state = None;
        let mut last_accepting_length = 0;

        let chars: Vec<char> = input.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            if !self.alphabet.contains(&c) {
                break;
            }

            match self.transitions.get(&(current_state, TransitionSymbol::Char(c))) {
                Some(&next_state) => {
                    current_state = next_state;
                    if self.final_states.contains(&current_state) {
                        last_accepting_state = Some(current_state);
                        last_accepting_length = i + 1;
                    }
                }
                None => break,
            }
        }

        match last_accepting_state {
            Some(state) => {
                let token = chars[..last_accepting_length].iter().collect::<String>();
                let action = self
                    .actions
                    .get(&state)
                    .cloned()
                    .unwrap_or_else(|| "UNKNOWN".to_string());
                let rest = input[last_accepting_length..].to_string();
                (token, action, rest)
            }
            None => (String::new(), String::new(), input.to_string()),
        }
    }

    pub fn minimize(&self) -> DFA {
        // Implementation of DFA minimization algorithm (Hopcroft's algorithm)
        // This would reduce the number of states in the DFA

        self.clone()
    }
}
