use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::convert::From;
use std::default::Default;
use std::fmt;

use crate::{Action, StateID, TransitionSymbol, NFA};

#[derive(Debug, Clone)]
pub struct DFA {
    pub states: BTreeSet<StateID>,
    pub alphabet: BTreeSet<char>,
    pub transitions: BTreeMap<(StateID, char), StateID>,
    pub initial_state: StateID,
    pub final_states: BTreeSet<StateID>,
    pub actions: BTreeMap<StateID, Action>,
}

impl Default for DFA {
    fn default() -> Self {
        DFA {
            states: BTreeSet::new(),
            alphabet: BTreeSet::new(),
            transitions: BTreeMap::new(),
            initial_state: 0,
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

        writeln!(f, "Start StateID: {:?}", self.initial_state)?;

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
        dfa.initial_state = dfa_state_counter;

        dfa_state_counter += 1;
        let mut queue = VecDeque::new();
        queue.push_back(start_set);

        while let Some(current_nfa_states) = queue.pop_front() {
            let current_dfa_state = state_map[&current_nfa_states];

            for &symbol in &dfa.alphabet {
                let mut next_nfa_states = BTreeSet::new();

                for &nfa_state in &current_nfa_states {
                    if let Some(targets) = nfa
                        .transitions
                        .get(&(nfa_state, TransitionSymbol::Literal(symbol)))
                    {
                        next_nfa_states.extend(targets);
                    }
                    for ((src, sym), targets) in &nfa.transitions {
                        if *src == nfa_state {
                            if let TransitionSymbol::Set(char_set) = sym {
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
                    .insert((current_dfa_state, symbol), target_dfa_state);
            }
        }

        dfa.minimize()
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
        let mut current_state = self.initial_state;
        let mut last_accepting_state = None;
        let mut last_accepting_length = 0;

        let chars: Vec<char> = input.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            if !self.alphabet.contains(&c) {
                break;
            }

            match self.transitions.get(&(current_state, c)) {
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
        // Step 1: Initialize partition with accepting and rejecting states
        let mut partition: Vec<BTreeSet<usize>> = Vec::new();

        let reject_states: BTreeSet<usize> = self
            .states
            .difference(&self.final_states)
            .cloned()
            .collect();
        if !reject_states.is_empty() {
            partition.push(reject_states);
        }

        // Separate accepting states by their ACTION
        let mut action_groups: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
        for &state in &self.final_states {
            if let Some(action) = self.actions.get(&state) {
                action_groups
                    .entry(action.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(state);
            }
        }

        for (_, group) in action_groups {
            partition.push(group);
        }

        // Step 2: Refine partition iteratively
        loop {
            let mut new_partition: Vec<BTreeSet<usize>> = Vec::new();
            for group in &partition {
                let mut subgroups: HashMap<Vec<usize>, BTreeSet<usize>> = HashMap::new();
                for &state in group {
                    let mut signature: Vec<usize> = Vec::new();
                    for symbol in &self.alphabet {
                        if let Some(&target) = self.transitions.get(&(state, *symbol)) {
                            let target_group = partition
                                .iter()
                                .position(|g| g.contains(&target))
                                .unwrap_or(0);
                            signature.push(target_group);
                        } else {
                            signature.push(usize::MAX);
                        }
                    }
                    subgroups
                        .entry(signature)
                        .or_insert_with(BTreeSet::new)
                        .insert(state);
                }
                for (_, subgroup) in subgroups {
                    new_partition.push(subgroup);
                }
            }
            if partition == new_partition {
                break;
            }
            partition = new_partition;
        }

        // Step 3: Build minimized DFA
        let mut state_to_group: BTreeMap<usize, usize> = BTreeMap::new();
        for (group_idx, group) in partition.iter().enumerate() {
            for &state in group {
                state_to_group.insert(state, group_idx);
            }
        }

        let new_states: BTreeSet<usize> = (0..partition.len()).collect();

        // Build new transitions
        let mut new_transitions: BTreeMap<(usize, char), usize> = BTreeMap::new();
        for (&(old_state, symbol), &target) in &self.transitions {
            let new_state = state_to_group[&old_state];
            let new_target = state_to_group[&target];
            new_transitions.insert((new_state, symbol), new_target);
        }

        // Map initial and final states
        let new_initial = state_to_group[&self.initial_state];
        let new_finals = self
            .final_states
            .iter()
            .map(|&s| state_to_group[&s])
            .collect::<BTreeSet<_>>();

        // Remap actions to use new state indices
        let mut new_actions: BTreeMap<usize, String> = BTreeMap::new();
        for (&old_state, action) in &self.actions {
            if let Some(&new_state) = state_to_group.get(&old_state) {
                new_actions.insert(new_state, action.clone());
            }
        }

        DFA {
            actions: new_actions,
            states: new_states,
            alphabet: self.alphabet.clone(),
            transitions: new_transitions,
            initial_state: new_initial,
            final_states: new_finals,
        }
    }
}
