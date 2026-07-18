//! Span-neutral state-machine definition and validation core.
//!
//! This workspace-private crate is shared by runtime and procedural-macro front ends.

use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateCategory {
    Active,
    Terminal,
    Absorbing,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Trigger {
    Direct,
    Route(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphPolicy {
    MustReachTerminal,
    Acyclic,
    CyclesExplicit,
    Persistent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationKind {
    Machine,
    State,
    Transition,
    Initial,
    Policy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefinitionLocation {
    pub kind: DeclarationKind,
    pub ordinal: usize,
}

impl DefinitionLocation {
    const MACHINE: Self = Self {
        kind: DeclarationKind::Machine,
        ordinal: 0,
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateDeclaration {
    pub id: String,
    pub category: StateCategory,
    pub routes: Vec<String>,
    pub declared_at: DefinitionLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionDeclaration {
    pub id: String,
    pub source: String,
    pub destination: String,
    pub trigger: Trigger,
    pub cycle_acknowledged: bool,
    pub declared_at: DefinitionLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialDeclaration {
    pub state: String,
    pub declared_at: DefinitionLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyDeclaration {
    pub policy: GraphPolicy,
    pub declared_at: DefinitionLocation,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MachineDefinition {
    pub states: Vec<StateDeclaration>,
    pub transitions: Vec<TransitionDeclaration>,
    pub initials: Vec<InitialDeclaration>,
    pub policies: Vec<PolicyDeclaration>,
    next_ordinal: usize,
}

impl MachineDefinition {
    pub fn new() -> Self {
        Self {
            next_ordinal: 1,
            ..Self::default()
        }
    }

    fn location(&mut self, kind: DeclarationKind) -> DefinitionLocation {
        let location = DefinitionLocation {
            kind,
            ordinal: self.next_ordinal,
        };
        self.next_ordinal += 1;
        location
    }

    pub fn state(mut self, id: impl Into<String>, category: StateCategory) -> Self {
        let declared_at = self.location(DeclarationKind::State);
        self.states.push(StateDeclaration {
            id: id.into(),
            category,
            routes: Vec::new(),
            declared_at,
        });
        self
    }

    pub fn routes<I, S>(mut self, state: &str, routes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let declaration = self
            .states
            .iter_mut()
            .find(|declaration| declaration.id == state)
            .expect("routes require the state to be declared first");
        declaration
            .routes
            .extend(routes.into_iter().map(Into::into));
        self
    }

    pub fn initial(mut self, state: impl Into<String>) -> Self {
        let declared_at = self.location(DeclarationKind::Initial);
        self.initials.push(InitialDeclaration {
            state: state.into(),
            declared_at,
        });
        self
    }

    pub fn transition(
        self,
        id: impl Into<String>,
        source: impl Into<String>,
        destination: impl Into<String>,
        trigger: Trigger,
    ) -> Self {
        self.transition_with_cycle_acknowledgement(id, source, destination, trigger, false)
    }

    pub fn acknowledged_transition(
        self,
        id: impl Into<String>,
        source: impl Into<String>,
        destination: impl Into<String>,
        trigger: Trigger,
    ) -> Self {
        self.transition_with_cycle_acknowledgement(id, source, destination, trigger, true)
    }

    fn transition_with_cycle_acknowledgement(
        mut self,
        id: impl Into<String>,
        source: impl Into<String>,
        destination: impl Into<String>,
        trigger: Trigger,
        cycle_acknowledged: bool,
    ) -> Self {
        let declared_at = self.location(DeclarationKind::Transition);
        self.transitions.push(TransitionDeclaration {
            id: id.into(),
            source: source.into(),
            destination: destination.into(),
            trigger,
            cycle_acknowledged,
            declared_at,
        });
        self
    }

    pub fn policy(mut self, policy: GraphPolicy) -> Self {
        let declared_at = self.location(DeclarationKind::Policy);
        self.policies.push(PolicyDeclaration {
            policy,
            declared_at,
        });
        self
    }

    pub fn validate(self) -> Result<ValidatedMachine, ValidationReport> {
        validate(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticCode {
    Sm001,
    Sm002,
    Sm003,
    Sm004,
    Sm005,
    Sm006,
    Sm007,
    Sm008,
    Sm009,
    Sm010,
    Sm011,
    Sm012,
    Sm013,
    Sm014,
    Sm015,
    Sm016,
    Sm017,
    Sm018,
    Sm101,
    Sm102,
    Sm103,
    Sm104,
    Sm105,
}

impl DiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sm001 => "SM001",
            Self::Sm002 => "SM002",
            Self::Sm003 => "SM003",
            Self::Sm004 => "SM004",
            Self::Sm005 => "SM005",
            Self::Sm006 => "SM006",
            Self::Sm007 => "SM007",
            Self::Sm008 => "SM008",
            Self::Sm009 => "SM009",
            Self::Sm010 => "SM010",
            Self::Sm011 => "SM011",
            Self::Sm012 => "SM012",
            Self::Sm013 => "SM013",
            Self::Sm014 => "SM014",
            Self::Sm015 => "SM015",
            Self::Sm016 => "SM016",
            Self::Sm017 => "SM017",
            Self::Sm018 => "SM018",
            Self::Sm101 => "SM101",
            Self::Sm102 => "SM102",
            Self::Sm103 => "SM103",
            Self::Sm104 => "SM104",
            Self::Sm105 => "SM105",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphWitness {
    States(Vec<String>),
    Cycle(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDiagnostic {
    pub code: DiagnosticCode,
    pub message: String,
    pub primary: DefinitionLocation,
    pub related: Vec<DefinitionLocation>,
    pub witness: Option<GraphWitness>,
}

impl ValidationDiagnostic {
    fn new(code: DiagnosticCode, message: impl Into<String>, primary: DefinitionLocation) -> Self {
        Self {
            code,
            message: message.into(),
            primary,
            related: Vec::new(),
            witness: None,
        }
    }

    fn related(mut self, related: impl IntoIterator<Item = DefinitionLocation>) -> Self {
        self.related.extend(related);
        self
    }

    fn witness(mut self, witness: GraphWitness) -> Self {
        self.witness = Some(witness);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    diagnostics: Vec<ValidationDiagnostic>,
}

impl ValidationReport {
    pub fn diagnostics(&self) -> &[ValidationDiagnostic] {
        &self.diagnostics
    }

    pub fn codes(&self) -> Vec<DiagnosticCode> {
        self.diagnostics.iter().map(|error| error.code).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedMachine {
    definition: MachineDefinition,
}

impl ValidatedMachine {
    pub fn definition(&self) -> &MachineDefinition {
        &self.definition
    }
}

fn validate(definition: MachineDefinition) -> Result<ValidatedMachine, ValidationReport> {
    let mut diagnostics = Vec::new();

    if definition.states.is_empty() {
        diagnostics.push(ValidationDiagnostic::new(
            DiagnosticCode::Sm001,
            "a machine must declare at least one state",
            DefinitionLocation::MACHINE,
        ));
    }

    let (state_indices, duplicate_states) = collect_unique_states(&definition, &mut diagnostics);
    collect_unique_transitions(&definition, &mut diagnostics);

    let initial_index = validate_initial(&definition, &state_indices, &mut diagnostics);
    let endpoints_valid = validate_endpoints(&definition, &state_indices, &mut diagnostics);

    validate_triggers_and_declared_routes(&definition, &state_indices, &mut diagnostics);
    validate_state_categories(&definition, &state_indices, &mut diagnostics);

    let graph_is_analyzable = !duplicate_states
        && endpoints_valid
        && initial_index.is_some()
        && !definition.states.is_empty();

    let adjacency = build_adjacency(&definition, &state_indices);
    if graph_is_analyzable {
        validate_local_degrees_and_reachability(
            &definition,
            initial_index.expect("checked above"),
            &adjacency,
            &state_indices,
            &mut diagnostics,
        );
    }

    validate_route_coverage(&definition, &state_indices, &mut diagnostics);

    if diagnostics.is_empty() {
        validate_policies(&definition, &adjacency, &state_indices, &mut diagnostics);
    }

    if diagnostics.is_empty() {
        Ok(ValidatedMachine { definition })
    } else {
        Err(ValidationReport { diagnostics })
    }
}

fn collect_unique_states(
    definition: &MachineDefinition,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) -> (HashMap<String, usize>, bool) {
    let mut indices: HashMap<String, usize> = HashMap::new();
    let mut duplicate = false;

    for (index, state) in definition.states.iter().enumerate() {
        if let Some(original) = indices.get(&state.id).copied() {
            duplicate = true;
            diagnostics.push(
                ValidationDiagnostic::new(
                    DiagnosticCode::Sm004,
                    format!("state `{}` is declared more than once", state.id),
                    state.declared_at,
                )
                .related([definition.states[original].declared_at]),
            );
        } else {
            indices.insert(state.id.clone(), index);
        }
    }

    (indices, duplicate)
}

fn collect_unique_transitions(
    definition: &MachineDefinition,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    let mut indices: HashMap<String, usize> = HashMap::new();

    for (index, transition) in definition.transitions.iter().enumerate() {
        if let Some(original) = indices.get(&transition.id).copied() {
            diagnostics.push(
                ValidationDiagnostic::new(
                    DiagnosticCode::Sm005,
                    format!("transition `{}` is declared more than once", transition.id),
                    transition.declared_at,
                )
                .related([definition.transitions[original].declared_at]),
            );
        } else {
            indices.insert(transition.id.clone(), index);
        }
    }
}

fn validate_initial(
    definition: &MachineDefinition,
    state_indices: &HashMap<String, usize>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) -> Option<usize> {
    match definition.initials.as_slice() {
        [] => {
            diagnostics.push(ValidationDiagnostic::new(
                DiagnosticCode::Sm002,
                "a machine must declare exactly one initial state",
                DefinitionLocation::MACHINE,
            ));
            None
        }
        [initial] => match state_indices.get(&initial.state).copied() {
            Some(index) => Some(index),
            None => {
                diagnostics.push(ValidationDiagnostic::new(
                    DiagnosticCode::Sm018,
                    format!("initial state `{}` is not declared", initial.state),
                    initial.declared_at,
                ));
                None
            }
        },
        initials => {
            diagnostics.push(
                ValidationDiagnostic::new(
                    DiagnosticCode::Sm003,
                    "a machine declares more than one initial state",
                    initials[1].declared_at,
                )
                .related(initials.iter().map(|initial| initial.declared_at)),
            );
            None
        }
    }
}

fn validate_endpoints(
    definition: &MachineDefinition,
    state_indices: &HashMap<String, usize>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) -> bool {
    let mut valid = true;

    for transition in &definition.transitions {
        if !state_indices.contains_key(&transition.source) {
            valid = false;
            diagnostics.push(ValidationDiagnostic::new(
                DiagnosticCode::Sm006,
                format!(
                    "transition `{}` has unknown source state `{}`",
                    transition.id, transition.source
                ),
                transition.declared_at,
            ));
        }
        if !state_indices.contains_key(&transition.destination) {
            valid = false;
            diagnostics.push(ValidationDiagnostic::new(
                DiagnosticCode::Sm007,
                format!(
                    "transition `{}` has unknown destination state `{}`",
                    transition.id, transition.destination
                ),
                transition.declared_at,
            ));
        }
    }

    valid
}

fn validate_triggers_and_declared_routes(
    definition: &MachineDefinition,
    state_indices: &HashMap<String, usize>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    for state in &definition.states {
        let mut routes = HashSet::new();
        for route in &state.routes {
            if !routes.insert(route) {
                diagnostics.push(ValidationDiagnostic::new(
                    DiagnosticCode::Sm016,
                    format!(
                        "route `{route}` is declared more than once for state `{}`",
                        state.id
                    ),
                    state.declared_at,
                ));
            }
        }
    }

    let mut triggers: HashMap<(&str, &Trigger), usize> = HashMap::new();
    for (index, transition) in definition.transitions.iter().enumerate() {
        if !state_indices.contains_key(&transition.source) {
            continue;
        }

        let key = (transition.source.as_str(), &transition.trigger);
        if let Some(original) = triggers.get(&key).copied() {
            let code = match transition.trigger {
                Trigger::Direct => DiagnosticCode::Sm008,
                Trigger::Route(_) => DiagnosticCode::Sm016,
            };
            diagnostics.push(
                ValidationDiagnostic::new(
                    code,
                    format!(
                        "state `{}` has more than one transition for trigger `{:?}`",
                        transition.source, transition.trigger
                    ),
                    transition.declared_at,
                )
                .related([definition.transitions[original].declared_at]),
            );
        } else {
            triggers.insert(key, index);
        }

        if let Trigger::Route(route) = &transition.trigger {
            let state = &definition.states[state_indices[&transition.source]];
            if !state.routes.contains(route) {
                diagnostics.push(ValidationDiagnostic::new(
                    DiagnosticCode::Sm017,
                    format!(
                        "transition `{}` handles undeclared route `{route}` from state `{}`",
                        transition.id, transition.source
                    ),
                    transition.declared_at,
                ));
            }
        }
    }
}

fn validate_state_categories(
    definition: &MachineDefinition,
    state_indices: &HashMap<String, usize>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    let mut absorbing_self_edges = vec![false; definition.states.len()];

    for transition in &definition.transitions {
        let Some(source_index) = state_indices.get(&transition.source).copied() else {
            continue;
        };
        let source = &definition.states[source_index];

        match source.category {
            StateCategory::Active => {}
            StateCategory::Terminal => diagnostics.push(
                ValidationDiagnostic::new(
                    DiagnosticCode::Sm011,
                    format!(
                        "terminal state `{}` cannot have outgoing transition `{}`",
                        source.id, transition.id
                    ),
                    transition.declared_at,
                )
                .related([source.declared_at]),
            ),
            StateCategory::Absorbing if transition.destination == source.id => {
                absorbing_self_edges[source_index] = true;
            }
            StateCategory::Absorbing => diagnostics.push(
                ValidationDiagnostic::new(
                    DiagnosticCode::Sm012,
                    format!(
                        "absorbing state `{}` cannot transition to `{}`",
                        source.id, transition.destination
                    ),
                    transition.declared_at,
                )
                .related([source.declared_at]),
            ),
        }
    }

    for (index, state) in definition.states.iter().enumerate() {
        if state.category == StateCategory::Absorbing && !absorbing_self_edges[index] {
            diagnostics.push(ValidationDiagnostic::new(
                DiagnosticCode::Sm013,
                format!(
                    "absorbing state `{}` must declare a self-transition",
                    state.id
                ),
                state.declared_at,
            ));
        }
    }
}

fn build_adjacency(
    definition: &MachineDefinition,
    state_indices: &HashMap<String, usize>,
) -> Vec<Vec<usize>> {
    let mut adjacency = vec![Vec::new(); definition.states.len()];

    for transition in &definition.transitions {
        if let (Some(source), Some(destination)) = (
            state_indices.get(&transition.source),
            state_indices.get(&transition.destination),
        ) {
            adjacency[*source].push(*destination);
        }
    }

    adjacency
}

fn validate_local_degrees_and_reachability(
    definition: &MachineDefinition,
    initial_index: usize,
    adjacency: &[Vec<usize>],
    state_indices: &HashMap<String, usize>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    let mut incoming = vec![0_usize; definition.states.len()];
    for destinations in adjacency {
        for destination in destinations {
            incoming[*destination] += 1;
        }
    }

    let mut no_incoming = HashSet::new();
    for (index, state) in definition.states.iter().enumerate() {
        if index != initial_index && incoming[index] == 0 {
            no_incoming.insert(index);
            diagnostics.push(ValidationDiagnostic::new(
                DiagnosticCode::Sm009,
                format!("state `{}` has no incoming transition", state.id),
                state.declared_at,
            ));
        }

        if state.category == StateCategory::Active && adjacency[index].is_empty() {
            diagnostics.push(ValidationDiagnostic::new(
                DiagnosticCode::Sm010,
                format!("active state `{}` has no outgoing transition", state.id),
                state.declared_at,
            ));
        }
    }

    let reachable = reachable_from(initial_index, adjacency);
    let reachable_names = definition
        .states
        .iter()
        .enumerate()
        .filter(|(index, _)| reachable.contains(index))
        .map(|(_, state)| state.id.clone())
        .collect::<Vec<_>>();

    for (index, state) in definition.states.iter().enumerate() {
        if !reachable.contains(&index) && !no_incoming.contains(&index) {
            diagnostics.push(
                ValidationDiagnostic::new(
                    DiagnosticCode::Sm014,
                    format!(
                        "state `{}` is unreachable from initial state `{}`",
                        state.id, definition.states[initial_index].id
                    ),
                    state.declared_at,
                )
                .witness(GraphWitness::States(reachable_names.clone())),
            );
        }
    }

    debug_assert_eq!(state_indices.len(), definition.states.len());
}

fn validate_route_coverage(
    definition: &MachineDefinition,
    state_indices: &HashMap<String, usize>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    let mut handled: Vec<HashSet<&str>> = vec![HashSet::new(); definition.states.len()];
    for transition in &definition.transitions {
        if let (Some(source), Trigger::Route(route)) =
            (state_indices.get(&transition.source), &transition.trigger)
        {
            handled[*source].insert(route);
        }
    }

    for (index, state) in definition.states.iter().enumerate() {
        for route in &state.routes {
            if !handled[index].contains(route.as_str()) {
                diagnostics.push(ValidationDiagnostic::new(
                    DiagnosticCode::Sm015,
                    format!("state `{}` has no handler for route `{route}`", state.id),
                    state.declared_at,
                ));
            }
        }
    }
}

fn validate_policies(
    definition: &MachineDefinition,
    adjacency: &[Vec<usize>],
    state_indices: &HashMap<String, usize>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    let selected = definition
        .policies
        .iter()
        .map(|declaration| declaration.policy)
        .collect::<HashSet<_>>();

    if selected.contains(&GraphPolicy::Persistent)
        && selected.contains(&GraphPolicy::MustReachTerminal)
    {
        let declarations = definition
            .policies
            .iter()
            .filter(|declaration| {
                matches!(
                    declaration.policy,
                    GraphPolicy::Persistent | GraphPolicy::MustReachTerminal
                )
            })
            .collect::<Vec<_>>();
        diagnostics.push(
            ValidationDiagnostic::new(
                DiagnosticCode::Sm105,
                "policies `persistent` and `must_reach_terminal` are incompatible",
                declarations[1].declared_at,
            )
            .related(
                declarations
                    .iter()
                    .map(|declaration| declaration.declared_at),
            ),
        );
        return;
    }

    if selected.contains(&GraphPolicy::MustReachTerminal) {
        validate_terminal_reachability(definition, adjacency, diagnostics);
    }

    let components = strongly_connected_components(adjacency);
    let component_ids = component_ids(definition.states.len(), &components);
    let cyclic_components = cyclic_components(adjacency, &components);

    if selected.contains(&GraphPolicy::Acyclic) {
        if let Some(component) = cyclic_components.first() {
            diagnostics.push(
                ValidationDiagnostic::new(
                    DiagnosticCode::Sm103,
                    "policy `acyclic` forbids a declared cycle",
                    definition.states[component[0]].declared_at,
                )
                .witness(GraphWitness::Cycle(cycle_names(definition, component))),
            );
        }
    }

    if selected.contains(&GraphPolicy::CyclesExplicit) {
        let cyclic_ids = cyclic_components
            .iter()
            .map(|component| component_ids[component[0]])
            .collect::<HashSet<_>>();

        for transition in &definition.transitions {
            let source = state_indices[&transition.source];
            let destination = state_indices[&transition.destination];
            let component_id = component_ids[source];
            if component_id == component_ids[destination]
                && cyclic_ids.contains(&component_id)
                && !transition.cycle_acknowledged
            {
                let component = &components[component_id];
                diagnostics.push(
                    ValidationDiagnostic::new(
                        DiagnosticCode::Sm104,
                        format!(
                            "transition `{}` participates in an unacknowledged cycle",
                            transition.id
                        ),
                        transition.declared_at,
                    )
                    .witness(GraphWitness::Cycle(cycle_names(definition, component))),
                );
            }
        }
    }
}

fn validate_terminal_reachability(
    definition: &MachineDefinition,
    adjacency: &[Vec<usize>],
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    let terminals = definition
        .states
        .iter()
        .enumerate()
        .filter(|(_, state)| state.category == StateCategory::Terminal)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();

    if terminals.is_empty() {
        diagnostics.push(ValidationDiagnostic::new(
            DiagnosticCode::Sm101,
            "policy `must_reach_terminal` requires at least one terminal state",
            DefinitionLocation::MACHINE,
        ));
        return;
    }

    let mut reverse = vec![Vec::new(); adjacency.len()];
    for (source, destinations) in adjacency.iter().enumerate() {
        for destination in destinations {
            reverse[*destination].push(source);
        }
    }

    let mut can_reach_terminal = HashSet::new();
    let mut queue = VecDeque::from(terminals);
    while let Some(state) = queue.pop_front() {
        if can_reach_terminal.insert(state) {
            queue.extend(reverse[state].iter().copied());
        }
    }

    for (index, state) in definition.states.iter().enumerate() {
        if state.category != StateCategory::Terminal && !can_reach_terminal.contains(&index) {
            let witness = reachable_without_terminal(index, adjacency, &can_reach_terminal)
                .into_iter()
                .map(|witness_index| definition.states[witness_index].id.clone())
                .collect();
            diagnostics.push(
                ValidationDiagnostic::new(
                    DiagnosticCode::Sm102,
                    format!("state `{}` cannot reach a terminal state", state.id),
                    state.declared_at,
                )
                .witness(GraphWitness::States(witness)),
            );
        }
    }
}

fn reachable_from(start: usize, adjacency: &[Vec<usize>]) -> HashSet<usize> {
    let mut reachable = HashSet::new();
    let mut stack = vec![start];
    while let Some(state) = stack.pop() {
        if reachable.insert(state) {
            stack.extend(adjacency[state].iter().rev().copied());
        }
    }
    reachable
}

fn reachable_without_terminal(
    start: usize,
    adjacency: &[Vec<usize>],
    can_reach_terminal: &HashSet<usize>,
) -> Vec<usize> {
    let mut reachable = HashSet::new();
    let mut stack = vec![start];
    while let Some(state) = stack.pop() {
        if !can_reach_terminal.contains(&state) && reachable.insert(state) {
            stack.extend(adjacency[state].iter().rev().copied());
        }
    }
    let mut reachable = reachable.into_iter().collect::<Vec<_>>();
    reachable.sort_unstable();
    reachable
}

fn strongly_connected_components(adjacency: &[Vec<usize>]) -> Vec<Vec<usize>> {
    struct Tarjan<'a> {
        adjacency: &'a [Vec<usize>],
        next_index: usize,
        indices: Vec<Option<usize>>,
        low_links: Vec<usize>,
        stack: Vec<usize>,
        on_stack: Vec<bool>,
        components: Vec<Vec<usize>>,
    }

    impl Tarjan<'_> {
        fn visit(&mut self, state: usize) {
            let index = self.next_index;
            self.next_index += 1;
            self.indices[state] = Some(index);
            self.low_links[state] = index;
            self.stack.push(state);
            self.on_stack[state] = true;

            for &destination in &self.adjacency[state] {
                if self.indices[destination].is_none() {
                    self.visit(destination);
                    self.low_links[state] = self.low_links[state].min(self.low_links[destination]);
                } else if self.on_stack[destination] {
                    self.low_links[state] = self.low_links[state]
                        .min(self.indices[destination].expect("visited destination has an index"));
                }
            }

            if self.low_links[state] == index {
                let mut component = Vec::new();
                loop {
                    let member = self.stack.pop().expect("current state remains on stack");
                    self.on_stack[member] = false;
                    component.push(member);
                    if member == state {
                        break;
                    }
                }
                component.sort_unstable();
                self.components.push(component);
            }
        }
    }

    let mut tarjan = Tarjan {
        adjacency,
        next_index: 0,
        indices: vec![None; adjacency.len()],
        low_links: vec![0; adjacency.len()],
        stack: Vec::new(),
        on_stack: vec![false; adjacency.len()],
        components: Vec::new(),
    };

    for state in 0..adjacency.len() {
        if tarjan.indices[state].is_none() {
            tarjan.visit(state);
        }
    }

    tarjan.components.sort_by_key(|component| component[0]);
    tarjan.components
}

fn component_ids(state_count: usize, components: &[Vec<usize>]) -> Vec<usize> {
    let mut ids = vec![0; state_count];
    for (component_id, component) in components.iter().enumerate() {
        for state in component {
            ids[*state] = component_id;
        }
    }
    ids
}

fn cyclic_components<'a>(
    adjacency: &[Vec<usize>],
    components: &'a [Vec<usize>],
) -> Vec<&'a Vec<usize>> {
    components
        .iter()
        .filter(|component| component.len() > 1 || adjacency[component[0]].contains(&component[0]))
        .collect()
}

fn cycle_names(definition: &MachineDefinition, component: &[usize]) -> Vec<String> {
    let mut names = component
        .iter()
        .map(|index| definition.states[*index].id.clone())
        .collect::<Vec<_>>();
    if let Some(first) = names.first().cloned() {
        names.push(first);
    }
    names
}

#[cfg(test)]
mod tests {
    use super::{
        DiagnosticCode, GraphPolicy, GraphWitness, MachineDefinition, StateCategory, Trigger,
    };

    fn direct() -> Trigger {
        Trigger::Direct
    }

    fn route(name: &str) -> Trigger {
        Trigger::Route(name.into())
    }

    fn codes(definition: MachineDefinition) -> Vec<DiagnosticCode> {
        definition.validate().unwrap_err().codes()
    }

    fn linear_machine() -> MachineDefinition {
        MachineDefinition::new()
            .state("Draft", StateCategory::Active)
            .state("Done", StateCategory::Terminal)
            .initial("Draft")
            .transition("finish", "Draft", "Done", direct())
    }

    #[test]
    fn accepts_single_terminal_state() {
        let validated = MachineDefinition::new()
            .state("Done", StateCategory::Terminal)
            .initial("Done")
            .validate()
            .expect("single terminal state is valid");

        assert_eq!(validated.definition().states.len(), 1);
    }

    #[test]
    fn accepts_linear_machine_and_terminal_policy() {
        linear_machine()
            .policy(GraphPolicy::MustReachTerminal)
            .policy(GraphPolicy::Acyclic)
            .validate()
            .expect("linear machine satisfies both policies");
    }

    #[test]
    fn validates_identity_initial_and_endpoint_failures_in_phase_order() {
        let definition = MachineDefinition::new()
            .state("A", StateCategory::Active)
            .state("A", StateCategory::Terminal)
            .initial("Missing")
            .transition("edge", "Unknown", "Elsewhere", direct())
            .transition("edge", "Unknown", "Elsewhere", direct());

        assert_eq!(
            codes(definition),
            vec![
                DiagnosticCode::Sm004,
                DiagnosticCode::Sm005,
                DiagnosticCode::Sm018,
                DiagnosticCode::Sm006,
                DiagnosticCode::Sm007,
                DiagnosticCode::Sm006,
                DiagnosticCode::Sm007,
            ]
        );
    }

    #[test]
    fn rejects_empty_missing_and_multiple_initial_definitions() {
        assert_eq!(
            codes(MachineDefinition::new()),
            vec![DiagnosticCode::Sm001, DiagnosticCode::Sm002]
        );

        assert_eq!(
            codes(MachineDefinition::new().state("A", StateCategory::Terminal)),
            vec![DiagnosticCode::Sm002]
        );

        assert_eq!(
            codes(
                MachineDefinition::new()
                    .state("A", StateCategory::Terminal)
                    .state("B", StateCategory::Terminal)
                    .initial("A")
                    .initial("B")
            ),
            vec![DiagnosticCode::Sm003]
        );
    }

    #[test]
    fn rejects_duplicate_direct_and_route_triggers() {
        let definition = MachineDefinition::new()
            .state("A", StateCategory::Active)
            .routes("A", ["left"])
            .state("B", StateCategory::Terminal)
            .initial("A")
            .transition("one", "A", "B", direct())
            .transition("two", "A", "B", direct())
            .transition("three", "A", "B", route("left"))
            .transition("four", "A", "B", route("left"));

        assert_eq!(
            codes(definition),
            vec![DiagnosticCode::Sm008, DiagnosticCode::Sm016]
        );
    }

    #[test]
    fn rejects_missing_and_undeclared_routes() {
        let definition = MachineDefinition::new()
            .state("A", StateCategory::Active)
            .routes("A", ["left", "right"])
            .state("B", StateCategory::Terminal)
            .initial("A")
            .transition("left", "A", "B", route("left"))
            .transition("other", "A", "B", route("other"));

        assert_eq!(
            codes(definition),
            vec![DiagnosticCode::Sm017, DiagnosticCode::Sm015]
        );
    }

    #[test]
    fn distinguishes_no_incoming_from_disconnected_cycles() {
        let definition = MachineDefinition::new()
            .state("Start", StateCategory::Active)
            .state("Done", StateCategory::Terminal)
            .state("Isolated", StateCategory::Active)
            .state("A", StateCategory::Active)
            .state("B", StateCategory::Active)
            .initial("Start")
            .transition("finish", "Start", "Done", direct())
            .transition("isolated-loop", "Isolated", "Isolated", direct())
            .transition("a-to-b", "A", "B", direct())
            .transition("b-to-a", "B", "A", direct());

        assert_eq!(
            codes(definition),
            vec![
                DiagnosticCode::Sm014,
                DiagnosticCode::Sm014,
                DiagnosticCode::Sm014,
            ]
        );

        let no_incoming = MachineDefinition::new()
            .state("Start", StateCategory::Active)
            .state("Done", StateCategory::Terminal)
            .state("Orphan", StateCategory::Terminal)
            .initial("Start")
            .transition("finish", "Start", "Done", direct());
        assert_eq!(codes(no_incoming), vec![DiagnosticCode::Sm009]);
    }

    #[test]
    fn rejects_active_terminal_and_absorbing_shape_violations() {
        let active_dead_end = MachineDefinition::new()
            .state("A", StateCategory::Active)
            .initial("A");
        assert_eq!(codes(active_dead_end), vec![DiagnosticCode::Sm010]);

        let terminal_escape = MachineDefinition::new()
            .state("Done", StateCategory::Terminal)
            .state("Other", StateCategory::Terminal)
            .initial("Done")
            .transition("escape", "Done", "Other", direct());
        assert_eq!(codes(terminal_escape), vec![DiagnosticCode::Sm011]);

        let absorbing_escape = MachineDefinition::new()
            .state("Start", StateCategory::Active)
            .state("Stopped", StateCategory::Absorbing)
            .state("Other", StateCategory::Terminal)
            .initial("Start")
            .transition("stop", "Start", "Stopped", direct())
            .transition("escape", "Stopped", "Other", direct());
        assert_eq!(
            codes(absorbing_escape),
            vec![DiagnosticCode::Sm012, DiagnosticCode::Sm013]
        );

        let absorbing_dead_end = MachineDefinition::new()
            .state("Stopped", StateCategory::Absorbing)
            .initial("Stopped");
        assert_eq!(codes(absorbing_dead_end), vec![DiagnosticCode::Sm013]);
    }

    #[test]
    fn accepts_persistent_and_absorbing_machines() {
        MachineDefinition::new()
            .state("Running", StateCategory::Active)
            .initial("Running")
            .transition("tick", "Running", "Running", direct())
            .policy(GraphPolicy::Persistent)
            .validate()
            .expect("persistent self-loop is valid");

        MachineDefinition::new()
            .state("Running", StateCategory::Active)
            .state("Cancelled", StateCategory::Absorbing)
            .initial("Running")
            .transition("cancel", "Running", "Cancelled", direct())
            .transition("observe", "Cancelled", "Cancelled", direct())
            .validate()
            .expect("absorbing self-loop is valid");
    }

    #[test]
    fn validates_terminal_reachability_policy() {
        let no_terminal = MachineDefinition::new()
            .state("Running", StateCategory::Active)
            .initial("Running")
            .transition("tick", "Running", "Running", direct())
            .policy(GraphPolicy::MustReachTerminal);
        assert_eq!(codes(no_terminal), vec![DiagnosticCode::Sm101]);
    }

    #[test]
    fn must_reach_terminal_reports_closed_reachable_component() {
        let definition = MachineDefinition::new()
            .state("Start", StateCategory::Active)
            .routes("Start", ["finish", "loop"])
            .state("Loop", StateCategory::Active)
            .state("Done", StateCategory::Terminal)
            .initial("Start")
            .transition("finish", "Start", "Done", route("finish"))
            .transition("enter-loop", "Start", "Loop", route("loop"))
            .transition("stay", "Loop", "Loop", direct())
            .policy(GraphPolicy::MustReachTerminal);

        let report = definition.validate().unwrap_err();
        assert_eq!(report.codes(), vec![DiagnosticCode::Sm102]);
        assert!(matches!(
            report.diagnostics()[0].witness,
            Some(GraphWitness::States(_))
        ));
    }

    #[test]
    fn validates_acyclic_and_explicit_cycle_policies() {
        let cycle = || {
            MachineDefinition::new()
                .state("A", StateCategory::Active)
                .state("B", StateCategory::Active)
                .initial("A")
                .transition("a-to-b", "A", "B", direct())
                .transition("b-to-a", "B", "A", direct())
        };

        assert_eq!(
            codes(cycle().policy(GraphPolicy::Acyclic)),
            vec![DiagnosticCode::Sm103]
        );
        assert_eq!(
            codes(cycle().policy(GraphPolicy::CyclesExplicit)),
            vec![DiagnosticCode::Sm104, DiagnosticCode::Sm104]
        );

        MachineDefinition::new()
            .state("A", StateCategory::Active)
            .state("B", StateCategory::Active)
            .initial("A")
            .acknowledged_transition("a-to-b", "A", "B", direct())
            .acknowledged_transition("b-to-a", "B", "A", direct())
            .policy(GraphPolicy::CyclesExplicit)
            .validate()
            .expect("fully acknowledged cycle is valid");
    }

    #[test]
    fn rejects_contradictory_policies() {
        assert_eq!(
            codes(
                linear_machine()
                    .policy(GraphPolicy::Persistent)
                    .policy(GraphPolicy::MustReachTerminal)
            ),
            vec![DiagnosticCode::Sm105]
        );
    }

    #[test]
    fn diagnostics_expose_stable_codes_and_locations() {
        let report = MachineDefinition::new()
            .state("A", StateCategory::Active)
            .initial("A")
            .validate()
            .unwrap_err();
        let diagnostic = &report.diagnostics()[0];
        assert_eq!(diagnostic.code.as_str(), "SM010");
        assert_eq!(diagnostic.primary.kind, super::DeclarationKind::State);
        assert!(!diagnostic.message.is_empty());
    }
}
