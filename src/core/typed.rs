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
//! What it does guarantee is narrower and still useful: phase-incompatible nodes,
//! transitions, and branch handlers are rejected at compile time when callers stay on
//! the typed workflow API. Route coverage and finish coverage are still validated at
//! runtime by the branch builder.
//!
//! # Example
//! ```rust
//! use orichalcum::typed::{
//!     BranchBuildError, BranchExecuteError, Flow, FlowState, Next, StateNode, Transition,
//! };
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
//! #[derive(Debug)]
//! enum ReviewWorkflowError {
//!     Review(ReviewError),
//!     Build(BranchBuildError<ReviewDecision>),
//!     Execute(BranchExecuteError<ReviewDecision, ReviewError>),
//! }
//!
//! impl From<ReviewError> for ReviewWorkflowError {
//!     fn from(value: ReviewError) -> Self {
//!         Self::Review(value)
//!     }
//! }
//!
//! impl From<BranchBuildError<ReviewDecision>> for ReviewWorkflowError {
//!     fn from(value: BranchBuildError<ReviewDecision>) -> Self {
//!         Self::Build(value)
//!     }
//! }
//!
//! impl From<BranchExecuteError<ReviewDecision, ReviewError>> for ReviewWorkflowError {
//!     fn from(value: BranchExecuteError<ReviewDecision, ReviewError>) -> Self {
//!         Self::Execute(value)
//!     }
//! }
//!
//! struct SubmitForReview;
//!
//! impl Transition<Draft, ReviewData> for SubmitForReview {
//!     type NextPhase = Review;
//!     type Error = ReviewError;
//!
//!     fn advance(&self, state: &mut FlowState<Draft, ReviewData>) -> Result<(), Self::Error> {
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
//! struct RequestChanges;
//!
//! impl Transition<Review, ReviewData> for RequestChanges {
//!     type NextPhase = Draft;
//!     type Error = ReviewError;
//!
//!     fn advance(
//!         &self,
//!         state: &mut FlowState<Review, ReviewData>,
//!     ) -> Result<(), Self::Error> {
//!         state.data_mut().approved = false;
//!         Ok(())
//!     }
//! }
//!
//! enum ReviewOutcome {
//!     Approved(Flow<Approved, ReviewData>),
//!     Draft(Flow<Draft, ReviewData>),
//!     StillInReview(Flow<Review, ReviewData>),
//! }
//!
//! let review_flow = Flow::<Draft, _>::new(ReviewData {
//!     document: "Ship it".into(),
//!     approved: false,
//! })
//! .transition(SubmitForReview)?;
//!
//! let outcome: ReviewOutcome = review_flow
//!     .step(&ReviewNode)?
//!     .branch::<ReviewOutcome>()
//!     .on(ReviewDecision::Approve, Approve, ReviewOutcome::Approved)?
//!     .on(
//!         ReviewDecision::RequestChanges,
//!         RequestChanges,
//!         ReviewOutcome::Draft,
//!     )?
//!     .on_finish(ReviewOutcome::StillInReview)?
//!     .finish()?;
//!
//! match outcome {
//!     ReviewOutcome::Approved(flow) => assert!(flow.data().approved),
//!     ReviewOutcome::Draft(_) => unreachable!("example review always approves"),
//!     ReviewOutcome::StillInReview(_) => unreachable!("review node should route"),
//! }
//! # Ok::<(), ReviewWorkflowError>(())
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
//!     fn advance(&self, _state: &mut FlowState<Review, Data>) -> Result<(), Self::Error> {
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
//!
//! The current branch builder deliberately trades some runtime overhead for a simpler,
//! honest API: registered branch arms are stored as boxed erased executors and matched
//! linearly at runtime. That cost buys framework-owned branch wiring without forcing a
//! type-level heterogeneous registry into the public API.
//!
//! ```compile_fail
//! use orichalcum::typed::{Flow, FlowState, Next, StateNode, Transition};
//!
//! struct Draft;
//! struct Review;
//! struct Approved;
//! struct Data;
//! struct ReviewNode;
//! struct Approve;
//! #[derive(Clone, Debug, PartialEq, Eq)]
//! enum Decision { Approve }
//! enum Outcome { Approved(Flow<Approved, Data>) }
//! #[derive(Debug)]
//! struct Error;
//!
//! impl StateNode<Draft, Data> for ReviewNode {
//!     type Route = Decision;
//!     type Error = Error;
//!
//!     fn run(
//!         &self,
//!         _state: &mut FlowState<Draft, Data>,
//!     ) -> Result<Next<Self::Route>, Self::Error> {
//!         Ok(Next::Route(Decision::Approve))
//!     }
//! }
//!
//! impl Transition<Review, Data> for Approve {
//!     type NextPhase = Approved;
//!     type Error = Error;
//!
//!     fn advance(&self, _state: &mut FlowState<Review, Data>) -> Result<(), Self::Error> {
//!         Ok(())
//!     }
//! }
//!
//! let decision = Flow::<Draft, _>::new(Data).step(&ReviewNode).unwrap();
//! let _ = decision
//!     .branch::<Outcome>()
//!     .on(Decision::Approve, Approve, Outcome::Approved);
//! ```
//!
//! The compile failures above are intentional: `Approve` and `ReviewNode` are only
//! available once the workflow has entered the right phase, raw phase relabeling is not
//! part of the public typed API, and branch handlers must also use transitions legal for
//! the current phase. Missing route handlers and missing finish handlers remain runtime
//! validation errors, not compile-time proofs.
//!
//! These APIs are additive today. The existing dynamic workflow engine remains
//! available for cases where runtime-defined state or graph structure is still the
//! right tradeoff.
//!
//! This module is the beginning of a stronger typed-workflow story, not the end of it.

use std::convert::Infallible;
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

type BranchExecutor<P, D, O, E> = Box<dyn FnOnce(Flow<P, D>) -> Result<O, E> + 'static>;
type FinishExecutor<P, D, O> = Box<dyn FnOnce(Flow<P, D>) -> O + 'static>;

struct RegisteredArm<R, P, D, O, E> {
    route: R,
    execute: BranchExecutor<P, D, O, E>,
}

/// Error returned while building a typed branch resolver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchBuildError<R> {
    /// The same route was registered more than once on one branch builder.
    DuplicateRoute(R),
    /// Finish handling was configured more than once.
    FinishAlreadyConfigured,
}

/// Error returned while executing a typed branch resolver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchExecuteError<R, E> {
    /// The produced route had no registered handler.
    UnhandledRoute(R),
    /// The branch resolved to `Next::Finish` but no finish handler was configured.
    FinishNotHandled,
    /// The selected transition failed.
    Transition(E),
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

    /// Start framework-owned typed branch resolution that will wrap the final result in `O`.
    pub fn branch<O>(self) -> BranchBuilder<P, D, R, O> {
        BranchBuilder {
            flow: Flow::from_state(self.state),
            next: self.next,
            finish_handler: None,
        }
    }

    /// Advanced escape hatch for custom branch resolution logic.
    ///
    /// Prefer `branch::<O>().on(...).finish()` for normal typed workflows so the
    /// framework owns route-to-transition wiring.
    pub fn resolve<T, Error>(
        self,
        resolver: impl FnOnce(Flow<P, D>, Next<R>) -> Result<T, Error>,
    ) -> Result<T, Error> {
        resolver(Flow::from_state(self.state), self.next)
    }
}

/// Builder for framework-owned typed branch resolution before any route handlers are registered.
pub struct BranchBuilder<P, D, R, O> {
    flow: Flow<P, D>,
    next: Next<R>,
    finish_handler: Option<FinishExecutor<P, D, O>>,
}

/// Builder for framework-owned typed branch resolution after the transition error type is known.
pub struct ConfiguredBranchBuilder<P, D, R, O, E> {
    flow: Flow<P, D>,
    next: Next<R>,
    route_handlers: Vec<RegisteredArm<R, P, D, O, E>>,
    finish_handler: Option<FinishExecutor<P, D, O>>,
}

impl<P, D, R, O> BranchBuilder<P, D, R, O>
where
    R: PartialEq + Clone + fmt::Debug,
{
    /// Register the first route handler and establish the branch error type `E`.
    pub fn on<T, F>(
        self,
        route: R,
        transition: T,
        outcome_ctor: F,
    ) -> Result<ConfiguredBranchBuilder<P, D, R, O, T::Error>, BranchBuildError<R>>
    where
        T: Transition<P, D> + 'static,
        F: FnOnce(Flow<T::NextPhase, D>) -> O + 'static,
    {
        let BranchBuilder {
            flow,
            next,
            finish_handler,
        } = self;

        let execute: BranchExecutor<P, D, O, T::Error> = Box::new(move |flow| {
            flow.transition(transition).map(outcome_ctor)
        });

        Ok(ConfiguredBranchBuilder {
            flow,
            next,
            route_handlers: vec![RegisteredArm { route, execute }],
            finish_handler,
        })
    }

    /// Register explicit finish handling before any route handlers exist.
    pub fn on_finish<F>(mut self, outcome_ctor: F) -> Result<Self, BranchBuildError<R>>
    where
        F: FnOnce(Flow<P, D>) -> O + 'static,
    {
        if self.finish_handler.is_some() {
            return Err(BranchBuildError::FinishAlreadyConfigured);
        }

        self.finish_handler = Some(Box::new(outcome_ctor));
        Ok(self)
    }

    /// Execute a branch builder that has only finish handling configured.
    pub fn finish(self) -> Result<O, BranchExecuteError<R, Infallible>> {
        let BranchBuilder {
            flow,
            next,
            finish_handler,
        } = self;

        match next {
            Next::Route(route) => Err(BranchExecuteError::UnhandledRoute(route)),
            Next::Finish => finish_handler
                .map(|handler| handler(flow))
                .ok_or(BranchExecuteError::FinishNotHandled),
        }
    }
}

impl<P, D, R, O, E> ConfiguredBranchBuilder<P, D, R, O, E>
where
    R: PartialEq + Clone + fmt::Debug,
{
    /// Register a route handler using a transition legal from the current phase.
    pub fn on<T, F>(
        mut self,
        route: R,
        transition: T,
        outcome_ctor: F,
    ) -> Result<Self, BranchBuildError<R>>
    where
        T: Transition<P, D, Error = E> + 'static,
        F: FnOnce(Flow<T::NextPhase, D>) -> O + 'static,
    {
        if self.route_handlers.iter().any(|arm| arm.route == route) {
            return Err(BranchBuildError::DuplicateRoute(route.clone()));
        }

        let execute: BranchExecutor<P, D, O, E> = Box::new(move |flow| {
            flow.transition(transition).map(outcome_ctor)
        });

        self.route_handlers.push(RegisteredArm { route, execute });
        Ok(self)
    }

    /// Register explicit handling for `Next::Finish`.
    pub fn on_finish<F>(mut self, outcome_ctor: F) -> Result<Self, BranchBuildError<R>>
    where
        F: FnOnce(Flow<P, D>) -> O + 'static,
    {
        if self.finish_handler.is_some() {
            return Err(BranchBuildError::FinishAlreadyConfigured);
        }

        self.finish_handler = Some(Box::new(outcome_ctor));
        Ok(self)
    }

    /// Resolve the already-produced branch decision through framework-owned wiring.
    pub fn finish(self) -> Result<O, BranchExecuteError<R, E>> {
        let ConfiguredBranchBuilder {
            flow,
            next,
            route_handlers,
            finish_handler,
        } = self;

        match next {
            Next::Route(route) => {
                for arm in route_handlers {
                    if arm.route == route {
                        return (arm.execute)(flow).map_err(BranchExecuteError::Transition);
                    }
                }

                Err(BranchExecuteError::UnhandledRoute(route))
            }
            Next::Finish => finish_handler
                .map(|handler| handler(flow))
                .ok_or(BranchExecuteError::FinishNotHandled),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BranchBuildError, BranchExecuteError, Branch, Flow, FlowState, Next, StateNode,
        Transition,
    };

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

        fn advance(&self, state: &mut FlowState<Draft, DocumentData>) -> Result<(), Self::Error> {
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

    #[allow(dead_code)]
    #[derive(Debug)]
    enum ReviewOutcome {
        Draft(Flow<Draft, DocumentData>),
        Approved(Flow<Approved, DocumentData>),
        StillInReview(Flow<Review, DocumentData>),
    }

    #[test]
    fn typed_flow_supports_framework_owned_branching() {
        let review_flow = Flow::<Draft, _>::new(DocumentData {
            content: "Publishable draft".into(),
            reviewer_notes: vec!["looks good".into()],
            approved: false,
        })
        .transition(SubmitForReview)
        .expect("non-empty draft should enter review");

        let approved = review_flow
            .step(&ReviewNode)
            .expect("review with notes should succeed")
            .branch::<ReviewOutcome>()
            .on(ReviewDecision::Approve, ApproveTransition, ReviewOutcome::Approved)
            .expect("approve route should register")
            .on(
                ReviewDecision::RequestChanges,
                RequestChangesTransition,
                ReviewOutcome::Draft,
            )
            .expect("request changes route should register")
            .on_finish(ReviewOutcome::StillInReview)
            .expect("finish handler should register")
            .finish()
            .expect("approve route should execute");

        match approved {
            ReviewOutcome::Approved(flow) => {
                assert!(flow.data().approved);
                assert_eq!(flow.data().content, "Publishable draft");
            }
            ReviewOutcome::Draft(_) => panic!("expected approval outcome"),
            ReviewOutcome::StillInReview(_) => panic!("expected routed outcome"),
        }
    }

    #[test]
    fn duplicate_route_registration_is_rejected() {
        let branch = Branch {
            state: FlowState::<Review, _>::new(DocumentData {
                content: "Publishable draft".into(),
                reviewer_notes: vec!["looks good".into()],
                approved: false,
            }),
            next: Next::Route(ReviewDecision::Approve),
        };

        let duplicate = branch
            .branch::<ReviewOutcome>()
            .on(ReviewDecision::Approve, ApproveTransition, ReviewOutcome::Approved)
            .expect("first registration should succeed")
            .on(ReviewDecision::Approve, ApproveTransition, ReviewOutcome::Approved);

        assert!(matches!(
            duplicate,
            Err(BranchBuildError::DuplicateRoute(ReviewDecision::Approve))
        ));
    }

    #[test]
    fn finish_without_handler_is_rejected() {
        let branch = Branch {
            state: FlowState::<Review, _>::new(DocumentData {
                content: "Publishable draft".into(),
                reviewer_notes: vec!["looks good".into()],
                approved: false,
            }),
            next: Next::<ReviewDecision>::Finish,
        };

        let finish = branch.branch::<ReviewOutcome>().finish();

        assert!(matches!(finish, Err(BranchExecuteError::FinishNotHandled)));
    }
}
