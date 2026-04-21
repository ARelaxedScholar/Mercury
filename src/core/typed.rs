//! Typed workflow primitives for phase-aware orchestration.
//!
//! This module introduces a typestate-friendly workflow API alongside Orichalcum's
//! existing dynamic `NodeLogic` / `Flow` model.
//!
//! The core idea is simple:
//! - `FlowState<P, D>` carries your workflow data `D` together with a phase marker `P`
//! - `StateNode<P, D>` may only run while the flow is in phase `P`
//! - `Transition<P, D>` may only advance the flow from phase `P`
//! - `Next<R>` and `Branch<P, D, R>` keep routing decisions strongly typed
//!
//! This module does **not** yet encode an entire workflow graph in the type system.
//! What it does guarantee is narrower and still useful: phase-incompatible nodes and
//! transitions are rejected at compile time when callers stay on the typed workflow API.
//!
//! # Example
//! ```rust
//! use orichalcum::typed::{Flow, FlowState, Next, StateNode, Transition};
//!
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! struct Draft;
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! struct Review;
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! struct Approved;
//!
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! struct ReviewData {
//!     document: String,
//!     approved: bool,
//! }
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! enum ReviewDecision {
//!     Approve,
//!     RequestChanges,
//! }
//!
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! enum ReviewError {
//!     EmptyDocument,
//! }
//!
//! struct SubmitForReview;
//!
//! impl Transition<Draft, ReviewData> for SubmitForReview {
//!     type NextPhase = Review;
//!     type Error = ReviewError;
//!
//!     fn advance(
//!         &self,
//!         state: &mut FlowState<Draft, ReviewData>,
//!     ) -> Result<(), Self::Error> {
//!         if state.data().document.trim().is_empty() {
//!             return Err(ReviewError::EmptyDocument);
//!         }
//!
//!         Ok(())
//!     }
//! }
//!
//! struct ReviewNode;
//!
//! impl StateNode<Review, ReviewData> for ReviewNode {
//!     type Route = ReviewDecision;
//!     type Error = ReviewError;
//!
//!     fn run(
//!         &self,
//!         state: &mut FlowState<Review, ReviewData>,
//!     ) -> Result<Next<Self::Route>, Self::Error> {
//!         state.data_mut().approved = true;
//!         Ok(Next::Route(ReviewDecision::Approve))
//!     }
//! }
//!
//! struct Approve;
//!
//! impl Transition<Review, ReviewData> for Approve {
//!     type NextPhase = Approved;
//!     type Error = ReviewError;
//!
//!     fn advance(
//!         &self,
//!         _state: &mut FlowState<Review, ReviewData>,
//!     ) -> Result<(), Self::Error> {
//!         Ok(())
//!     }
//! }
//!
//! let review_flow = Flow::<Draft, _>::new(ReviewData {
//!     document: "Ship it".into(),
//!     approved: false,
//! })
//! .transition(SubmitForReview)
//! .unwrap();
//!
//! let decision = review_flow.step(&ReviewNode).unwrap();
//!
//! let approved_flow = decision
//!     .resolve(|flow, next| match next {
//!         Next::Route(ReviewDecision::Approve) => flow.transition(Approve),
//!         Next::Route(ReviewDecision::RequestChanges) => unreachable!("example review always approves"),
//!         Next::Finish => unreachable!("review must pick a route"),
//!     })
//!     .unwrap();
//!
//! assert!(approved_flow.data().approved);
//! ```
//!
//! # Compile-time phase safety
//! ```compile_fail
//! use orichalcum::typed::{Flow, FlowState, Transition};
//!
//! struct Draft;
//! struct Review;
//! struct Approved;
//! struct Data;
//! struct Approve;
//! struct ReviewError;
//!
//! impl Transition<Review, Data> for Approve {
//!     type NextPhase = Approved;
//!     type Error = ReviewError;
//!
//!     fn advance(
//!         &self,
//!         _state: &mut FlowState<Review, Data>,
//!     ) -> Result<(), Self::Error> {
//!         Ok(())
//!     }
//! }
//!
//! let _ = Flow::<Draft, _>::new(Data).transition(Approve);
//! ```
//!
//! ```compile_fail
//! use orichalcum::typed::{Flow, FlowState, Next, StateNode};
//!
//! struct Draft;
//! struct Review;
//! struct Data;
//! struct ReviewNode;
//! struct ReviewError;
//!
//! impl StateNode<Review, Data> for ReviewNode {
//!     type Route = ();
//!     type Error = ReviewError;
//!
//!     fn run(
//!         &self,
//!         _state: &mut FlowState<Review, Data>,
//!     ) -> Result<Next<Self::Route>, Self::Error> {
//!         Ok(Next::Finish)
//!     }
//! }
//!
//! let _ = Flow::<Draft, _>::new(Data).step(&ReviewNode);
//! ```
//!
//! ```compile_fail
//! use orichalcum::typed::FlowState;
//!
//! struct Draft;
//! struct Approved;
//!
//! let state = FlowState::<Draft, _>::new(());
//! let _approved = state.transition::<Approved>();
//! ```
//!
//! The compile failures above are intentional: `Approve` and `ReviewNode` are only
//! available once the workflow has entered `Review`, and raw phase relabeling is not
//! part of the public typed API.
//!
//! These APIs are additive today. The existing dynamic workflow engine remains
//! available for cases where runtime-defined state or graph structure is still the
//! right tradeoff.
//!
//! This module is the beginning of a stronger typed-workflow story, not the end of it.

use std::fmt;
use std::marker::PhantomData;

/// The next control-flow decision produced by a typed workflow node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Next<R> {
    /// Continue via a typed route.
    Route(R),
    /// Finish execution while staying in the current phase.
    Finish,
}

/// Typed workflow state pairing domain data with a typestate phase marker.
#[derive(Clone, PartialEq, Eq)]
pub struct FlowState<P, D> {
    data: D,
    _phase: PhantomData<fn() -> P>,
}

impl<P, D> FlowState<P, D> {
    /// Create a new flow state in phase `P`.
    pub fn new(data: D) -> Self {
        Self {
            data,
            _phase: PhantomData,
        }
    }

    /// Borrow the typed workflow data.
    pub fn data(&self) -> &D {
        &self.data
    }

    /// Mutably borrow the typed workflow data.
    pub fn data_mut(&mut self) -> &mut D {
        &mut self.data
    }

    /// Consume the state and return the underlying data.
    pub fn into_data(self) -> D {
        self.data
    }

    /// Transition into a different phase without changing the underlying data.
    pub(crate) fn transition<NextPhase>(self) -> FlowState<NextPhase, D> {
        FlowState::new(self.data)
    }
}

impl<P, D: fmt::Debug> fmt::Debug for FlowState<P, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlowState")
            .field("data", &self.data)
            .field("phase", &std::any::type_name::<P>())
            .finish()
    }
}

/// Typed node API for workflows whose legal operations depend on the current phase.
pub trait StateNode<P, D> {
    /// The route chosen by this node.
    type Route;
    /// The error produced by this node.
    type Error;

    /// Run the node against the typed flow state.
    fn run(&self, state: &mut FlowState<P, D>) -> Result<Next<Self::Route>, Self::Error>;
}

/// A legal phase transition out of `P` for data `D`.
pub trait Transition<P, D> {
    /// The phase entered after this transition succeeds.
    type NextPhase;
    /// The error produced by the transition.
    type Error;

    /// Validate and/or mutate state before Orichalcum advances to `NextPhase`.
    fn advance(&self, state: &mut FlowState<P, D>) -> Result<(), Self::Error>;
}

/// A typed, phase-aware flow wrapper.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flow<P, D> {
    state: FlowState<P, D>,
}

impl<P, D> Flow<P, D> {
    /// Create a new typed flow that starts in phase `P`.
    pub fn new(data: D) -> Self {
        Self {
            state: FlowState::new(data),
        }
    }

    /// Construct a flow from an existing typed state within the trusted typed API.
    pub(crate) fn from_state(state: FlowState<P, D>) -> Self {
        Self { state }
    }

    /// Borrow the typed state.
    pub fn state(&self) -> &FlowState<P, D> {
        &self.state
    }

    /// Mutably borrow the typed state.
    pub fn state_mut(&mut self) -> &mut FlowState<P, D> {
        &mut self.state
    }

    /// Borrow the underlying workflow data.
    pub fn data(&self) -> &D {
        self.state.data()
    }

    /// Mutably borrow the underlying workflow data.
    pub fn data_mut(&mut self) -> &mut D {
        self.state.data_mut()
    }

    /// Consume the flow and return the typed state.
    pub fn into_state(self) -> FlowState<P, D> {
        self.state
    }

    /// Consume the flow and return the underlying workflow data.
    pub fn into_data(self) -> D {
        self.state.into_data()
    }

    /// Run a typed node in the current phase, producing a typed branch decision.
    pub fn step<N>(mut self, node: &N) -> Result<Branch<P, D, N::Route>, N::Error>
    where
        N: StateNode<P, D>,
    {
        let next = node.run(&mut self.state)?;
        Ok(Branch {
            state: self.state,
            next,
        })
    }

    /// Apply a legal phase transition from the current phase.
    pub fn transition<T>(mut self, transition: T) -> Result<Flow<T::NextPhase, D>, T::Error>
    where
        T: Transition<P, D>,
    {
        transition.advance(&mut self.state)?;
        Ok(Flow::from_state(self.state.transition()))
    }
}

/// A typed branch decision produced by a state node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Branch<P, D, R> {
    state: FlowState<P, D>,
    next: Next<R>,
}

impl<P, D, R> Branch<P, D, R> {
    /// Inspect the pending branch decision.
    pub fn next(&self) -> &Next<R> {
        &self.next
    }

    /// Inspect the current typed state.
    pub fn state(&self) -> &FlowState<P, D> {
        &self.state
    }

    /// Mutably inspect the current typed state before resolving the branch.
    pub fn state_mut(&mut self) -> &mut FlowState<P, D> {
        &mut self.state
    }

    /// Consume the branch and return the typed state and next decision.
    pub fn into_parts(self) -> (FlowState<P, D>, Next<R>) {
        (self.state, self.next)
    }

    /// Resolve a branch by matching the typed route and deciding what flow comes next.
    pub fn resolve<T, Error>(
        self,
        resolver: impl FnOnce(Flow<P, D>, Next<R>) -> Result<T, Error>,
    ) -> Result<T, Error> {
        resolver(Flow::from_state(self.state), self.next)
    }
}

#[cfg(test)]
mod tests {
    use super::{Flow, FlowState, Next, StateNode, Transition};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Draft;
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Review;
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Approved;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DocumentData {
        content: String,
        reviewer_notes: Vec<String>,
        approved: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ReviewDecision {
        Approve,
        RequestChanges,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum ReviewError {
        EmptyDocument,
        MissingNotes,
    }

    struct SubmitForReview;

    impl Transition<Draft, DocumentData> for SubmitForReview {
        type NextPhase = Review;
        type Error = ReviewError;

        fn advance(
            &self,
            state: &mut FlowState<Draft, DocumentData>,
        ) -> Result<(), Self::Error> {
            if state.data().content.trim().is_empty() {
                return Err(ReviewError::EmptyDocument);
            }

            Ok(())
        }
    }

    struct ReviewNode;

    impl StateNode<Review, DocumentData> for ReviewNode {
        type Route = ReviewDecision;
        type Error = ReviewError;

        fn run(
            &self,
            state: &mut FlowState<Review, DocumentData>,
        ) -> Result<Next<Self::Route>, Self::Error> {
            if state.data().reviewer_notes.is_empty() {
                return Err(ReviewError::MissingNotes);
            }

            if state
                .data()
                .reviewer_notes
                .iter()
                .any(|note| note.contains("changes"))
            {
                Ok(Next::Route(ReviewDecision::RequestChanges))
            } else {
                state.data_mut().approved = true;
                Ok(Next::Route(ReviewDecision::Approve))
            }
        }
    }

    struct ApproveTransition;

    impl Transition<Review, DocumentData> for ApproveTransition {
        type NextPhase = Approved;
        type Error = ReviewError;

        fn advance(
            &self,
            _state: &mut FlowState<Review, DocumentData>,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    struct RequestChangesTransition;

    impl Transition<Review, DocumentData> for RequestChangesTransition {
        type NextPhase = Draft;
        type Error = ReviewError;

        fn advance(
            &self,
            state: &mut FlowState<Review, DocumentData>,
        ) -> Result<(), Self::Error> {
            state.data_mut().approved = false;
            Ok(())
        }
    }

    enum ReviewOutcome {
        Draft(Flow<Draft, DocumentData>),
        Approved(Flow<Approved, DocumentData>),
    }

    #[test]
    fn typed_flow_supports_phase_legal_transitions() {
        let review_flow = Flow::<Draft, _>::new(DocumentData {
            content: "Publishable draft".into(),
            reviewer_notes: vec!["looks good".into()],
            approved: false,
        })
        .transition(SubmitForReview)
        .expect("non-empty draft should enter review");

        let decision = review_flow
            .step(&ReviewNode)
            .expect("review with notes should succeed");

        let approved = decision
            .resolve(|flow, next| match next {
                Next::Route(ReviewDecision::Approve) => {
                    flow.transition(ApproveTransition).map(ReviewOutcome::Approved)
                }
                Next::Route(ReviewDecision::RequestChanges) => flow
                    .transition(RequestChangesTransition)
                    .map(ReviewOutcome::Draft),
                Next::Finish => unreachable!("review should choose a route"),
            })
            .expect("approve route should be legal");

        match approved {
            ReviewOutcome::Approved(flow) => {
                assert!(flow.data().approved);
                assert_eq!(flow.data().content, "Publishable draft");
            }
            ReviewOutcome::Draft(_) => panic!("expected approval outcome"),
        }
    }

    #[test]
    fn typed_flow_can_branch_back_to_prior_phase() {
        let review_flow = Flow::<Draft, _>::new(DocumentData {
            content: "Needs work".into(),
            reviewer_notes: vec!["changes required".into()],
            approved: false,
        })
        .transition(SubmitForReview)
        .expect("draft should enter review");

        let decision = review_flow
            .step(&ReviewNode)
            .expect("review with notes should succeed");

        let sent_back = decision
            .resolve(|flow, next| match next {
                Next::Route(ReviewDecision::Approve) => {
                    flow.transition(ApproveTransition).map(ReviewOutcome::Approved)
                }
                Next::Route(ReviewDecision::RequestChanges) => flow
                    .transition(RequestChangesTransition)
                    .map(ReviewOutcome::Draft),
                Next::Finish => unreachable!("review should choose a route"),
            })
            .expect("request-changes route should be legal");

        match sent_back {
            ReviewOutcome::Draft(flow) => {
                assert!(!flow.data().approved);
                assert_eq!(flow.data().content, "Needs work");
            }
            ReviewOutcome::Approved(_) => panic!("expected draft outcome"),
        }
    }
}
