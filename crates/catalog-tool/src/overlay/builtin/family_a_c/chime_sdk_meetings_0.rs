use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "BatchCreateAttendee",
            deferred(
                "Model requires MeetingId, Attendees. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates up to 100 attendees for an active Amazon Chime SDK meeting. For more information about the Amazon Chime SDK, see Using the Amazon Chime SDK in the Amazon Chime Developer Guide .",
            ),
        ),
        (
            "BatchUpdateAttendeeCapabilitiesExcept",
            deferred(
                "Model requires MeetingId, ExcludedAttendeeIds, Capabilities. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates AttendeeCapabilities except the capabilities listed in an ExcludedAttendeeIds table. You use the capabilities with a set of values that control what the capabilities can do, such as SendReceive data. For more information…",
            ),
        ),
        (
            "CreateAttendee",
            deferred(
                "Model requires MeetingId, ExternalUserId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new attendee for an active Amazon Chime SDK meeting. For more information about the Amazon Chime SDK, see Using the Amazon Chime SDK in the Amazon Chime Developer Guide .",
            ),
        ),
        (
            "CreateMeeting",
            deferred(
                "Model requires ClientRequestToken, MediaRegion, ExternalMeetingId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new Amazon Chime SDK meeting in the specified media Region with no initial attendees. For more information about specifying media Regions, see Available Regions and Using meeting Regions , both in the Amazon Chime SDK…",
            ),
        ),
        (
            "CreateMeetingWithAttendees",
            deferred(
                "Model requires ClientRequestToken, MediaRegion, ExternalMeetingId, Attendees. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new Amazon Chime SDK meeting in the specified media Region, with attendees. For more information about specifying media Regions, see Available Regions and Using meeting Regions , both in the Amazon Chime SDK Developer…",
            ),
        ),
        (
            "DeleteAttendee",
            deferred(
                "Model requires MeetingId, AttendeeId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an attendee from the specified Amazon Chime SDK meeting and deletes their JoinToken . Attendees are automatically deleted when a Amazon Chime SDK meeting is deleted. For more information about the Amazon Chime SDK, see…",
            ),
        ),
        (
            "DeleteMeeting",
            deferred(
                "Model requires MeetingId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified Amazon Chime SDK meeting. The operation deletes all attendees, disconnects all clients, and prevents new clients from joining the meeting. For more information about the Amazon Chime SDK, see Using the…",
            ),
        ),
        (
            "GetAttendee",
            deferred(
                "Model requires MeetingId, AttendeeId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the Amazon Chime SDK attendee details for a specified meeting ID and attendee ID. For more information about the Amazon Chime SDK, see Using the Amazon Chime SDK in the Amazon Chime Developer Guide .",
            ),
        ),
        (
            "GetMeeting",
            deferred(
                "Model requires MeetingId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the Amazon Chime SDK meeting details for the specified meeting ID. For more information about the Amazon Chime SDK, see Using the Amazon Chime SDK in the Amazon Chime Developer Guide .",
            ),
        ),
        ("ListAttendees", denied(DenyReason::Secret)),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceARN. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the tags available for the specified resource.",
            ),
        ),
        (
            "StartMeetingTranscription",
            deferred(
                "Model requires MeetingId, TranscriptionConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts transcription for the specified meetingId . For more information, refer to Using Amazon Chime SDK live transcription in the Amazon Chime SDK Developer Guide . If you specify an invalid configuration, a TranscriptFailed…",
            ),
        ),
        (
            "StopMeetingTranscription",
            deferred(
                "Model requires MeetingId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops transcription for the specified meetingId . For more information, refer to Using Amazon Chime SDK live transcription in the Amazon Chime SDK Developer Guide . By default, Amazon Transcribe may use and store audio content…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceARN, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The resource that supports tags.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceARN, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the specified tags from the specified resources. When you specify a tag key, the action removes both that key and its associated value. The operation succeeds even if you attempt to remove tags from a resource that were…",
            ),
        ),
        (
            "UpdateAttendeeCapabilities",
            deferred(
                "Model requires MeetingId, AttendeeId, Capabilities. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The capabilities that you want to update. You use the capabilities with a set of values that control what the capabilities can do, such as SendReceive data. For more information about those values, see . When using capabilities,…",
            ),
        ),
    ])
}
