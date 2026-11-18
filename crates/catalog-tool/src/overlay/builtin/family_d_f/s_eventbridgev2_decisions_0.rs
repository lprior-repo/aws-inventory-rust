// Explicit dispositions from .local/aws-models/models/eventbridgev2/service/2025-05-15/eventbridgev2-2025-05-15.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    ("CreateEventBus", "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an event bus."),
    ("CreateEventSource", "Requires Name, EventBusArn, Configuration; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an EventSource, which forwards events from an origin (an AWS service or another account) onto an event bus."),
    ("CreateSubscriber", "Requires Name, EventBusArn, InvokeConfiguration; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a subscriber on an event bus, which delivers matching events to the configured target."),
    ("DeleteEventBus", "Requires EventBusArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an event bus."),
    ("DeleteEventSource", "Requires EventSourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an EventSource."),
    ("DeleteResourcePolicy", "Requires ResourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the named resource policy attached to an event bus."),
    ("DeleteSubscriber", "Requires SubscriberArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a subscriber."),
    
    
    
    ("GetResourcePolicy", "Requires ResourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the named resource policy attached to an event bus."),
    ("ListResourcePolicies", "Requires ResourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists the resource policies attached to an event bus as summaries (policy name and revision ID)."),
    ("ListTagsForResource", "Requires ResourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists the tags on an event bus, subscriber, or event source."),
    ("PutEvents", "Requires EventBusArn, Entries; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Publishes events to an event bus."),
    ("PutRawEvents", "Requires EventBusArn, Entries; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Publishes pre-shaped events to an event bus."),
    ("PutResourcePolicy", "Requires ResourceArn, PolicyDocument; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Attaches a named resource policy to an event bus — the only resource type that supports policies; other resource ARNs are rejected."),
    ("RevokeResource", "Requires Arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Revokes a subscriber or an EventSource."),
    ("TagResource", "Requires ResourceArn, Tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds or replaces tags on an event bus, subscriber, or event source."),
    ("UntagResource", "Requires ResourceArn, TagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes tags from an event bus, subscriber, or event source."),
    ("UpdateEventBus", "Requires EventBusArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates an event bus."),
    ("UpdateEventSource", "Requires EventSourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates an EventSource."),
    ("UpdateSubscriber", "Requires SubscriberArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a subscriber."),
];
