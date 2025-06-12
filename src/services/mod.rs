pub mod board_service;
pub mod vote_service;

// Re-export BoardService for easier access
pub use board_service::BoardService;
pub use vote_service::VoteService;
