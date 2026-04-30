use orichalcum::typed::{
    BranchBuildError, BranchExecuteError, Flow, FlowState, Next, StateNode, Transition,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Draft;
#[derive(Debug, Clone, PartialEq, Eq)]
struct Review;
#[derive(Debug, Clone, PartialEq, Eq)]
struct Approved;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TicketData {
    body: String,
    reviewer_notes: Vec<String>,
    approved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReviewDecision {
    Approve,
    RequestChanges,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WorkflowError {
    EmptyBody,
    MissingNotes,
}

struct SubmitForReview;

impl Transition<Draft, TicketData> for SubmitForReview {
    type NextPhase = Review;
    type Error = WorkflowError;

    fn advance(&self, state: &mut FlowState<Draft, TicketData>) -> Result<(), Self::Error> {
        if state.data().body.trim().is_empty() {
            return Err(WorkflowError::EmptyBody);
        }

        Ok(())
    }
}

struct ReviewNode;

impl StateNode<Review, TicketData> for ReviewNode {
    type Route = ReviewDecision;
    type Error = WorkflowError;

    fn run(
        &self,
        state: &mut FlowState<Review, TicketData>,
    ) -> Result<Next<Self::Route>, Self::Error> {
        if state.data().reviewer_notes.is_empty() {
            return Err(WorkflowError::MissingNotes);
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

struct NoopNode;

impl StateNode<Review, TicketData> for NoopNode {
    type Route = ReviewDecision;
    type Error = WorkflowError;

    fn run(
        &self,
        _state: &mut FlowState<Review, TicketData>,
    ) -> Result<Next<Self::Route>, Self::Error> {
        Ok(Next::Finish)
    }
}

struct ApproveTransition;

impl Transition<Review, TicketData> for ApproveTransition {
    type NextPhase = Approved;
    type Error = WorkflowError;

    fn advance(
        &self,
        _state: &mut FlowState<Review, TicketData>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct RequestChangesTransition;

impl Transition<Review, TicketData> for RequestChangesTransition {
    type NextPhase = Draft;
    type Error = WorkflowError;

    fn advance(
        &self,
        state: &mut FlowState<Review, TicketData>,
    ) -> Result<(), Self::Error> {
        state.data_mut().approved = false;
        Ok(())
    }
}

struct FailingApproveTransition;

impl Transition<Review, TicketData> for FailingApproveTransition {
    type NextPhase = Approved;
    type Error = WorkflowError;

    fn advance(
        &self,
        _state: &mut FlowState<Review, TicketData>,
    ) -> Result<(), Self::Error> {
        Err(WorkflowError::MissingNotes)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum ReviewOutcome {
    Draft(Flow<Draft, TicketData>),
    Approved(Flow<Approved, TicketData>),
    StillInReview(Flow<Review, TicketData>),
}

#[test]
fn typed_workflow_routes_to_approved_phase() {
    let review_flow = Flow::<Draft, _>::new(TicketData {
        body: "Ship the typed workflow API".into(),
        reviewer_notes: vec!["looks good".into()],
        approved: false,
    })
    .transition(SubmitForReview)
    .expect("non-empty body should enter review");

    let outcome = review_flow
        .step(&ReviewNode)
        .expect("review should succeed")
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
        .expect("approve path should succeed");

    match outcome {
        ReviewOutcome::Approved(flow) => assert!(flow.data().approved),
        ReviewOutcome::Draft(_) => panic!("expected approval"),
        ReviewOutcome::StillInReview(_) => panic!("expected routed approval"),
    }
}

#[test]
fn typed_transition_rejects_invalid_data_before_phase_change() {
    let err = Flow::<Draft, _>::new(TicketData {
        body: "   ".into(),
        reviewer_notes: vec!["looks good".into()],
        approved: false,
    })
    .transition(SubmitForReview)
    .expect_err("empty body should not enter review");

    assert_eq!(err, WorkflowError::EmptyBody);
}

#[test]
fn typed_node_rejects_missing_review_notes() {
    let review_flow = Flow::<Draft, _>::new(TicketData {
        body: "Ready for review".into(),
        reviewer_notes: vec![],
        approved: false,
    })
    .transition(SubmitForReview)
    .expect("non-empty body should enter review");

    let err = review_flow
        .step(&ReviewNode)
        .expect_err("review node should reject missing notes");

    assert_eq!(err, WorkflowError::MissingNotes);
}

#[test]
fn duplicate_route_registration_is_rejected() {
    let branch = Flow::<Draft, _>::new(TicketData {
        body: "Duplicate route".into(),
        reviewer_notes: vec!["looks good".into()],
        approved: false,
    })
    .transition(SubmitForReview)
    .expect("non-empty body should enter review")
    .step(&ReviewNode)
    .expect("review should succeed");

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
fn unhandled_route_is_rejected() {
    let review_flow = Flow::<Draft, _>::new(TicketData {
        body: "Needs revision".into(),
        reviewer_notes: vec!["changes required".into()],
        approved: false,
    })
    .transition(SubmitForReview)
    .expect("non-empty body should enter review");

    let finish = review_flow
        .step(&ReviewNode)
        .expect("review should succeed")
        .branch::<ReviewOutcome>()
        .on(ReviewDecision::Approve, ApproveTransition, ReviewOutcome::Approved)
        .expect("approve route should register")
        .finish();

    assert!(matches!(
        finish,
        Err(BranchExecuteError::UnhandledRoute(
            ReviewDecision::RequestChanges,
        ))
    ));
}

#[test]
fn missing_finish_handler_is_rejected() {
    let review_flow = Flow::<Draft, _>::new(TicketData {
        body: "No-op review".into(),
        reviewer_notes: vec!["looks good".into()],
        approved: false,
    })
    .transition(SubmitForReview)
    .expect("non-empty body should enter review");

    let finish = review_flow
        .step(&NoopNode)
        .expect("noop node should succeed")
        .branch::<ReviewOutcome>()
        .on(ReviewDecision::Approve, ApproveTransition, ReviewOutcome::Approved)
        .expect("approve route should register")
        .finish();

    assert!(matches!(finish, Err(BranchExecuteError::FinishNotHandled)));
}

#[test]
fn explicit_finish_handling_is_preserved() {
    let review_flow = Flow::<Draft, _>::new(TicketData {
        body: "No-op review".into(),
        reviewer_notes: vec!["looks good".into()],
        approved: false,
    })
    .transition(SubmitForReview)
    .expect("non-empty body should enter review");

    let outcome = review_flow
        .step(&NoopNode)
        .expect("noop node should succeed")
        .branch::<ReviewOutcome>()
        .on(ReviewDecision::Approve, ApproveTransition, ReviewOutcome::Approved)
        .expect("approve route should register")
        .on_finish(ReviewOutcome::StillInReview)
        .expect("finish handler should register")
        .finish()
        .expect("finish should return the current flow");

    match outcome {
        ReviewOutcome::StillInReview(flow) => assert!(!flow.data().approved),
        ReviewOutcome::Approved(_) | ReviewOutcome::Draft(_) => {
            panic!("expected finish to preserve current flow")
        }
    }
}

#[test]
fn transition_failure_is_propagated_through_branch_builder() {
    let review_flow = Flow::<Draft, _>::new(TicketData {
        body: "Ship the typed workflow API".into(),
        reviewer_notes: vec!["looks good".into()],
        approved: false,
    })
    .transition(SubmitForReview)
    .expect("non-empty body should enter review");

    let finish = review_flow
        .step(&ReviewNode)
        .expect("review should succeed")
        .branch::<ReviewOutcome>()
        .on(
            ReviewDecision::Approve,
            FailingApproveTransition,
            ReviewOutcome::Approved,
        )
        .expect("approve route should register")
        .on_finish(ReviewOutcome::StillInReview)
        .expect("finish handler should register")
        .finish();

    assert!(matches!(
        finish,
        Err(BranchExecuteError::Transition(WorkflowError::MissingNotes))
    ));
}
