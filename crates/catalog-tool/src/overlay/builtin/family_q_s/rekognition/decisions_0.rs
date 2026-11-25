use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("AssociateFaces", Decision::Denied(DenyReason::Mutating)),
    ("CompareFaces", Decision::Denied(DenyReason::Paid)),
    ("CopyProjectVersion", Decision::Denied(DenyReason::Mutating)),
    ("CreateCollection", Decision::Denied(DenyReason::Mutating)),
    ("CreateDataset", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateFaceLivenessSession",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateProject", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateProjectVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateStreamProcessor",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateUser", Decision::Denied(DenyReason::Mutating)),
    ("DeleteCollection", Decision::Denied(DenyReason::Mutating)),
    ("DeleteDataset", Decision::Denied(DenyReason::Mutating)),
    ("DeleteFaces", Decision::Denied(DenyReason::Mutating)),
    ("DeleteProject", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteProjectPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteProjectVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteStreamProcessor",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteUser", Decision::Denied(DenyReason::Mutating)),
    (
        "DescribeCollection",
        Decision::Deferred(
            "Required selectors CollectionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members FaceCount, FaceModelVersion, CollectionARN, CreationTimestamp, UserCount. Pinned purpose: Describes the specified collection. You can use DescribeCollection to get information, such as the number of faces indexed into a collection and the version of the model used by the collection for face detection. For more information, see Describing ",
        ),
    ),
    (
        "DescribeDataset",
        Decision::Deferred(
            "Required selectors DatasetArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DatasetDescription. Pinned purpose: This operation applies only to Amazon Rekognition Custom Labels. Describes an Amazon Rekognition Custom Labels dataset. You can get information such as the current status of a dataset and statistics about the images and labels in a dataset. This oper",
        ),
    ),
    (
        "DescribeProjectVersions",
        Decision::Deferred(
            "Required selectors ProjectArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ProjectVersionDescriptions, NextToken. Pinned purpose: Lists and describes the versions of an Amazon Rekognition project. You can specify up to 10 model or adapter versions in ProjectVersionArns . If you don't specify a value, descriptions for all model/adapter versions in the project are returned. This ",
        ),
    ),
    (
        "DescribeStreamProcessor",
        Decision::Deferred(
            "Required selectors Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Name, StreamProcessorArn, Status, StatusMessage, CreationTimestamp, LastUpdateTimestamp, Input, Output, RoleArn, Settings, NotificationChannel, KmsKeyId, RegionsOfInterest, DataSharingPreference. Pinned purpose: Service availability notice: Streaming Video and Bulk Image Analysis is no longer available to new customers. For more information, see Rekognition feature availability changes . This change does not impact the availability of other Amazon Rekognitio",
        ),
    ),
    ("DetectCustomLabels", Decision::Denied(DenyReason::Paid)),
    ("DetectFaces", Decision::Denied(DenyReason::Paid)),
    ("DetectLabels", Decision::Denied(DenyReason::Paid)),
    ("DetectModerationLabels", Decision::Denied(DenyReason::Paid)),
    (
        "DetectProtectiveEquipment",
        Decision::Denied(DenyReason::Paid),
    ),
    ("DetectText", Decision::Denied(DenyReason::Paid)),
    ("DisassociateFaces", Decision::Denied(DenyReason::Mutating)),
    (
        "DistributeDatasetEntries",
        Decision::Deferred(
            "Required selectors Datasets are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: This operation applies only to Amazon Rekognition Custom Labels. Distributes the entries (images) in a training dataset across the training dataset and the test dataset for a project. DistributeDatasetEntries moves 20% of the training dataset images ",
        ),
    ),
    (
        "GetCelebrityInfo",
        Decision::Deferred(
            "Required selectors Id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Urls, Name, KnownGender. Pinned purpose: Gets the name and additional information about a celebrity based on their Amazon Rekognition ID. The additional information is returned as an array of URLs. If there is no additional information about the celebrity, this list is empty. For more infor",
        ),
    ),
    (
        "GetCelebrityRecognition",
        Decision::Deferred(
            "Required selectors JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobStatus, StatusMessage, VideoMetadata, NextToken, Celebrities, JobId, Video, JobTag. Pinned purpose: Gets the celebrity recognition results for a Amazon Rekognition Video analysis started by StartCelebrityRecognition . Celebrity recognition in a video is an asynchronous operation. Analysis is started by a call to StartCelebrityRecognition which retu",
        ),
    ),
    (
        "GetContentModeration",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "GetFaceDetection",
        Decision::Deferred(
            "Required selectors JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobStatus, StatusMessage, VideoMetadata, NextToken, Faces, JobId, Video, JobTag. Pinned purpose: Gets face detection results for a Amazon Rekognition Video analysis started by StartFaceDetection . Face detection with Amazon Rekognition Video is an asynchronous operation. You start face detection by calling StartFaceDetection which returns a job ",
        ),
    ),
    (
        "GetFaceLivenessSessionResults",
        Decision::Deferred(
            "Required selectors SessionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members SessionId, Status, Confidence, ReferenceImage, AuditImages, Challenge, Feedback, Metadata. Pinned purpose: Retrieves the results of a specific Face Liveness session. It requires the sessionId as input, which was created using CreateFaceLivenessSession . Returns the corresponding Face Liveness confidence score, a reference image that includes a face boundi",
        ),
    ),
    (
        "GetFaceSearch",
        Decision::Deferred(
            "Required selectors JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobStatus, StatusMessage, NextToken, VideoMetadata, Persons, JobId, Video, JobTag. Pinned purpose: Gets the face search results for Amazon Rekognition Video face search started by StartFaceSearch . The search returns faces in a collection that match the faces of persons detected in a video. It also includes the time(s) that faces are matched in th",
        ),
    ),
    (
        "GetLabelDetection",
        Decision::Deferred(
            "Required selectors JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobStatus, StatusMessage, VideoMetadata, NextToken, Labels, LabelModelVersion, JobId, Video, JobTag, GetRequestMetadata. Pinned purpose: Gets the label detection results of a Amazon Rekognition Video analysis started by StartLabelDetection . The label detection operation is started by a call to StartLabelDetection which returns a job identifier ( JobId ). When the label detection oper",
        ),
    ),
    ("GetMediaAnalysisJob", Decision::Denied(DenyReason::Content)),
    (
        "GetPersonTracking",
        Decision::Deferred(
            "Required selectors JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobStatus, StatusMessage, VideoMetadata, NextToken, Persons, JobId, Video, JobTag. Pinned purpose: End of support notice: On October 31, 2025, AWS will discontinue support for Amazon Rekognition People Pathing. After October 31, 2025, you will no longer be able to use the Rekognition People Pathing capability. For more information, visit this blog",
        ),
    ),
    (
        "GetSegmentDetection",
        Decision::Deferred(
            "Required selectors JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobStatus, StatusMessage, VideoMetadata, AudioMetadata, NextToken, Segments, SelectedSegmentTypes, JobId, Video, JobTag. Pinned purpose: Gets the segment detection results of a Amazon Rekognition Video analysis started by StartSegmentDetection . Segment detection with Amazon Rekognition Video is an asynchronous operation. You start segment detection by calling StartSegmentDetection wh",
        ),
    ),
    (
        "GetTextDetection",
        Decision::Deferred(
            "Required selectors JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobStatus, StatusMessage, VideoMetadata, TextDetections, NextToken, TextModelVersion, JobId, Video, JobTag. Pinned purpose: Gets the text detection results of a Amazon Rekognition Video analysis started by StartTextDetection . Text detection with Amazon Rekognition Video is an asynchronous operation. You start text detection by calling StartTextDetection which returns a j",
        ),
    ),
    (
        "IndexFaces",
        Decision::Deferred(
            "Required selectors CollectionId, Image are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members FaceRecords, OrientationCorrection, FaceModelVersion, UnindexedFaces. Pinned purpose: Detects faces in the input image and adds them to the specified collection. Amazon Rekognition doesn't save the actual faces that are detected. Instead, the underlying detection algorithm first detects the faces in the input image. For each face, the",
        ),
    ),
    (
        "ListDatasetEntries",
        Decision::Deferred(
            "Required selectors DatasetArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DatasetEntries, NextToken. Pinned purpose: This operation applies only to Amazon Rekognition Custom Labels. Lists the entries (images) within a dataset. An entry is a JSON Line that contains the information for a single image, including the image location, assigned labels, and object location",
        ),
    ),
    (
        "ListDatasetLabels",
        Decision::Deferred(
            "Required selectors DatasetArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DatasetLabelDescriptions, NextToken. Pinned purpose: This operation applies only to Amazon Rekognition Custom Labels. Lists the labels in a dataset. Amazon Rekognition Custom Labels uses labels to describe images. For more information, see Labeling images . Lists the labels in a dataset. Amazon Rekogni",
        ),
    ),
    (
        "ListFaces",
        Decision::Deferred(
            "Required selectors CollectionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Faces, NextToken, FaceModelVersion. Pinned purpose: Returns metadata for faces in the specified collection. This metadata includes information such as the bounding box coordinates, the confidence (that the bounding box contains a face), and face ID. For an example, see Listing Faces in a Collection in",
        ),
    ),
    (
        "ListProjectPolicies",
        Decision::Deferred(
            "Required selectors ProjectArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ProjectPolicies, NextToken. Pinned purpose: This operation applies only to Amazon Rekognition Custom Labels. Gets a list of the project policies attached to a project. To attach a project policy to a project, call PutProjectPolicy . To remove a project policy from a project, call DeleteProject",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Returns a list of tags in an Amazon Rekognition collection, stream processor, or Custom Labels model. This operation requires permissions to perform the rekognition:ListTagsForResource action.",
        ),
    ),
    (
        "ListUsers",
        Decision::Deferred(
            "Required selectors CollectionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Users, NextToken. Pinned purpose: Returns metadata of the User such as UserID in the specified collection. Anonymous User (to reserve faces without any identity) is not returned as part of this request. The results are sorted by system generated primary key ID. If the response is tru",
        ),
    ),
    ("PutProjectPolicy", Decision::Denied(DenyReason::Mutating)),
    ("RecognizeCelebrities", Decision::Denied(DenyReason::Paid)),
    ("SearchFaces", Decision::Denied(DenyReason::Paid)),
    ("SearchFacesByImage", Decision::Denied(DenyReason::Paid)),
    (
        "SearchUsers",
        Decision::Deferred(
            "Required selectors CollectionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members UserMatches, FaceModelVersion, SearchedFace, SearchedUser. Pinned purpose: Searches for UserIDs within a collection based on a FaceId or UserId . This API can be used to find the closest UserID (with a highest similarity) to associate a face. The request must be provided with either FaceId or UserId . The operation returns ",
        ),
    ),
    (
        "SearchUsersByImage",
        Decision::Deferred(
            "Required selectors CollectionId, Image are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members UserMatches, FaceModelVersion, SearchedFace, UnsearchedFaces. Pinned purpose: Searches for UserIDs using a supplied image. It first detects the largest face in the image, and then searches a specified collection for matching UserIDs. The operation returns an array of UserIDs that match the face in the supplied image, ordered b",
        ),
    ),
    (
        "StartCelebrityRecognition",
        Decision::Denied(DenyReason::Mutating),
    ),
];
