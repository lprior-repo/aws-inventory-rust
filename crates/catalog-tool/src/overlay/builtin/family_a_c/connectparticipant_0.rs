// Pinned source: .local/aws-models/models/connectparticipant/service/2018-09-07/connectparticipant-2018-09-07.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CancelParticipantAuthentication",
            deferred(
                "Model requires SessionId, ConnectionToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancels the authentication session. The opted out branch of the Authenticate Customer flow block will be taken. The current supported channel is chat. This API is not supported for Apple Messages for Business, WhatsApp, or SMS…",
            ),
        ),
        (
            "CompleteAttachmentUpload",
            deferred(
                "Model requires AttachmentIds, ClientToken, ConnectionToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Allows you to confirm that the attachment has been uploaded using the pre-signed URL provided in StartAttachmentUpload API. A conflict exception is thrown when an attachment with that identifier is already being uploaded. For…",
            ),
        ),
        (
            "CreateParticipantConnection",
            deferred(
                "Model requires ParticipantToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates the participant's connection. For security recommendations, see Connect Customer Chat security best practices . For WebRTC security recommendations, see Connect Customer WebRTC security best practices . ParticipantToken…",
            ),
        ),
        (
            "DescribeView",
            deferred(
                "Model requires ViewToken, ConnectionToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the view for the specified view token. For security recommendations, see Connect Customer Chat security best practices .",
            ),
        ),
        (
            "DisconnectParticipant",
            deferred(
                "Model requires ConnectionToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disconnects a participant. For security recommendations, see Connect Customer Chat security best practices . ConnectionToken is used for invoking this API instead of ParticipantToken . The Amazon Connect Participant Service APIs…",
            ),
        ),
        (
            "GetAttachment",
            deferred(
                "Model requires AttachmentId, ConnectionToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides a pre-signed URL for download of a completed attachment. This is an asynchronous API for use with active contacts. For security recommendations, see Connect Customer Chat security best practices . The participant role…",
            ),
        ),
        (
            "GetAuthenticationUrl",
            deferred(
                "Model requires SessionId, RedirectUri, ConnectionToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the AuthenticationUrl for the current authentication session for the AuthenticateCustomer flow block. For security recommendations, see Connect Customer Chat security best practices . This API can only be called within…",
            ),
        ),
        (
            "GetTranscript",
            deferred(
                "Model requires ConnectionToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a transcript of the session, including details about any attachments. For information about accessing past chat contact transcripts for a persistent chat, see Enable persistent chat . For security recommendations, see…",
            ),
        ),
        (
            "SendEvent",
            deferred(
                "Model requires ContentType, ConnectionToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The application/vnd.amazonaws.connect.event.connection.acknowledged ContentType is no longer maintained since December 31, 2024. This event has been migrated to the CreateParticipantConnection API using the ConnectParticipant…",
            ),
        ),
        (
            "SendMessage",
            deferred(
                "Model requires ContentType, Content, ConnectionToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sends a message. For security recommendations, see Connect Customer Chat security best practices . ConnectionToken is used for invoking this API instead of ParticipantToken . The Amazon Connect Participant Service APIs do not use…",
            ),
        ),
        (
            "StartAttachmentUpload",
            deferred(
                "Model requires ContentType, AttachmentSizeInBytes, AttachmentName, ClientToken, ConnectionToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides a pre-signed Amazon S3 URL in response for uploading the file directly to S3. For security recommendations, see Connect Customer Chat security best practices . ConnectionToken is used for invoking this API instead of…",
            ),
        ),
    ])
}
