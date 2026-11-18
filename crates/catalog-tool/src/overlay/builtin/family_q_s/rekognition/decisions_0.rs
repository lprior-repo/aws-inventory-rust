// Pinned operation purposes: .local/aws-models/models/rekognition/service/2016-06-27/rekognition-2016-06-27.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Associates one or more faces with an existing UserID. Takes an array of FaceIds . Each FaceId that are present in the FaceIds list is associated with the provided UserID. The number of FaceIds that can be used as input in a single request is limited to 100. Note that the total number of faces that can be associated with a single UserID is also limited to 100. Once a UserID has 100 faces associated
    ("AssociateFaces", Decision::Denied(DenyReason::Mutating)),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Compares a face in the source input image with each of the 100 largest faces detected in the target input image. If the source image contains multiple faces, the service detects the largest face and compares it with each face detected in the target image. CompareFaces uses machine learning algorithms, which are probabilistic. A false negative is an
    ("CompareFaces", Decision::Denied(DenyReason::Paid)),
    // Changes service state rather than reading inventory. Pinned purpose: This operation applies only to Amazon Rekognition Custom Labels. Copies a version of an Amazon Rekognition Custom Labels model from a source project to a destination project. The source and destination projects can be in different AWS accounts but must be in the same AWS Region. You can't copy a model to another AWS service. To copy a model version to a different AWS account, you need to create a
    ("CopyProjectVersion", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a collection in an AWS Region. You can add faces to the collection using the IndexFaces operation. For example, you might create collections, one for each of your application users. A user can then index faces using the IndexFaces operation and persist results in a specific collection. Then, a user can search the collection for faces in the user-specific container. When you create a collec
    ("CreateCollection", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: This operation applies only to Amazon Rekognition Custom Labels. Creates a new Amazon Rekognition Custom Labels dataset. You can create a dataset by using an Amazon Sagemaker format manifest file or by copying an existing Amazon Rekognition Custom Labels dataset. To create a training dataset for a project, specify TRAIN for the value of DatasetType . To create the test dataset for a project, speci
    ("CreateDataset", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: This API operation initiates a Face Liveness session. It returns a SessionId , which you can use to start streaming Face Liveness video and get the results for a Face Liveness session. You can use the OutputConfig option in the Settings parameter to provide an Amazon S3 bucket location. The Amazon S3 bucket stores reference images and audit images. If no Amazon S3 bucket is defined, raw bytes are
    (
        "CreateFaceLivenessSession",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new Amazon Rekognition project. A project is a group of resources (datasets, model versions) that you use to create and manage a Amazon Rekognition Custom Labels Model or custom adapter. You can specify a feature to create the project with, if no feature is specified then Custom Labels is used by default. For adapters, you can also choose whether or not to have the project auto update by
    ("CreateProject", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new version of Amazon Rekognition project (like a Custom Labels model or a custom adapter) and begins training. Models and adapters are managed as part of a Rekognition project. The response from CreateProjectVersion is an Amazon Resource Name (ARN) for the project version. The FeatureConfig operation argument allows you to configure specific model or adapter settings. You can provide a
    (
        "CreateProjectVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Service availability notice: Streaming Video and Bulk Image Analysis is no longer available to new customers. For more information, see Rekognition feature availability changes . This change does not impact the availability of other Amazon Rekognition features. Creates an Amazon Rekognition stream processor that you can use to detect and recognize faces or to detect labels in a streaming video. Am
    (
        "CreateStreamProcessor",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new User within a collection specified by CollectionId . Takes UserId as a parameter, which is a user provided ID which should be unique within the collection. The provided UserId will alias the system generated UUID to make the UserId more user friendly. Uses a ClientToken , an idempotency token that ensures a call to CreateUser completes only once. If the value is not supplied, the AWS
    ("CreateUser", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified collection. Note that this operation removes all faces in the collection. For an example, see Deleting a collection . This operation requires permissions to perform the rekognition:DeleteCollection action.
    ("DeleteCollection", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: This operation applies only to Amazon Rekognition Custom Labels. Deletes an existing Amazon Rekognition Custom Labels dataset. Deleting a dataset might take while. Use DescribeDataset to check the current status. The dataset is still deleting if the value of Status is DELETE_IN_PROGRESS . If you try to access the dataset after it is deleted, you get a ResourceNotFoundException exception. You can't
    ("DeleteDataset", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes faces from a collection. You specify a collection ID and an array of face IDs to remove from the collection. This operation requires permissions to perform the rekognition:DeleteFaces action.
    ("DeleteFaces", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a Amazon Rekognition project. To delete a project you must first delete all models or adapters associated with the project. To delete a model or adapter, see DeleteProjectVersion . DeleteProject is an asynchronous operation. To check if the project is deleted, call DescribeProjects . The project is deleted when the project no longer appears in the response. Be aware that deleting a given p
    ("DeleteProject", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: This operation applies only to Amazon Rekognition Custom Labels. Deletes an existing project policy. To get a list of project policies attached to a project, call ListProjectPolicies . To attach a project policy to a project, call PutProjectPolicy . This operation requires permissions to perform the rekognition:DeleteProjectPolicy action.
    (
        "DeleteProjectPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a Rekognition project model or project version, like a Amazon Rekognition Custom Labels model or a custom adapter. You can't delete a project version if it is running or if it is training. To check the status of a project version, use the Status field returned from DescribeProjectVersions . To stop a project version call StopProjectVersion . If the project version is training, wait until i
    (
        "DeleteProjectVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Service availability notice: Streaming Video and Bulk Image Analysis is no longer available to new customers. For more information, see Rekognition feature availability changes . This change does not impact the availability of other Amazon Rekognition features. Deletes the stream processor identified by Name . You assign the value for Name when you create the stream processor with CreateStreamProc
    (
        "DeleteStreamProcessor",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified UserID within the collection. Faces that are associated with the UserID are disassociated from the UserID before deleting the specified UserID. If the specified Collection or UserID is already deleted or not found, a ResourceNotFoundException will be thrown. If the action is successful with a 200 response, an empty HTTP body is returned.
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
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: This operation applies only to Amazon Rekognition Custom Labels. Detects custom labels in a supplied image by using an Amazon Rekognition Custom Labels model. You specify which version of a model version to use by using the ProjectVersionArn input parameter. You pass the input image as base64-encoded image bytes or as a reference to an image in an
    ("DetectCustomLabels", Decision::Denied(DenyReason::Paid)),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Detects faces within an image that is provided as input. DetectFaces detects the 100 largest faces in the image. For each face detected, the operation returns face details. These details include a bounding box of the face, a confidence value (that the bounding box contains a face), and a fixed set of attributes such as facial landmarks (for example
    ("DetectFaces", Decision::Denied(DenyReason::Paid)),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Detects instances of real-world entities within an image (JPEG or PNG) provided as input. This includes objects like flower, tree, and table; events like wedding, graduation, and birthday party; and concepts like landscape, evening, and nature. For an example, see Analyzing images stored in an Amazon S3 bucket in the Amazon Rekognition Developer Gu
    ("DetectLabels", Decision::Denied(DenyReason::Paid)),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Detects unsafe content in a specified JPEG or PNG format image. Use DetectModerationLabels to moderate images depending on your requirements. For example, you might want to filter images that contain nudity, but not images containing suggestive content. To filter images, use the labels returned by DetectModerationLabels to determine which types of
    ("DetectModerationLabels", Decision::Denied(DenyReason::Paid)),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Detects Personal Protective Equipment (PPE) worn by people detected in an image. Amazon Rekognition can detect the following types of PPE. Face cover Hand cover Head cover You pass the input image as base64-encoded image bytes or as a reference to an image in an Amazon S3 bucket. The image must be either a PNG or JPG formatted file. DetectProtectiv
    (
        "DetectProtectiveEquipment",
        Decision::Denied(DenyReason::Paid),
    ),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Detects text in the input image and converts it into machine-readable text. Pass the input image as base64-encoded image bytes or as a reference to an image in an Amazon S3 bucket. If you use the AWS CLI to call Amazon Rekognition operations, you must pass it as a reference to an image in an Amazon S3 bucket. For the AWS CLI, passing image bytes is
    ("DetectText", Decision::Denied(DenyReason::Paid)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the association between a Face supplied in an array of FaceIds and the User. If the User is not present already, then a ResourceNotFound exception is thrown. If successful, an array of faces that are disassociated from the User is returned. If a given face is already disassociated from the given UserID, it will be ignored and not be returned in the response. If a given face is already asso
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
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Gets the inappropriate, unwanted, or offensive content analysis results for a Amazon Rekognition Video analysis started by StartContentModeration . For a list of moderation labels in Amazon Rekognition, see Using the image and video moderation APIs . Amazon Rekognition Video inappropriate or offensive content detection in a stored video is an asynchronous operation. You start analysis by calling S
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
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Service availability notice: Streaming Video and Bulk Image Analysis is no longer available to new customers. For more information, see Rekognition feature availability changes . This change does not impact the availability of other Amazon Rekognition features. Retrieves the results for a given media analysis job. Takes a JobId returned by StartMediaAnalysisJob.
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
    // Changes service state rather than reading inventory. Pinned purpose: This operation applies only to Amazon Rekognition Custom Labels. Attaches a project policy to a Amazon Rekognition Custom Labels project in a trusting AWS account. A project policy specifies that a trusted AWS account can copy a model version from a trusting AWS account to a project in the trusted AWS account. To copy a model version you use the CopyProjectVersion operation. Only applies to Custom
    ("PutProjectPolicy", Decision::Denied(DenyReason::Mutating)),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Returns an array of celebrities recognized in the input image. For more information, see Recognizing celebrities in the Amazon Rekognition Developer Guide. RecognizeCelebrities returns the 64 largest faces in the image. It lists the recognized celebrities in the CelebrityFaces array and any unrecognized faces in the UnrecognizedFaces array. Recogni
    ("RecognizeCelebrities", Decision::Denied(DenyReason::Paid)),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: For a given input face ID, searches for matching faces in the collection the face belongs to. You get a face ID when you add a face to the collection using the IndexFaces operation. The operation compares the features of the input face with faces in the specified collection. You can also search faces without indexing faces by using the SearchFacesB
    ("SearchFaces", Decision::Denied(DenyReason::Paid)),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: For a given input image, first detects the largest face in the image, and then searches the specified collection for matching faces. The operation compares the features of the input face with faces in the specified collection. To search for all faces in an input image, you might first call the IndexFaces operation, and then use the face IDs returne
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
    // Changes service state rather than reading inventory. Pinned purpose: Starts asynchronous recognition of celebrities in a stored video. Amazon Rekognition Video can detect celebrities in a video must be stored in an Amazon S3 bucket. Use Video to specify the bucket name and the filename of the video. StartCelebrityRecognition returns a job identifier ( JobId ) which you use to get the results of the analysis. When celebrity recognition analysis is finished, Amazon R
    (
        "StartCelebrityRecognition",
        Decision::Denied(DenyReason::Mutating),
    ),
];
