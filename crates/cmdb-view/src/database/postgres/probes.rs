use sqlx::PgPool;
use std::cell::{Cell, RefCell};

thread_local! {
    static CAPTURE: Cell<bool> = const { Cell::new(false) };
    static POOL: RefCell<Option<PgPool>> = const { RefCell::new(None) };
    static QUERIES: Cell<usize> = const { Cell::new(0) };
}

pub(super) fn capture_pool() {
    CAPTURE.set(true);
    POOL.with_borrow_mut(|pool| *pool = None);
}

pub(super) fn record_pool(pool: &PgPool) {
    if CAPTURE.get() {
        POOL.with_borrow_mut(|captured| *captured = Some(pool.clone()));
    }
}

pub(super) fn take_pool() -> Option<PgPool> {
    CAPTURE.set(false);
    POOL.with_borrow_mut(Option::take)
}

pub(super) fn query() {
    QUERIES.set(QUERIES.get().saturating_add(1));
}

pub(super) fn reset_queries() {
    QUERIES.set(0);
}

pub(super) fn query_count() -> usize {
    QUERIES.get()
}
