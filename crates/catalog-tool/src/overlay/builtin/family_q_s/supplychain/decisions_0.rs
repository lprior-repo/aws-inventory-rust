// Pinned operation purposes: .local/aws-models/models/supplychain/service/2024-01-01/supplychain-2024-01-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: CreateBillOfMaterialsImportJob creates an import job for the Product Bill Of Materials (BOM) entity. For information on the product_bom entity, see the AWS Supply Chain User Guide. The CSV file must be located in an Amazon S3 location accessible to AWS Supply Chain. It is recommended to use the same Amazon S3 bucket created during your AWS Supply Chain instance creation.
    ("CreateBillOfMaterialsImportJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically create a data pipeline to ingest data from source systems such as Amazon S3 buckets, to a predefined Amazon Web Services Supply Chain dataset (product, inbound_order) or a temporary dataset along with the data transformation query provided with the API.
    ("CreateDataIntegrationFlow", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically create an Amazon Web Services Supply Chain data lake dataset. Developers can create the datasets using their pre-defined or custom schema for a given instance ID, namespace, and dataset name.
    ("CreateDataLakeDataset", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically create an Amazon Web Services Supply Chain data lake namespace. Developers can create the namespaces for a given instance ID.
    ("CreateDataLakeNamespace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically create an Amazon Web Services Supply Chain instance by applying KMS keys and relevant information associated with the API without using the Amazon Web Services console. This is an asynchronous operation. Upon receiving a CreateInstance request, Amazon Web Services Supply Chain immediately returns the instance resource, instance ID, and the initializing state while s
    ("CreateInstance", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enable you to programmatically delete an existing data pipeline for the provided Amazon Web Services Supply Chain instance and DataIntegrationFlow name.
    ("DeleteDataIntegrationFlow", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically delete an Amazon Web Services Supply Chain data lake dataset. Developers can delete the existing datasets for a given instance ID, namespace, and instance name.
    ("DeleteDataLakeDataset", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically delete an Amazon Web Services Supply Chain data lake namespace and its underling datasets. Developers can delete the existing namespaces for a given instance ID and namespace name.
    ("DeleteDataLakeNamespace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically delete an Amazon Web Services Supply Chain instance by deleting the KMS keys and relevant information associated with the API without using the Amazon Web Services console. This is an asynchronous operation. Upon receiving a DeleteInstance request, Amazon Web Services Supply Chain immediately returns a response with the instance resource, delete state while cleaning
    ("DeleteInstance", Decision::Denied(DenyReason::Mutating)),
    ("GetBillOfMaterialsImportJob", Decision::Deferred("Required selectors instanceId, jobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members job. Pinned purpose: Get status and details of a BillOfMaterialsImportJob.")),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically view an Amazon Web Services Supply Chain Data Integration Event. Developers can view the eventType, eventGroupId, eventTimestamp, datasetTarget, datasetLoadExecution.
    ("GetDataIntegrationEvent", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically view a specific data pipeline for the provided Amazon Web Services Supply Chain instance and DataIntegrationFlow name.
    ("GetDataIntegrationFlow", Decision::Denied(DenyReason::Mutating)),
    ("GetDataIntegrationFlowExecution", Decision::Deferred("Required selectors instanceId, flowName, executionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members flowExecution. Pinned purpose: Get the flow execution.")),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically view an Amazon Web Services Supply Chain data lake dataset. Developers can view the data lake dataset information such as namespace, schema, and so on for a given instance ID, namespace, and dataset name.
    ("GetDataLakeDataset", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically view an Amazon Web Services Supply Chain data lake namespace. Developers can view the data lake namespace information such as description for a given instance ID and namespace name.
    ("GetDataLakeNamespace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically retrieve the information related to an Amazon Web Services Supply Chain instance ID.
    ("GetInstance", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically list all data integration events for the provided Amazon Web Services Supply Chain instance.
    ("ListDataIntegrationEvents", Decision::Denied(DenyReason::Mutating)),
    ("ListDataIntegrationFlowExecutions", Decision::Deferred("Required selectors instanceId, flowName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members flowExecutions, nextToken. Pinned purpose: List flow executions.")),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically list all data pipelines for the provided Amazon Web Services Supply Chain instance.
    ("ListDataIntegrationFlows", Decision::Denied(DenyReason::Mutating)),
    ("ListTagsForResource", Decision::Deferred("Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: List all the tags for an Amazon Web ServicesSupply Chain resource. You can list all the tags added to a resource. By listing the tags, developers can view the tag level information on a resource and perform actions such as, deleting a resource associated with a particular tag.")),
    // Changes service state rather than reading inventory. Pinned purpose: Send the data payload for the event with real-time data for analysis or monitoring. The real-time data events are stored in an Amazon Web Services service before being processed and stored in data lake.
    ("SendDataIntegrationEvent", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: You can create tags during or after creating a resource such as instance, data flow, or dataset in AWS Supply chain. During the data ingestion process, you can add tags such as dev, test, or prod to data flows created during the data ingestion process in the AWS Supply Chain datasets. You can use these tags to identify a group of resources or a single resource used by the developer.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: You can delete tags for an Amazon Web Services Supply chain resource such as instance, data flow, or dataset in AWS Supply Chain. During the data ingestion process, you can delete tags such as dev, test, or prod to data flows created during the data ingestion process in the AWS Supply Chain datasets.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically update an existing data pipeline to ingest data from the source systems such as, Amazon S3 buckets, to a predefined Amazon Web Services Supply Chain dataset (product, inbound_order) or a temporary dataset along with the data transformation query provided with the API.
    ("UpdateDataIntegrationFlow", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically update an Amazon Web Services Supply Chain data lake dataset. Developers can update the description of a data lake dataset for a given instance ID, namespace, and dataset name.
    ("UpdateDataLakeDataset", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically update an Amazon Web Services Supply Chain data lake namespace. Developers can update the description of a data lake namespace for a given instance ID and namespace name.
    ("UpdateDataLakeNamespace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to programmatically update an Amazon Web Services Supply Chain instance description by providing all the relevant information such as account ID, instance ID and so on without using the AWS console.
    ("UpdateInstance", Decision::Denied(DenyReason::Mutating)),
];
