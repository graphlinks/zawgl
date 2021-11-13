pub mod tx_context;
pub mod tx_handler;
pub mod request_handler;
use request_handler::RequestHandler;
use tx_handler::{Scenario, TxHandler, TxStatus, needs_write_lock};

use one_graph_core::model::PropertyGraph;
use self::tx_context::TxContext;

pub struct ResultGraph {
    pub scenario: Scenario,
    pub patterns: Vec<PropertyGraph>,
}

#[derive(Debug)]
pub enum DatabaseError {
    EngineError,
    TxError,
}

pub fn handle_graph_request<'a>(tx_handler: TxHandler, graph_request_handler: RequestHandler<'a>, patterns: &Vec<PropertyGraph>, tx_context: Option<TxContext>) -> Result<Vec<ResultGraph>, DatabaseError> {
    
    let tx_lock = tx_handler.lock();
    let tx_status = tx_lock.borrow_mut().get_session_status(&tx_context);
    match tx_status {
        TxStatus::OpenNewTx(ctx) => {
            tx_lock.borrow_mut().acquire_session_lock();
            graph_request_handler.write().unwrap().open_graph_tx(ctx);
            graph_request_handler.write().unwrap().handle_graph_request_tx(patterns, ctx)
        },
        TxStatus::ContinueCurrentTx(ctx) => graph_request_handler.write().unwrap().handle_graph_request_tx(patterns, ctx),
        TxStatus::CommitCurrentTx(ctx) => { 
            let res = graph_request_handler.write().unwrap().commit_tx(ctx);
            tx_lock.borrow_mut().release_session_lock();
            res
        },
        TxStatus::WaitForCurrentTx => {
            tx_lock.borrow_mut().acquire_session_lock();
            handle_graph_request(tx_handler.clone(), graph_request_handler, patterns, tx_context)
        },
        TxStatus::NoTx => {
            if needs_write_lock(patterns) {
                graph_request_handler.write().unwrap().handle_graph_request(patterns)
            } else {
                graph_request_handler.read().unwrap().handle_graph_request(patterns)
            }
        },
    }
}