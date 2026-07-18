//! Experimental compile-time front end for Orichalcum graph definitions.
//!
//! Most users should enable Orichalcum's `experimental-graph` feature and import
//! [`experimental_state_machine!`] from the `orichalcum` crate. This implementation
//! crate is published so Cargo can resolve the procedural macro transitively; its
//! syntax and generated API remain unstable throughout Orichalcum's 0.x releases.

use std::collections::HashMap;

use orichalcum_definition::{GraphPolicy, MachineDefinition, StateCategory, Trigger};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result, Token, braced};

enum Item {
    Machine {
        name: Ident,
    },
    Initial {
        state: Ident,
    },
    State {
        category: StateCategory,
        state: Ident,
    },
    Routes {
        state: Ident,
        routes: Vec<Ident>,
    },
    Transition {
        id: Ident,
        source: Ident,
        destination: Ident,
        route: Option<Ident>,
        cycle_acknowledged: bool,
    },
    Policy {
        policy: GraphPolicy,
        name: Ident,
    },
}

impl Item {
    fn span(&self) -> Span {
        match self {
            Self::Machine { name } => name.span(),
            Self::Initial { state } | Self::State { state, .. } | Self::Routes { state, .. } => {
                state.span()
            }
            Self::Transition { id, .. } => id.span(),
            Self::Policy { name, .. } => name.span(),
        }
    }
}

struct Input {
    items: Vec<Item>,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut items = Vec::new();

        while !input.is_empty() {
            let keyword: Ident = input.parse()?;
            match keyword.to_string().as_str() {
                "machine" => {
                    let name = input.parse()?;
                    input.parse::<Token![;]>()?;
                    items.push(Item::Machine { name });
                }
                "initial" => {
                    let state = input.parse()?;
                    input.parse::<Token![;]>()?;
                    items.push(Item::Initial { state });
                }
                "active" => {
                    let state = input.parse()?;
                    input.parse::<Token![;]>()?;
                    items.push(Item::State {
                        category: StateCategory::Active,
                        state,
                    });
                }
                "terminal" => {
                    let state = input.parse()?;
                    input.parse::<Token![;]>()?;
                    items.push(Item::State {
                        category: StateCategory::Terminal,
                        state,
                    });
                }
                "absorbing" => {
                    let state = input.parse()?;
                    input.parse::<Token![;]>()?;
                    items.push(Item::State {
                        category: StateCategory::Absorbing,
                        state,
                    });
                }
                "routes" => {
                    let state = input.parse()?;
                    let content;
                    braced!(content in input);
                    let routes = content
                        .parse_terminated(Ident::parse, Token![,])?
                        .into_iter()
                        .collect();
                    input.parse::<Token![;]>()?;
                    items.push(Item::Routes { state, routes });
                }
                "transition" => {
                    let id = input.parse()?;
                    input.parse::<Token![:]>()?;
                    let source = input.parse()?;
                    input.parse::<Token![->]>()?;
                    let destination = input.parse()?;
                    let mut route = None;
                    let mut cycle_acknowledged = false;
                    while !input.peek(Token![;]) {
                        let modifier: Ident = input.parse()?;
                        match modifier.to_string().as_str() {
                            "on" if route.is_none() => route = Some(input.parse()?),
                            "cycle" if !cycle_acknowledged => cycle_acknowledged = true,
                            _ => {
                                return Err(syn::Error::new(
                                    modifier.span(),
                                    "expected `on Route`, `cycle`, or `;`",
                                ));
                            }
                        }
                    }
                    input.parse::<Token![;]>()?;
                    items.push(Item::Transition {
                        id,
                        source,
                        destination,
                        route,
                        cycle_acknowledged,
                    });
                }
                "policy" => {
                    let name: Ident = input.parse()?;
                    let policy = match name.to_string().as_str() {
                        "must_reach_terminal" => GraphPolicy::MustReachTerminal,
                        "acyclic" => GraphPolicy::Acyclic,
                        "cycles_explicit" => GraphPolicy::CyclesExplicit,
                        "persistent" => GraphPolicy::Persistent,
                        _ => {
                            return Err(syn::Error::new(name.span(), "unknown graph policy"));
                        }
                    };
                    input.parse::<Token![;]>()?;
                    items.push(Item::Policy { policy, name });
                }
                _ => {
                    return Err(syn::Error::new(
                        keyword.span(),
                        "expected a graph declaration",
                    ));
                }
            }
        }

        Ok(Self { items })
    }
}

/// Validate a minimal direct-transition graph during procedural macro expansion.
///
/// This macro is an architectural spike, not a stable public DSL.
#[proc_macro]
pub fn experimental_state_machine(tokens: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokens as Input);
    expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand(input: Input) -> Result<proc_macro2::TokenStream> {
    let mut definition = MachineDefinition::new();
    let mut spans = HashMap::new();
    let mut machine_name = None;
    let mut state_markers = Vec::new();
    let mut state_descriptors = Vec::new();
    let mut route_descriptors = Vec::new();
    let mut route_sets = Vec::new();
    let mut transition_descriptors = Vec::new();
    let mut transition_methods = Vec::new();
    let mut routed_transitions = Vec::new();
    let mut policy_descriptors = Vec::new();
    let mut initial_name = None;
    let mut initial_marker = None;

    for item in input.items {
        match &item {
            Item::Machine { name } => {
                if machine_name.replace(name.clone()).is_some() {
                    return Err(syn::Error::new(
                        name.span(),
                        "the experimental syntax accepts one `machine` declaration",
                    ));
                }
            }
            Item::Initial { state } => {
                definition = definition.initial(state.to_string());
                spans.insert(
                    definition
                        .initials
                        .last()
                        .expect("initial was inserted")
                        .declared_at
                        .ordinal,
                    item.span(),
                );
                initial_name = Some(state.to_string());
                initial_marker = Some(state.clone());
            }
            Item::State { category, state } => {
                definition = definition.state(state.to_string(), *category);
                spans.insert(
                    definition
                        .states
                        .last()
                        .expect("state was inserted")
                        .declared_at
                        .ordinal,
                    item.span(),
                );
                state_markers.push(state.clone());
                let category_name = match category {
                    StateCategory::Active => "active",
                    StateCategory::Terminal => "terminal",
                    StateCategory::Absorbing => "absorbing",
                };
                let state_name = state.to_string();
                state_descriptors.push(quote!((#state_name, #category_name)));
            }
            Item::Routes { state, routes } => {
                let state_name = state.to_string();
                if !definition
                    .states
                    .iter()
                    .any(|declaration| declaration.id == state_name)
                {
                    return Err(syn::Error::new(
                        state.span(),
                        "`routes` must follow the corresponding state declaration",
                    ));
                }
                definition = definition.routes(&state_name, routes.iter().map(Ident::to_string));
                route_sets.push((state.clone(), routes.clone()));
                let routes = routes.iter().map(Ident::to_string).collect::<Vec<_>>();
                route_descriptors.push(quote!((#state_name, &[#(#routes),*])));
            }
            Item::Transition {
                id,
                source,
                destination,
                route,
                cycle_acknowledged,
            } => {
                let trigger = route
                    .as_ref()
                    .map(|route| Trigger::Route(route.to_string()))
                    .unwrap_or(Trigger::Direct);
                definition = if *cycle_acknowledged {
                    definition.acknowledged_transition(
                        id.to_string(),
                        source.to_string(),
                        destination.to_string(),
                        trigger,
                    )
                } else {
                    definition.transition(
                        id.to_string(),
                        source.to_string(),
                        destination.to_string(),
                        trigger,
                    )
                };
                spans.insert(
                    definition
                        .transitions
                        .last()
                        .expect("transition was inserted")
                        .declared_at
                        .ordinal,
                    item.span(),
                );
                let method = id.clone();
                let source_marker = source.clone();
                let destination_marker = destination.clone();
                let id = id.to_string();
                let source = source.to_string();
                let destination = destination.to_string();
                let trigger = route
                    .as_ref()
                    .map(|route| route.to_string())
                    .unwrap_or_else(|| "direct".into());
                transition_descriptors.push(quote!((#id, #source, #destination, #trigger)));
                if let Some(route) = route {
                    routed_transitions.push((
                        source_marker,
                        route.clone(),
                        method,
                        destination_marker,
                    ));
                } else {
                    transition_methods.push(quote! {
                        impl<D> Execution<#source_marker, D> {
                            pub fn #method<E, F>(
                                mut self,
                                effect: F,
                            ) -> Result<Execution<#destination_marker, D>, ExecutionFailure<#source_marker, D, E>>
                            where
                                F: FnOnce(&mut D) -> Result<(), E>,
                            {
                                match effect(&mut self.data) {
                                    Ok(()) => Ok(Execution {
                                        data: self.data,
                                        _phase: ::core::marker::PhantomData,
                                    }),
                                    Err(error) => Err(ExecutionFailure {
                                        execution: self,
                                        error,
                                        transition: #id,
                                    }),
                                }
                            }
                        }
                    });
                }
            }
            Item::Policy { policy, name } => {
                definition = definition.policy(*policy);
                spans.insert(
                    definition
                        .policies
                        .last()
                        .expect("policy was inserted")
                        .declared_at
                        .ordinal,
                    item.span(),
                );
                policy_descriptors.push(name.to_string());
            }
        }
    }

    let machine_name = machine_name.ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "the experimental syntax requires `machine name;`",
        )
    })?;
    let initial_name = initial_name.unwrap_or_default();
    let initial_marker = initial_marker.unwrap_or_else(|| format_ident!("MissingInitial"));

    if let Err(report) = definition.validate() {
        let mut error = None;
        for diagnostic in report.diagnostics() {
            let span = spans
                .get(&diagnostic.primary.ordinal)
                .copied()
                .unwrap_or_else(Span::call_site);
            let diagnostic_error = syn::Error::new(
                span,
                format!("[{}] {}", diagnostic.code.as_str(), diagnostic.message),
            );
            if let Some(error) = &mut error {
                syn::Error::combine(error, diagnostic_error);
            } else {
                error = Some(diagnostic_error);
            }
        }
        return Err(error.expect("an invalid definition has diagnostics"));
    }

    let mut route_dispatch = Vec::new();
    for (state, routes) in route_sets {
        if routes.is_empty() {
            continue;
        }
        let route_name = format_ident!("{}Route", state);
        let outcome_name = format_ident!("{}Outcome", state);
        let mut effect_types = Vec::new();
        let mut effect_names = Vec::new();
        let mut outcome_variants = Vec::new();
        let mut dispatch_arms = Vec::new();

        for (index, route) in routes.iter().enumerate() {
            let (_, _, transition, destination) = routed_transitions
                .iter()
                .find(|(source, transition_route, _, _)| {
                    source == &state && transition_route == route
                })
                .expect("validated route has exactly one transition");
            let effect_type = format_ident!("Effect{index}");
            effect_types.push(effect_type.clone());
            effect_names.push(transition.clone());
            outcome_variants.push(quote!(#route(Execution<#destination, D>)));
            dispatch_arms.push(quote! {
                #route_name::#route => match #transition(&mut self.data) {
                    Ok(()) => Ok(#outcome_name::#route(Execution {
                        data: self.data,
                        _phase: ::core::marker::PhantomData,
                    })),
                    Err(error) => Err(ExecutionFailure {
                        execution: self,
                        error,
                        transition: stringify!(#transition),
                    }),
                }
            });
        }

        route_dispatch.push(quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum #route_name {
                #(#routes),*
            }

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum #outcome_name<D> {
                #(#outcome_variants),*
            }

            impl<D> Execution<#state, D> {
                pub fn dispatch<E, #(#effect_types),*>(
                    mut self,
                    route: #route_name,
                    #(#effect_names: #effect_types),*
                ) -> Result<#outcome_name<D>, ExecutionFailure<#state, D, E>>
                where
                    #(#effect_types: FnOnce(&mut D) -> Result<(), E>),*
                {
                    match route {
                        #(#dispatch_arms),*
                    }
                }
            }
        });
    }

    let descriptor_name = format_ident!("Definition");
    Ok(quote! {
        pub mod #machine_name {
            #(#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct #state_markers;)*

            pub struct #descriptor_name;

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct Execution<P, D> {
                data: D,
                _phase: ::core::marker::PhantomData<fn() -> P>,
            }

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct ExecutionFailure<P, D, E> {
                execution: Execution<P, D>,
                error: E,
                transition: &'static str,
            }

            impl<P, D, E> ExecutionFailure<P, D, E> {
                pub fn execution(&self) -> &Execution<P, D> {
                    &self.execution
                }

                pub fn error(&self) -> &E {
                    &self.error
                }

                pub fn transition(&self) -> &'static str {
                    self.transition
                }

                pub fn into_parts(self) -> (Execution<P, D>, E) {
                    (self.execution, self.error)
                }
            }

            impl<P, D> Execution<P, D> {
                pub fn data(&self) -> &D {
                    &self.data
                }

                pub fn data_mut(&mut self) -> &mut D {
                    &mut self.data
                }

                pub fn into_data(self) -> D {
                    self.data
                }
            }

            impl #descriptor_name {
                pub const STATES: &'static [(&'static str, &'static str)] = &[
                    #(#state_descriptors),*
                ];
                pub const INITIAL: &'static str = #initial_name;
                pub const ROUTES: &'static [
                    (&'static str, &'static [&'static str])
                ] = &[#(#route_descriptors),*];
                pub const TRANSITIONS: &'static [
                    (&'static str, &'static str, &'static str, &'static str)
                ] = &[#(#transition_descriptors),*];
                pub const POLICIES: &'static [&'static str] = &[#(#policy_descriptors),*];

                pub fn start<D>(data: D) -> Execution<#initial_marker, D> {
                    Execution {
                        data,
                        _phase: ::core::marker::PhantomData,
                    }
                }
            }

            #(#transition_methods)*
            #(#route_dispatch)*
        }
    })
}
