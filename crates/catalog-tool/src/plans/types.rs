/// Why an apparently paginated operation must await overlay review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PaginationReason {
    /// Tokens matched both structures.
    TokenBothSides,
    /// Only an input token matched.
    TokenInputOnly,
    /// Only an output token matched.
    TokenOutputOnly,
    /// No token matched, but a boolean advertises more results.
    TruncationFlag,
}

/// Pure pagination evidence; paths borrow the parsed model.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Pagination<'a> {
    /// Validated merged paginator; optional fields remain model facts.
    Modeled {
        /// Input structure token member.
        input_token: &'a str,
        /// Output token path, possibly dotted.
        output_token: &'a str,
        /// Optional result items path.
        items: Option<&'a str>,
        /// Optional request page-size member.
        page_size: Option<&'a str>,
    },
    /// No modeled paginator or pagination vocabulary/flag evidence.
    Unpaginated,
    /// Possible truncation; never assume this is unpaginated.
    NeedsOverlay(PaginationReason),
}

/// Largest supported request page size, not an inferred default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LargestPageSize {
    /// Explicit, validated range maximum.
    Known(u32),
    /// Neither member nor target specifies a maximum.
    Unbounded,
}
