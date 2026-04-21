use orichalcum::typed::{Flow, FlowState, Next, StateNode, Transition};

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

    fn advance(
        &self,
        state: &mut FlowState<Draft, TicketData>,
    ) -> Result<(), Self::Error> {
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

#[test]
fn typed_workflow_routes_to_approved_phase() {
    let review_flow = Flow::<Draft, _>::new(TicketData {
        body: "Ship the typed workflow API".into(),
        reviewer_notes: vec!["looks good".into()],
        approved: false,
    })
    .transition(SubmitForReview)
    .expect("non-empty body should enter review");

    let approved = review_flow
        .step(&ReviewNode)
        .expect("review should succeed")
        .resolve(|flow, next| match next {
            Next::Route(ReviewDecision::Approve) => flow.transition(ApproveTransition),
            Next::Route(ReviewDecision::RequestChanges) => {
                panic!("unexpected request-changes route")
            }
            Next::Finish => panic!("unexpected finish route"),
        })
        .expect("approve path should succeed");

    assert!(approved.data().approved);
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
fn typed_branch_finish_is_preserved() {
    let review_flow = Flow::<Draft, _>::new(TicketData {
        body: "No-op review".into(),
        reviewer_notes: vec!["looks good".into()],
        approved: false,
    })
    .transition(SubmitForReview)
    .expect("non-empty body should enter review");

    let still_in_review = review_flow
        .step(&NoopNode)
        .expect("noop node should succeed")
        .resolve(|flow, next| match next {
            Next::Finish => Ok::<_, ()>(flow),
            Next::Route(route) => panic!("unexpected route: {route:?}"),
        })
        .expect("finish should return the current flow");

    assert!(!still_in_review.data().approved);
}
