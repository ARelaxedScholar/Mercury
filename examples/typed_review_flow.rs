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

#[allow(dead_code)]
#[derive(Debug)]
enum ReviewWorkflowError {
    Review(ReviewError),
    Build(BranchBuildError<ReviewDecision>),
    Execute(BranchExecuteError<ReviewDecision, ReviewError>),
}

impl From<ReviewError> for ReviewWorkflowError {
    fn from(value: ReviewError) -> Self {
        Self::Review(value)
    }
}

impl From<BranchBuildError<ReviewDecision>> for ReviewWorkflowError {
    fn from(value: BranchBuildError<ReviewDecision>) -> Self {
        Self::Build(value)
    }
}

impl From<BranchExecuteError<ReviewDecision, ReviewError>> for ReviewWorkflowError {
    fn from(value: BranchExecuteError<ReviewDecision, ReviewError>) -> Self {
        Self::Execute(value)
    }
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

enum ReviewOutcome {
    Approved(Flow<Approved, DocumentData>),
    Draft(Flow<Draft, DocumentData>),
    StillInReview(Flow<Review, DocumentData>),
}

fn main() -> Result<(), ReviewWorkflowError> {
    let review_flow = Flow::<Draft, _>::new(DocumentData {
        content: "Typed workflows now let Orichalcum own route-to-transition branch wiring.".into(),
        reviewer_notes: vec!["looks good".into()],
        approved: false,
    })
    .transition(SubmitForReview)?;

    let outcome: ReviewOutcome = review_flow
        .step(&ReviewNode)?
        .branch::<ReviewOutcome>()
        .on(ReviewDecision::Approve, ApproveTransition, ReviewOutcome::Approved)?
        .on(
            ReviewDecision::RequestChanges,
            RequestChangesTransition,
            ReviewOutcome::Draft,
        )?
        .on_finish(ReviewOutcome::StillInReview)?
        .finish()?;

    match outcome {
        ReviewOutcome::Approved(flow) => {
            println!(
                "Approved: {} | approved={} | notes={:?}",
                flow.data().content,
                flow.data().approved,
                flow.data().reviewer_notes
            );
        }
        ReviewOutcome::Draft(flow) => {
            println!(
                "Sent back to draft: {} | approved={} | notes={:?}",
                flow.data().content,
                flow.data().approved,
                flow.data().reviewer_notes
            );
        }
        ReviewOutcome::StillInReview(flow) => {
            println!(
                "Still in review: {} | approved={} | notes={:?}",
                flow.data().content,
                flow.data().approved,
                flow.data().reviewer_notes
            );
        }
    }

    Ok(())
}
