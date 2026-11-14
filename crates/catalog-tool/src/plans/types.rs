#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PaginationReason {
    TokenBothSides,

    TokenInputOnly,

    TokenOutputOnly,

    TruncationFlag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Pagination<'a> {
    Modeled {
        input_token: &'a str,

        output_token: &'a str,

        items: Option<&'a str>,

        page_size: Option<&'a str>,
    },

    Unpaginated,

    NeedsOverlay(PaginationReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LargestPageSize {
    Known(u32),

    Unbounded,
}
