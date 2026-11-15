// Pinned source: .local/aws-models/models/b2bi/service/2022-06-23/b2bi-2022-06-23.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("CreateCapability", deferred("Model requires name, type, configuration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Instantiates a capability based on the specified parameters. A trading capability contains the information required to transform incoming EDI documents into JSON or XML outputs.")),
        ("CreatePartnership", deferred("Model requires profileId, name, email, capabilities. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a partnership between a customer and a trading partner, based on the supplied parameters. A partnership represents the connection between you and your trading partner. It ties together a profile and one or more trading…")),
        ("CreateProfile", deferred("Model requires name, phone, businessName, logging. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a customer profile. You can have up to five customer profiles, each representing a distinct private network. A profile is the mechanism used to create the concept of a private network.")),
        ("CreateStarterMappingTemplate", deferred("Model requires mappingType, templateDetails. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Amazon Web Services B2B Data Interchange uses a mapping template in JSONata or XSLT format to transform a customer input file into a JSON or XML file that can be converted to EDI. If you provide a sample EDI file with the same…")),
        ("CreateTransformer", deferred("Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a transformer. Amazon Web Services B2B Data Interchange currently supports two scenarios: Inbound EDI : the Amazon Web Services customer receives an EDI file from their trading partner. Amazon Web Services B2B Data…")),
        ("DeleteCapability", deferred("Model requires capabilityId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified capability. A trading capability contains the information required to transform incoming EDI documents into JSON or XML outputs.")),
        ("DeletePartnership", deferred("Model requires partnershipId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified partnership. A partnership represents the connection between you and your trading partner. It ties together a profile and one or more trading capabilities.")),
        ("DeleteProfile", deferred("Model requires profileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified profile. A profile is the mechanism used to create the concept of a private network.")),
        ("DeleteTransformer", deferred("Model requires transformerId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified transformer. A transformer can take an EDI file as input and transform it into a JSON-or XML-formatted document. Alternatively, a transformer can take a JSON-or XML-formatted document as input and transform…")),
        ("GenerateMapping", deferred("Model requires inputFileContent, outputFileContent, mappingType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Takes sample input and output documents and uses Amazon Bedrock to generate a mapping automatically. Depending on the accuracy and other factors, you can then edit the mapping for your needs. Before you can use the AI-assisted…")),
        ("GetCapability", deferred("Model requires capabilityId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the details for the specified capability. A trading capability contains the information required to transform incoming EDI documents into JSON or XML outputs.")),
        // Model: Retrieves the details for a partnership, based on the partner and profile IDs specified. A partnership represents the connection between you and your trading partner. It ties together a profile and…
        ("GetPartnership", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("partnershipId")?, Binding::From { operation: name("ListPartnerships")?, path: path("partnershipId")? })].into_iter().collect(),
            queryable: vec![path("partnershipId")?],
            ..read("$", Identity::Key(vec![path("partnershipId")?]), Scope::Region, Paginator::None)?
        }),
        // Model: Retrieves the details for the profile specified by the profile ID. A profile is the mechanism used to create the concept of a private network.
        ("GetProfile", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("profileId")?, Binding::From { operation: name("ListProfiles")?, path: path("profileId")? })].into_iter().collect(),
            queryable: vec![path("profileId")?],
            ..read("$", Identity::Key(vec![path("profileId")?]), Scope::Region, Paginator::None)?
        }),
        // Model: Retrieves the details for the transformer specified by the transformer ID. A transformer can take an EDI file as input and transform it into a JSON-or XML-formatted document. Alternatively, a…
        ("GetTransformer", denied(DenyReason::Content)),
        ("GetTransformerJob", deferred("Model requires transformerJobId, transformerId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the details of the transformer run, based on the Transformer job ID. If 30 days have elapsed since your transformer job was started, the system deletes it. So, if you run GetTransformerJob and supply a transformerId and…")),
        // Model: Lists the capabilities associated with your Amazon Web Services account for your current or specified region. A trading capability contains the information required to transform incoming EDI…
        ("ListCapabilities", OperationOverlay {
            queryable: vec![path("capabilityId")?],
            ..read("capabilities", Identity::Key(vec![path("capabilityId")?]), Scope::Region, token("nextToken", "nextToken")?)?
        }),
        // Model: Lists the partnerships associated with your Amazon Web Services account for your current or specified region. A partnership represents the connection between you and your trading partner. It ties…
        ("ListPartnerships", OperationOverlay {
            queryable: vec![path("partnershipId")?],
            ..read("partnerships", Identity::Key(vec![path("partnershipId")?]), Scope::Region, token("nextToken", "nextToken")?)?
        }),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Lists the profiles associated with your Amazon Web Services account for your current or specified region. A profile is the mechanism used to create the concept of a private network.
        ("ListProfiles", OperationOverlay {
            queryable: vec![path("profileId")?],
            ..read("profiles", Identity::Key(vec![path("profileId")?]), Scope::Region, token("nextToken", "nextToken")?)?
        }),
        ("ListTagsForResource", deferred("Model requires ResourceARN. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all of the tags associated with the Amazon Resource Name (ARN) that you specify. The resource can be a capability, partnership, profile, or transformer.")),
        // Model: Lists the available transformers. A transformer can take an EDI file as input and transform it into a JSON-or XML-formatted document. Alternatively, a transformer can take a JSON-or XML-formatted…
        ("ListTransformers", denied(DenyReason::Content)),
        ("StartTransformerJob", deferred("Model requires inputFile, outputLocation, transformerId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Runs a job, using a transformer, to parse input EDI (electronic data interchange) file into the output structures used by Amazon Web Services B2B Data Interchange. If you only want to transform EDI (electronic data interchange)…")),
        ("TagResource", deferred("Model requires ResourceARN, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Attaches a key-value pair to a resource, as identified by its Amazon Resource Name (ARN). Resources are capability, partnership, profile, transformers and other entities. There is no response returned from this call.")),
        ("TestConversion", deferred("Model requires source, target. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This operation mimics the latter half of a typical Outbound EDI request. It takes an input JSON/XML in the B2Bi shape as input, converts it to an X12 EDI string, and return that string.")),
        ("TestMapping", deferred("Model requires inputFileContent, mappingTemplate, fileFormat. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Maps the input file according to the provided template file. The API call downloads the file contents from the Amazon S3 location, and passes the contents in as a string, to the inputFileContent parameter.")),
        ("TestParsing", deferred("Model requires inputFile, fileFormat, ediType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Parses the input EDI (electronic data interchange) file. The input file has a file size limit of 250 KB.")),
        ("UntagResource", deferred("Model requires ResourceARN, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Detaches a key-value pair from the specified resource, as identified by its Amazon Resource Name (ARN). Resources are capability, partnership, profile, transformers and other entities.")),
        ("UpdateCapability", deferred("Model requires capabilityId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates some of the parameters for a capability, based on the specified parameters. A trading capability contains the information required to transform incoming EDI documents into JSON or XML outputs.")),
        ("UpdatePartnership", deferred("Model requires partnershipId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates some of the parameters for a partnership between a customer and trading partner. A partnership represents the connection between you and your trading partner. It ties together a profile and one or more trading…")),
        ("UpdateProfile", deferred("Model requires profileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the specified parameters for a profile. A profile is the mechanism used to create the concept of a private network.")),
        ("UpdateTransformer", deferred("Model requires transformerId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the specified parameters for a transformer. A transformer can take an EDI file as input and transform it into a JSON-or XML-formatted document. Alternatively, a transformer can take a JSON-or XML-formatted document as…")),
    ])
}
