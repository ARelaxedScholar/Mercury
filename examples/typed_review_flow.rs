use orichalcum::typed::{Flow, FlowState, Next, StateNode, Transition};

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

fn main() -> Result<(), ReviewError> {
    let review_flow = Flow::<Draft, _>::new(DocumentData {
        content: "Typed workflows reject phase-incompatible transitions on the typed API path.".into(),
        reviewer_notes: vec!["looks good".into()],
        approved: false,
    })
    .transition(SubmitForReview)?;

    let decision = review_flow.step(&ReviewNode)?;

    let outcome = decision.resolve(|flow, next| match next {
        Next::Route(ReviewDecision::Approve) => flow
            .transition(ApproveTransition)
            .map(ReviewOutcome::Approved),
        Next::Route(ReviewDecision::RequestChanges) => flow
            .transition(RequestChangesTransition)
            .map(ReviewOutcome::Draft),
        Next::Finish => unreachable!("review node must choose a route"),
    })?;

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
    }

    Ok(())
}
