use catalog_tool::classes::{
    Class, DenyList, DenyReason, OperationFacts, ParseErrorKind, classify,
};

const INVALID: &[(&str, usize, ParseErrorKind)] = &[
    ("GetZ\tsecret\nGetA\tcontent\n", 2, ParseErrorKind::Unsorted),
    (
        "GetA\tsecret\nGetA\tcontent\n",
        2,
        ParseErrorKind::Duplicate,
    ),
    ("# comment\n\n", 2, ParseErrorKind::BlankLine),
    ("GetA secret\n", 1, ParseErrorKind::MissingTab),
    ("GetA\tunknown\n", 1, ParseErrorKind::UnknownReason),
    (
        "# comment\nGetA\tsecret",
        2,
        ParseErrorKind::MissingFinalNewline,
    ),
    (
        "# comment\nGetA\tsecret\r\n",
        2,
        ParseErrorKind::CarriageReturn,
    ),
    ("/GetA\tsecret\n", 1, ParseErrorKind::InvalidKey),
    ("svc//GetA\tsecret\n", 1, ParseErrorKind::InvalidKey),
    ("GetA\tsecret\textra\n", 1, ParseErrorKind::UnknownReason),
];

#[test]
fn parser_rejects_each_noncanonical_form_with_its_line() {
    INVALID.iter().for_each(|(text, line, kind)| {
        let error = DenyList::parse(text).err();
        assert_eq!(
            error.as_ref().map(|e| (e.line, e.kind)),
            Some((*line, *kind))
        );
    });
}

#[test]
fn scoped_denial_matches_only_its_service() -> Result<(), Box<dyn std::error::Error>> {
    let deny = DenyList::parse("alpha/ListThings\tcontent\n")?;
    let facts = OperationFacts {
        service: "alpha",
        name: "ListThings",
        has_required_input: false,
        readonly: true,
    };
    assert_eq!(classify(&facts, &deny), Class::Denied(DenyReason::Content));
    assert_eq!(
        classify(
            &OperationFacts {
                service: "beta",
                ..facts
            },
            &deny
        ),
        Class::Enumerate
    );
    Ok(())
}

#[test]
fn scoped_reason_wins_over_global_but_nonread_is_always_mutating()
-> Result<(), Box<dyn std::error::Error>> {
    let deny = DenyList::parse("GetA\tcontent\nPutA\tsecret\nsvc/GetA\tsecret\n")?;
    let facts = OperationFacts {
        service: "svc",
        name: "GetA",
        has_required_input: false,
        readonly: false,
    };
    assert_eq!(classify(&facts, &deny), Class::Denied(DenyReason::Secret));
    assert_eq!(
        classify(
            &OperationFacts {
                name: "PutA",
                ..facts
            },
            &deny
        ),
        Class::Denied(DenyReason::Mutating)
    );
    Ok(())
}

#[test]
fn empty_and_comment_only_lists_are_valid() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(DenyList::parse("")?.len(), 0);
    assert_eq!(DenyList::parse("# explanation\n")?.len(), 0);
    assert!(DenyList::parse("")?.is_empty());
    Ok(())
}

#[test]
fn parser_rejects_oversized_bytes_at_line_one() -> Result<(), Box<dyn std::error::Error>> {
    let exact = format!("#{}\n", "x".repeat(1_048_574));
    assert_eq!(DenyList::parse(&exact)?.len(), 0);
    let excess = "#".repeat(1_048_577);
    assert_eq!(
        DenyList::parse(&excess).err().map(|e| (e.line, e.kind)),
        Some((1, ParseErrorKind::Limit))
    );
    Ok(())
}

#[test]
fn parser_bounds_entry_count() -> Result<(), Box<dyn std::error::Error>> {
    let exact = (0..4096)
        .map(|i| format!("Get{i:04}\tsecret\n"))
        .collect::<String>();
    assert_eq!(DenyList::parse(&exact)?.len(), 4096);
    let excess = format!("{exact}Get4096\tsecret\n");
    assert_eq!(
        DenyList::parse(&excess).err().map(|e| (e.line, e.kind)),
        Some((4097, ParseErrorKind::Limit))
    );
    Ok(())
}

#[test]
fn scoped_keys_accept_directory_case_and_smithy_underscore_identifiers()
-> Result<(), Box<dyn std::error::Error>> {
    let deny = DenyList::parse("Service_A/Get_Value\tsecret\n")?;
    let facts = OperationFacts {
        service: "Service_A",
        name: "Get_Value",
        has_required_input: false,
        readonly: false,
    };
    assert_eq!(classify(&facts, &deny), Class::Denied(DenyReason::Secret));
    assert_eq!(DenyList::parse("_Get\tsecret\n")?.len(), 1);
    Ok(())
}
