use orichalcum::prelude::*;
use orichalcum::{Client, HashMap, signature};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let client = Client::new().with_ollama();
    let signature = signature!("document -> summary, sentiment");

    let node = client
        .semantic_node()
        .signature(signature)
        .instruction("Summarize the document and analyze its sentiment.")
        .task_id("doc_processor_v1")
        .seal();

    let flow = AsyncFlow::new(node);
    let mut state = HashMap::new();
    state.insert(
        "document".to_string(),
        "Rust is a multi-paradigm, general-purpose programming language.".into(),
    );

    flow.run(&mut state).await;

    println!("Summary: {}", state.get("summary").unwrap());
    println!("Sentiment: {}", state.get("sentiment").unwrap());
}
