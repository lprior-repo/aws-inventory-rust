//! Generated SDK catalog. Availability is not implemented collector coverage.
//! Service crates compile only when their explicit feature is enabled.
#![forbid(unsafe_code)]
#![deny(unused_must_use)]

pub const SERVICE_COUNT: usize = 437;
pub const SERVICE_MANIFEST_JSON: &str = include_str!("../sdk-services.json");

#[cfg(feature = "accessanalyzer")]
pub use aws_sdk_accessanalyzer as accessanalyzer;

#[cfg(feature = "account")]
pub use aws_sdk_account as account;

#[cfg(feature = "accountaccess")]
pub use aws_sdk_accountaccess as accountaccess;

#[cfg(feature = "accountaccessruntime")]
pub use aws_sdk_accountaccessruntime as accountaccessruntime;

#[cfg(feature = "acm")]
pub use aws_sdk_acm as acm;

#[cfg(feature = "acmpca")]
pub use aws_sdk_acmpca as acmpca;

#[cfg(feature = "agentregistry")]
pub use aws_sdk_agentregistry as agentregistry;

#[cfg(feature = "agentregistrycontrol")]
pub use aws_sdk_agentregistrycontrol as agentregistrycontrol;

#[cfg(feature = "aiops")]
pub use aws_sdk_aiops as aiops;

#[cfg(feature = "amp")]
pub use aws_sdk_amp as amp;

#[cfg(feature = "amplify")]
pub use aws_sdk_amplify as amplify;

#[cfg(feature = "amplifybackend")]
pub use aws_sdk_amplifybackend as amplifybackend;

#[cfg(feature = "amplifyuibuilder")]
pub use aws_sdk_amplifyuibuilder as amplifyuibuilder;

#[cfg(feature = "apigateway")]
pub use aws_sdk_apigateway as apigateway;

#[cfg(feature = "apigatewaymanagement")]
pub use aws_sdk_apigatewaymanagement as apigatewaymanagement;

#[cfg(feature = "apigatewayv2")]
pub use aws_sdk_apigatewayv2 as apigatewayv2;

#[cfg(feature = "appconfig")]
pub use aws_sdk_appconfig as appconfig;

#[cfg(feature = "appconfigdata")]
pub use aws_sdk_appconfigdata as appconfigdata;

#[cfg(feature = "appfabric")]
pub use aws_sdk_appfabric as appfabric;

#[cfg(feature = "appflow")]
pub use aws_sdk_appflow as appflow;

#[cfg(feature = "appintegrations")]
pub use aws_sdk_appintegrations as appintegrations;

#[cfg(feature = "applicationautoscaling")]
pub use aws_sdk_applicationautoscaling as applicationautoscaling;

#[cfg(feature = "applicationcostprofiler")]
pub use aws_sdk_applicationcostprofiler as applicationcostprofiler;

#[cfg(feature = "applicationdiscovery")]
pub use aws_sdk_applicationdiscovery as applicationdiscovery;

#[cfg(feature = "applicationinsights")]
pub use aws_sdk_applicationinsights as applicationinsights;

#[cfg(feature = "applicationsignals")]
pub use aws_sdk_applicationsignals as applicationsignals;

#[cfg(feature = "appmesh")]
pub use aws_sdk_appmesh as appmesh;

#[cfg(feature = "apprunner")]
pub use aws_sdk_apprunner as apprunner;

#[cfg(feature = "appstream")]
pub use aws_sdk_appstream as appstream;

#[cfg(feature = "appsync")]
pub use aws_sdk_appsync as appsync;

#[cfg(feature = "arcregionswitch")]
pub use aws_sdk_arcregionswitch as arcregionswitch;

#[cfg(feature = "arczonalshift")]
pub use aws_sdk_arczonalshift as arczonalshift;

#[cfg(feature = "artifact")]
pub use aws_sdk_artifact as artifact;

#[cfg(feature = "athena")]
pub use aws_sdk_athena as athena;

#[cfg(feature = "auditmanager")]
pub use aws_sdk_auditmanager as auditmanager;

#[cfg(feature = "autoscaling")]
pub use aws_sdk_autoscaling as autoscaling;

#[cfg(feature = "autoscalingplans")]
pub use aws_sdk_autoscalingplans as autoscalingplans;

#[cfg(feature = "b2bi")]
pub use aws_sdk_b2bi as b2bi;

#[cfg(feature = "backup")]
pub use aws_sdk_backup as backup;

#[cfg(feature = "backupgateway")]
pub use aws_sdk_backupgateway as backupgateway;

#[cfg(feature = "backupsearch")]
pub use aws_sdk_backupsearch as backupsearch;

#[cfg(feature = "batch")]
pub use aws_sdk_batch as batch;

#[cfg(feature = "bcmdashboards")]
pub use aws_sdk_bcmdashboards as bcmdashboards;

#[cfg(feature = "bcmdataexports")]
pub use aws_sdk_bcmdataexports as bcmdataexports;

#[cfg(feature = "bcmpricingcalculator")]
pub use aws_sdk_bcmpricingcalculator as bcmpricingcalculator;

#[cfg(feature = "bcmrecommendedactions")]
pub use aws_sdk_bcmrecommendedactions as bcmrecommendedactions;

#[cfg(feature = "bedrock")]
pub use aws_sdk_bedrock as bedrock;

#[cfg(feature = "bedrockagent")]
pub use aws_sdk_bedrockagent as bedrockagent;

#[cfg(feature = "bedrockagentcore")]
pub use aws_sdk_bedrockagentcore as bedrockagentcore;

#[cfg(feature = "bedrockagentcorecontrol")]
pub use aws_sdk_bedrockagentcorecontrol as bedrockagentcorecontrol;

#[cfg(feature = "bedrockagentruntime")]
pub use aws_sdk_bedrockagentruntime as bedrockagentruntime;

#[cfg(feature = "bedrockdataautomation")]
pub use aws_sdk_bedrockdataautomation as bedrockdataautomation;

#[cfg(feature = "bedrockdataautomationruntime")]
pub use aws_sdk_bedrockdataautomationruntime as bedrockdataautomationruntime;

#[cfg(feature = "bedrockruntime")]
pub use aws_sdk_bedrockruntime as bedrockruntime;

#[cfg(feature = "billing")]
pub use aws_sdk_billing as billing;

#[cfg(feature = "billingconductor")]
pub use aws_sdk_billingconductor as billingconductor;

#[cfg(feature = "braket")]
pub use aws_sdk_braket as braket;

#[cfg(feature = "budgets")]
pub use aws_sdk_budgets as budgets;

#[cfg(feature = "chatbot")]
pub use aws_sdk_chatbot as chatbot;

#[cfg(feature = "chime")]
pub use aws_sdk_chime as chime;

#[cfg(feature = "chimesdkidentity")]
pub use aws_sdk_chimesdkidentity as chimesdkidentity;

#[cfg(feature = "chimesdkmediapipelines")]
pub use aws_sdk_chimesdkmediapipelines as chimesdkmediapipelines;

#[cfg(feature = "chimesdkmeetings")]
pub use aws_sdk_chimesdkmeetings as chimesdkmeetings;

#[cfg(feature = "chimesdkmessaging")]
pub use aws_sdk_chimesdkmessaging as chimesdkmessaging;

#[cfg(feature = "chimesdkvoice")]
pub use aws_sdk_chimesdkvoice as chimesdkvoice;

#[cfg(feature = "cleanrooms")]
pub use aws_sdk_cleanrooms as cleanrooms;

#[cfg(feature = "cleanroomsml")]
pub use aws_sdk_cleanroomsml as cleanroomsml;

#[cfg(feature = "cloud9")]
pub use aws_sdk_cloud9 as cloud9;

#[cfg(feature = "cloudcontrol")]
pub use aws_sdk_cloudcontrol as cloudcontrol;

#[cfg(feature = "clouddirectory")]
pub use aws_sdk_clouddirectory as clouddirectory;

#[cfg(feature = "cloudformation")]
pub use aws_sdk_cloudformation as cloudformation;

#[cfg(feature = "cloudfront")]
pub use aws_sdk_cloudfront as cloudfront;

#[cfg(feature = "cloudfrontkeyvaluestore")]
pub use aws_sdk_cloudfrontkeyvaluestore as cloudfrontkeyvaluestore;

#[cfg(feature = "cloudhsm")]
pub use aws_sdk_cloudhsm as cloudhsm;

#[cfg(feature = "cloudhsmv2")]
pub use aws_sdk_cloudhsmv2 as cloudhsmv2;

#[cfg(feature = "cloudsearch")]
pub use aws_sdk_cloudsearch as cloudsearch;

#[cfg(feature = "cloudsearchdomain")]
pub use aws_sdk_cloudsearchdomain as cloudsearchdomain;

#[cfg(feature = "cloudtrail")]
pub use aws_sdk_cloudtrail as cloudtrail;

#[cfg(feature = "cloudtraildata")]
pub use aws_sdk_cloudtraildata as cloudtraildata;

#[cfg(feature = "cloudwatch")]
pub use aws_sdk_cloudwatch as cloudwatch;

#[cfg(feature = "cloudwatchevents")]
pub use aws_sdk_cloudwatchevents as cloudwatchevents;

#[cfg(feature = "cloudwatchlogs")]
pub use aws_sdk_cloudwatchlogs as cloudwatchlogs;

#[cfg(feature = "cloudwatchomni")]
pub use aws_sdk_cloudwatchomni as cloudwatchomni;

#[cfg(feature = "codeartifact")]
pub use aws_sdk_codeartifact as codeartifact;

#[cfg(feature = "codebuild")]
pub use aws_sdk_codebuild as codebuild;

#[cfg(feature = "codecatalyst")]
pub use aws_sdk_codecatalyst as codecatalyst;

#[cfg(feature = "codecommit")]
pub use aws_sdk_codecommit as codecommit;

#[cfg(feature = "codeconnections")]
pub use aws_sdk_codeconnections as codeconnections;

#[cfg(feature = "codedeploy")]
pub use aws_sdk_codedeploy as codedeploy;

#[cfg(feature = "codeguruprofiler")]
pub use aws_sdk_codeguruprofiler as codeguruprofiler;

#[cfg(feature = "codegurureviewer")]
pub use aws_sdk_codegurureviewer as codegurureviewer;

#[cfg(feature = "codegurusecurity")]
pub use aws_sdk_codegurusecurity as codegurusecurity;

#[cfg(feature = "codepipeline")]
pub use aws_sdk_codepipeline as codepipeline;

#[cfg(feature = "codestarconnections")]
pub use aws_sdk_codestarconnections as codestarconnections;

#[cfg(feature = "codestarnotifications")]
pub use aws_sdk_codestarnotifications as codestarnotifications;

#[cfg(feature = "cognitoidentity")]
pub use aws_sdk_cognitoidentity as cognitoidentity;

#[cfg(feature = "cognitoidentityprovider")]
pub use aws_sdk_cognitoidentityprovider as cognitoidentityprovider;

#[cfg(feature = "cognitosync")]
pub use aws_sdk_cognitosync as cognitosync;

#[cfg(feature = "comprehend")]
pub use aws_sdk_comprehend as comprehend;

#[cfg(feature = "comprehendmedical")]
pub use aws_sdk_comprehendmedical as comprehendmedical;

#[cfg(feature = "computeoptimizer")]
pub use aws_sdk_computeoptimizer as computeoptimizer;

#[cfg(feature = "computeoptimizerautomation")]
pub use aws_sdk_computeoptimizerautomation as computeoptimizerautomation;

#[cfg(feature = "config")]
pub use aws_sdk_config as config;

#[cfg(feature = "connect")]
pub use aws_sdk_connect as connect;

#[cfg(feature = "connectcampaigns")]
pub use aws_sdk_connectcampaigns as connectcampaigns;

#[cfg(feature = "connectcampaignsv2")]
pub use aws_sdk_connectcampaignsv2 as connectcampaignsv2;

#[cfg(feature = "connectcases")]
pub use aws_sdk_connectcases as connectcases;

#[cfg(feature = "connectcontactlens")]
pub use aws_sdk_connectcontactlens as connectcontactlens;

#[cfg(feature = "connecthealth")]
pub use aws_sdk_connecthealth as connecthealth;

#[cfg(feature = "connectparticipant")]
pub use aws_sdk_connectparticipant as connectparticipant;

#[cfg(feature = "continuum")]
pub use aws_sdk_continuum as continuum;

#[cfg(feature = "controlcatalog")]
pub use aws_sdk_controlcatalog as controlcatalog;

#[cfg(feature = "controltower")]
pub use aws_sdk_controltower as controltower;

#[cfg(feature = "costandusagereport")]
pub use aws_sdk_costandusagereport as costandusagereport;

#[cfg(feature = "costexplorer")]
pub use aws_sdk_costexplorer as costexplorer;

#[cfg(feature = "costoptimizationhub")]
pub use aws_sdk_costoptimizationhub as costoptimizationhub;

#[cfg(feature = "customerprofiles")]
pub use aws_sdk_customerprofiles as customerprofiles;

#[cfg(feature = "databasemigration")]
pub use aws_sdk_databasemigration as databasemigration;

#[cfg(feature = "databrew")]
pub use aws_sdk_databrew as databrew;

#[cfg(feature = "dataexchange")]
pub use aws_sdk_dataexchange as dataexchange;

#[cfg(feature = "datapipeline")]
pub use aws_sdk_datapipeline as datapipeline;

#[cfg(feature = "datasync")]
pub use aws_sdk_datasync as datasync;

#[cfg(feature = "datazone")]
pub use aws_sdk_datazone as datazone;

#[cfg(feature = "dax")]
pub use aws_sdk_dax as dax;

#[cfg(feature = "deadline")]
pub use aws_sdk_deadline as deadline;

#[cfg(feature = "detective")]
pub use aws_sdk_detective as detective;

#[cfg(feature = "devicefarm")]
pub use aws_sdk_devicefarm as devicefarm;

#[cfg(feature = "devopsagent")]
pub use aws_sdk_devopsagent as devopsagent;

#[cfg(feature = "devopsguru")]
pub use aws_sdk_devopsguru as devopsguru;

#[cfg(feature = "directconnect")]
pub use aws_sdk_directconnect as directconnect;

#[cfg(feature = "directory")]
pub use aws_sdk_directory as directory;

#[cfg(feature = "directoryservicedata")]
pub use aws_sdk_directoryservicedata as directoryservicedata;

#[cfg(feature = "dlm")]
pub use aws_sdk_dlm as dlm;

#[cfg(feature = "docdb")]
pub use aws_sdk_docdb as docdb;

#[cfg(feature = "docdbelastic")]
pub use aws_sdk_docdbelastic as docdbelastic;

#[cfg(feature = "drs")]
pub use aws_sdk_drs as drs;

#[cfg(feature = "dsql")]
pub use aws_sdk_dsql as dsql;

#[cfg(feature = "dynamodb")]
pub use aws_sdk_dynamodb as dynamodb;

#[cfg(feature = "dynamodbstreams")]
pub use aws_sdk_dynamodbstreams as dynamodbstreams;

#[cfg(feature = "ebs")]
pub use aws_sdk_ebs as ebs;

#[cfg(feature = "ec2")]
pub use aws_sdk_ec2 as ec2;

#[cfg(feature = "ec2instanceconnect")]
pub use aws_sdk_ec2instanceconnect as ec2instanceconnect;

#[cfg(feature = "ecr")]
pub use aws_sdk_ecr as ecr;

#[cfg(feature = "ecrpublic")]
pub use aws_sdk_ecrpublic as ecrpublic;

#[cfg(feature = "ecs")]
pub use aws_sdk_ecs as ecs;

#[cfg(feature = "efs")]
pub use aws_sdk_efs as efs;

#[cfg(feature = "eks")]
pub use aws_sdk_eks as eks;

#[cfg(feature = "eksauth")]
pub use aws_sdk_eksauth as eksauth;

#[cfg(feature = "elasticache")]
pub use aws_sdk_elasticache as elasticache;

#[cfg(feature = "elasticbeanstalk")]
pub use aws_sdk_elasticbeanstalk as elasticbeanstalk;

#[cfg(feature = "elasticloadbalancing")]
pub use aws_sdk_elasticloadbalancing as elasticloadbalancing;

#[cfg(feature = "elasticloadbalancingv2")]
pub use aws_sdk_elasticloadbalancingv2 as elasticloadbalancingv2;

#[cfg(feature = "elasticsearch")]
pub use aws_sdk_elasticsearch as elasticsearch;

#[cfg(feature = "elastictranscoder")]
pub use aws_sdk_elastictranscoder as elastictranscoder;

#[cfg(feature = "elementalinference")]
pub use aws_sdk_elementalinference as elementalinference;

#[cfg(feature = "emr")]
pub use aws_sdk_emr as emr;

#[cfg(feature = "emrcontainers")]
pub use aws_sdk_emrcontainers as emrcontainers;

#[cfg(feature = "emrserverless")]
pub use aws_sdk_emrserverless as emrserverless;

#[cfg(feature = "endusermessaging")]
pub use aws_sdk_endusermessaging as endusermessaging;

#[cfg(feature = "entityresolution")]
pub use aws_sdk_entityresolution as entityresolution;

#[cfg(feature = "eventbridge")]
pub use aws_sdk_eventbridge as eventbridge;

#[cfg(feature = "eventbridgev2")]
pub use aws_sdk_eventbridgev2 as eventbridgev2;

#[cfg(feature = "evs")]
pub use aws_sdk_evs as evs;

#[cfg(feature = "firehose")]
pub use aws_sdk_firehose as firehose;

#[cfg(feature = "fis")]
pub use aws_sdk_fis as fis;

#[cfg(feature = "fms")]
pub use aws_sdk_fms as fms;

#[cfg(feature = "forecast")]
pub use aws_sdk_forecast as forecast;

#[cfg(feature = "forecastquery")]
pub use aws_sdk_forecastquery as forecastquery;

#[cfg(feature = "frauddetector")]
pub use aws_sdk_frauddetector as frauddetector;

#[cfg(feature = "freetier")]
pub use aws_sdk_freetier as freetier;

#[cfg(feature = "fsx")]
pub use aws_sdk_fsx as fsx;

#[cfg(feature = "gamelift")]
pub use aws_sdk_gamelift as gamelift;

#[cfg(feature = "gameliftstreams")]
pub use aws_sdk_gameliftstreams as gameliftstreams;

#[cfg(feature = "geomaps")]
pub use aws_sdk_geomaps as geomaps;

#[cfg(feature = "geoplaces")]
pub use aws_sdk_geoplaces as geoplaces;

#[cfg(feature = "georoutes")]
pub use aws_sdk_georoutes as georoutes;

#[cfg(feature = "glacier")]
pub use aws_sdk_glacier as glacier;

#[cfg(feature = "globalaccelerator")]
pub use aws_sdk_globalaccelerator as globalaccelerator;

#[cfg(feature = "glue")]
pub use aws_sdk_glue as glue;

#[cfg(feature = "grafana")]
pub use aws_sdk_grafana as grafana;

#[cfg(feature = "greengrass")]
pub use aws_sdk_greengrass as greengrass;

#[cfg(feature = "greengrassv2")]
pub use aws_sdk_greengrassv2 as greengrassv2;

#[cfg(feature = "groundstation")]
pub use aws_sdk_groundstation as groundstation;

#[cfg(feature = "guardduty")]
pub use aws_sdk_guardduty as guardduty;

#[cfg(feature = "health")]
pub use aws_sdk_health as health;

#[cfg(feature = "healthlake")]
pub use aws_sdk_healthlake as healthlake;

#[cfg(feature = "iam")]
pub use aws_sdk_iam as iam;

#[cfg(feature = "iamtoolbox")]
pub use aws_sdk_iamtoolbox as iamtoolbox;

#[cfg(feature = "identitystore")]
pub use aws_sdk_identitystore as identitystore;

#[cfg(feature = "imagebuilder")]
pub use aws_sdk_imagebuilder as imagebuilder;

#[cfg(feature = "inspector")]
pub use aws_sdk_inspector as inspector;

#[cfg(feature = "inspector2")]
pub use aws_sdk_inspector2 as inspector2;

#[cfg(feature = "inspectorscan")]
pub use aws_sdk_inspectorscan as inspectorscan;

#[cfg(feature = "interconnect")]
pub use aws_sdk_interconnect as interconnect;

#[cfg(feature = "internetmonitor")]
pub use aws_sdk_internetmonitor as internetmonitor;

#[cfg(feature = "invoicing")]
pub use aws_sdk_invoicing as invoicing;

#[cfg(feature = "iot")]
pub use aws_sdk_iot as iot;

#[cfg(feature = "iotdataplane")]
pub use aws_sdk_iotdataplane as iotdataplane;

#[cfg(feature = "iotdeviceadvisor")]
pub use aws_sdk_iotdeviceadvisor as iotdeviceadvisor;

#[cfg(feature = "iotfleetwise")]
pub use aws_sdk_iotfleetwise as iotfleetwise;

#[cfg(feature = "iotjobsdataplane")]
pub use aws_sdk_iotjobsdataplane as iotjobsdataplane;

#[cfg(feature = "iotmanagedintegrations")]
pub use aws_sdk_iotmanagedintegrations as iotmanagedintegrations;

#[cfg(feature = "iotsecuretunneling")]
pub use aws_sdk_iotsecuretunneling as iotsecuretunneling;

#[cfg(feature = "iotsitewise")]
pub use aws_sdk_iotsitewise as iotsitewise;

#[cfg(feature = "iotthingsgraph")]
pub use aws_sdk_iotthingsgraph as iotthingsgraph;

#[cfg(feature = "iottwinmaker")]
pub use aws_sdk_iottwinmaker as iottwinmaker;

#[cfg(feature = "iotwireless")]
pub use aws_sdk_iotwireless as iotwireless;

#[cfg(feature = "ivs")]
pub use aws_sdk_ivs as ivs;

#[cfg(feature = "ivschat")]
pub use aws_sdk_ivschat as ivschat;

#[cfg(feature = "ivsrealtime")]
pub use aws_sdk_ivsrealtime as ivsrealtime;

#[cfg(feature = "kafka")]
pub use aws_sdk_kafka as kafka;

#[cfg(feature = "kafkaconnect")]
pub use aws_sdk_kafkaconnect as kafkaconnect;

#[cfg(feature = "kendra")]
pub use aws_sdk_kendra as kendra;

#[cfg(feature = "kendraranking")]
pub use aws_sdk_kendraranking as kendraranking;

#[cfg(feature = "keyspaces")]
pub use aws_sdk_keyspaces as keyspaces;

#[cfg(feature = "keyspacesstreams")]
pub use aws_sdk_keyspacesstreams as keyspacesstreams;

#[cfg(feature = "kinesis")]
pub use aws_sdk_kinesis as kinesis;

#[cfg(feature = "kinesisanalytics")]
pub use aws_sdk_kinesisanalytics as kinesisanalytics;

#[cfg(feature = "kinesisanalyticsv2")]
pub use aws_sdk_kinesisanalyticsv2 as kinesisanalyticsv2;

#[cfg(feature = "kinesisvideo")]
pub use aws_sdk_kinesisvideo as kinesisvideo;

#[cfg(feature = "kinesisvideoarchivedmedia")]
pub use aws_sdk_kinesisvideoarchivedmedia as kinesisvideoarchivedmedia;

#[cfg(feature = "kinesisvideomedia")]
pub use aws_sdk_kinesisvideomedia as kinesisvideomedia;

#[cfg(feature = "kinesisvideosignaling")]
pub use aws_sdk_kinesisvideosignaling as kinesisvideosignaling;

#[cfg(feature = "kinesisvideowebrtcstorage")]
pub use aws_sdk_kinesisvideowebrtcstorage as kinesisvideowebrtcstorage;

#[cfg(feature = "kms")]
pub use aws_sdk_kms as kms;

#[cfg(feature = "lakeformation")]
pub use aws_sdk_lakeformation as lakeformation;

#[cfg(feature = "lambda")]
pub use aws_sdk_lambda as lambda;

#[cfg(feature = "lambdacore")]
pub use aws_sdk_lambdacore as lambdacore;

#[cfg(feature = "lambdamicrovms")]
pub use aws_sdk_lambdamicrovms as lambdamicrovms;

#[cfg(feature = "lambdaweb")]
pub use aws_sdk_lambdaweb as lambdaweb;

#[cfg(feature = "launchwizard")]
pub use aws_sdk_launchwizard as launchwizard;

#[cfg(feature = "lexmodelbuilding")]
pub use aws_sdk_lexmodelbuilding as lexmodelbuilding;

#[cfg(feature = "lexmodelsv2")]
pub use aws_sdk_lexmodelsv2 as lexmodelsv2;

#[cfg(feature = "lexruntime")]
pub use aws_sdk_lexruntime as lexruntime;

#[cfg(feature = "lexruntimev2")]
pub use aws_sdk_lexruntimev2 as lexruntimev2;

#[cfg(feature = "licensemanager")]
pub use aws_sdk_licensemanager as licensemanager;

#[cfg(feature = "licensemanagerlinuxsubscriptions")]
pub use aws_sdk_licensemanagerlinuxsubscriptions as licensemanagerlinuxsubscriptions;

#[cfg(feature = "licensemanagerusersubscriptions")]
pub use aws_sdk_licensemanagerusersubscriptions as licensemanagerusersubscriptions;

#[cfg(feature = "lightsail")]
pub use aws_sdk_lightsail as lightsail;

#[cfg(feature = "location")]
pub use aws_sdk_location as location;

#[cfg(feature = "lookoutequipment")]
pub use aws_sdk_lookoutequipment as lookoutequipment;

#[cfg(feature = "m2")]
pub use aws_sdk_m2 as m2;

#[cfg(feature = "machinelearning")]
pub use aws_sdk_machinelearning as machinelearning;

#[cfg(feature = "macie2")]
pub use aws_sdk_macie2 as macie2;

#[cfg(feature = "mailmanager")]
pub use aws_sdk_mailmanager as mailmanager;

#[cfg(feature = "managedblockchain")]
pub use aws_sdk_managedblockchain as managedblockchain;

#[cfg(feature = "managedblockchainquery")]
pub use aws_sdk_managedblockchainquery as managedblockchainquery;

#[cfg(feature = "marketplaceagreement")]
pub use aws_sdk_marketplaceagreement as marketplaceagreement;

#[cfg(feature = "marketplacecatalog")]
pub use aws_sdk_marketplacecatalog as marketplacecatalog;

#[cfg(feature = "marketplacecommerceanalytics")]
pub use aws_sdk_marketplacecommerceanalytics as marketplacecommerceanalytics;

#[cfg(feature = "marketplacedeployment")]
pub use aws_sdk_marketplacedeployment as marketplacedeployment;

#[cfg(feature = "marketplacediscovery")]
pub use aws_sdk_marketplacediscovery as marketplacediscovery;

#[cfg(feature = "marketplaceentitlement")]
pub use aws_sdk_marketplaceentitlement as marketplaceentitlement;

#[cfg(feature = "marketplacemetering")]
pub use aws_sdk_marketplacemetering as marketplacemetering;

#[cfg(feature = "marketplacereporting")]
pub use aws_sdk_marketplacereporting as marketplacereporting;

#[cfg(feature = "mediaconnect")]
pub use aws_sdk_mediaconnect as mediaconnect;

#[cfg(feature = "mediaconvert")]
pub use aws_sdk_mediaconvert as mediaconvert;

#[cfg(feature = "medialive")]
pub use aws_sdk_medialive as medialive;

#[cfg(feature = "mediapackage")]
pub use aws_sdk_mediapackage as mediapackage;

#[cfg(feature = "mediapackagev2")]
pub use aws_sdk_mediapackagev2 as mediapackagev2;

#[cfg(feature = "mediapackagevod")]
pub use aws_sdk_mediapackagevod as mediapackagevod;

#[cfg(feature = "mediastore")]
pub use aws_sdk_mediastore as mediastore;

#[cfg(feature = "mediastoredata")]
pub use aws_sdk_mediastoredata as mediastoredata;

#[cfg(feature = "mediatailor")]
pub use aws_sdk_mediatailor as mediatailor;

#[cfg(feature = "medicalimaging")]
pub use aws_sdk_medicalimaging as medicalimaging;

#[cfg(feature = "memorydb")]
pub use aws_sdk_memorydb as memorydb;

#[cfg(feature = "mgn")]
pub use aws_sdk_mgn as mgn;

#[cfg(feature = "migrationhub")]
pub use aws_sdk_migrationhub as migrationhub;

#[cfg(feature = "migrationhubconfig")]
pub use aws_sdk_migrationhubconfig as migrationhubconfig;

#[cfg(feature = "migrationhuborchestrator")]
pub use aws_sdk_migrationhuborchestrator as migrationhuborchestrator;

#[cfg(feature = "migrationhubrefactorspaces")]
pub use aws_sdk_migrationhubrefactorspaces as migrationhubrefactorspaces;

#[cfg(feature = "migrationhubstrategy")]
pub use aws_sdk_migrationhubstrategy as migrationhubstrategy;

#[cfg(feature = "mpa")]
pub use aws_sdk_mpa as mpa;

#[cfg(feature = "mq")]
pub use aws_sdk_mq as mq;

#[cfg(feature = "mturk")]
pub use aws_sdk_mturk as mturk;

#[cfg(feature = "mwaa")]
pub use aws_sdk_mwaa as mwaa;

#[cfg(feature = "mwaaserverless")]
pub use aws_sdk_mwaaserverless as mwaaserverless;

#[cfg(feature = "neptune")]
pub use aws_sdk_neptune as neptune;

#[cfg(feature = "neptunedata")]
pub use aws_sdk_neptunedata as neptunedata;

#[cfg(feature = "neptunegraph")]
pub use aws_sdk_neptunegraph as neptunegraph;

#[cfg(feature = "networkfirewall")]
pub use aws_sdk_networkfirewall as networkfirewall;

#[cfg(feature = "networkflowmonitor")]
pub use aws_sdk_networkflowmonitor as networkflowmonitor;

#[cfg(feature = "networkmanager")]
pub use aws_sdk_networkmanager as networkmanager;

#[cfg(feature = "networkmonitor")]
pub use aws_sdk_networkmonitor as networkmonitor;

#[cfg(feature = "networksecuritymanager")]
pub use aws_sdk_networksecuritymanager as networksecuritymanager;

#[cfg(feature = "notifications")]
pub use aws_sdk_notifications as notifications;

#[cfg(feature = "notificationscontacts")]
pub use aws_sdk_notificationscontacts as notificationscontacts;

#[cfg(feature = "novaact")]
pub use aws_sdk_novaact as novaact;

#[cfg(feature = "oam")]
pub use aws_sdk_oam as oam;

#[cfg(feature = "observabilityadmin")]
pub use aws_sdk_observabilityadmin as observabilityadmin;

#[cfg(feature = "odb")]
pub use aws_sdk_odb as odb;

#[cfg(feature = "omics")]
pub use aws_sdk_omics as omics;

#[cfg(feature = "opensearch")]
pub use aws_sdk_opensearch as opensearch;

#[cfg(feature = "opensearchserverless")]
pub use aws_sdk_opensearchserverless as opensearchserverless;

#[cfg(feature = "organizations")]
pub use aws_sdk_organizations as organizations;

#[cfg(feature = "osis")]
pub use aws_sdk_osis as osis;

#[cfg(feature = "outposts")]
pub use aws_sdk_outposts as outposts;

#[cfg(feature = "partnercentralaccount")]
pub use aws_sdk_partnercentralaccount as partnercentralaccount;

#[cfg(feature = "partnercentralbenefits")]
pub use aws_sdk_partnercentralbenefits as partnercentralbenefits;

#[cfg(feature = "partnercentralchannel")]
pub use aws_sdk_partnercentralchannel as partnercentralchannel;

#[cfg(feature = "partnercentralrevenuemeasurement")]
pub use aws_sdk_partnercentralrevenuemeasurement as partnercentralrevenuemeasurement;

#[cfg(feature = "partnercentralselling")]
pub use aws_sdk_partnercentralselling as partnercentralselling;

#[cfg(feature = "paymentcryptography")]
pub use aws_sdk_paymentcryptography as paymentcryptography;

#[cfg(feature = "paymentcryptographydata")]
pub use aws_sdk_paymentcryptographydata as paymentcryptographydata;

#[cfg(feature = "pcaconnectorad")]
pub use aws_sdk_pcaconnectorad as pcaconnectorad;

#[cfg(feature = "pcaconnectorscep")]
pub use aws_sdk_pcaconnectorscep as pcaconnectorscep;

#[cfg(feature = "pcs")]
pub use aws_sdk_pcs as pcs;

#[cfg(feature = "personalize")]
pub use aws_sdk_personalize as personalize;

#[cfg(feature = "personalizeevents")]
pub use aws_sdk_personalizeevents as personalizeevents;

#[cfg(feature = "personalizeruntime")]
pub use aws_sdk_personalizeruntime as personalizeruntime;

#[cfg(feature = "pi")]
pub use aws_sdk_pi as pi;

#[cfg(feature = "pinpoint")]
pub use aws_sdk_pinpoint as pinpoint;

#[cfg(feature = "pinpointemail")]
pub use aws_sdk_pinpointemail as pinpointemail;

#[cfg(feature = "pinpointsmsvoice")]
pub use aws_sdk_pinpointsmsvoice as pinpointsmsvoice;

#[cfg(feature = "pinpointsmsvoicev2")]
pub use aws_sdk_pinpointsmsvoicev2 as pinpointsmsvoicev2;

#[cfg(feature = "pipes")]
pub use aws_sdk_pipes as pipes;

#[cfg(feature = "polly")]
pub use aws_sdk_polly as polly;

#[cfg(feature = "pricing")]
pub use aws_sdk_pricing as pricing;

#[cfg(feature = "pricingplanmanager")]
pub use aws_sdk_pricingplanmanager as pricingplanmanager;

#[cfg(feature = "proton")]
pub use aws_sdk_proton as proton;

#[cfg(feature = "qapps")]
pub use aws_sdk_qapps as qapps;

#[cfg(feature = "qbusiness")]
pub use aws_sdk_qbusiness as qbusiness;

#[cfg(feature = "qconnect")]
pub use aws_sdk_qconnect as qconnect;

#[cfg(feature = "quicksight")]
pub use aws_sdk_quicksight as quicksight;

#[cfg(feature = "ram")]
pub use aws_sdk_ram as ram;

#[cfg(feature = "rbin")]
pub use aws_sdk_rbin as rbin;

#[cfg(feature = "rds")]
pub use aws_sdk_rds as rds;

#[cfg(feature = "rdsdata")]
pub use aws_sdk_rdsdata as rdsdata;

#[cfg(feature = "redshift")]
pub use aws_sdk_redshift as redshift;

#[cfg(feature = "redshiftdata")]
pub use aws_sdk_redshiftdata as redshiftdata;

#[cfg(feature = "redshiftserverless")]
pub use aws_sdk_redshiftserverless as redshiftserverless;

#[cfg(feature = "rekognition")]
pub use aws_sdk_rekognition as rekognition;

#[cfg(feature = "repostspace")]
pub use aws_sdk_repostspace as repostspace;

#[cfg(feature = "resiliencehub")]
pub use aws_sdk_resiliencehub as resiliencehub;

#[cfg(feature = "resiliencehubv2")]
pub use aws_sdk_resiliencehubv2 as resiliencehubv2;

#[cfg(feature = "resourceexplorer2")]
pub use aws_sdk_resourceexplorer2 as resourceexplorer2;

#[cfg(feature = "resourcegroups")]
pub use aws_sdk_resourcegroups as resourcegroups;

#[cfg(feature = "resourcegroupstagging")]
pub use aws_sdk_resourcegroupstagging as resourcegroupstagging;

#[cfg(feature = "rolesanywhere")]
pub use aws_sdk_rolesanywhere as rolesanywhere;

#[cfg(feature = "route53")]
pub use aws_sdk_route53 as route53;

#[cfg(feature = "route53domains")]
pub use aws_sdk_route53domains as route53domains;

#[cfg(feature = "route53globalresolver")]
pub use aws_sdk_route53globalresolver as route53globalresolver;

#[cfg(feature = "route53profiles")]
pub use aws_sdk_route53profiles as route53profiles;

#[cfg(feature = "route53recoverycluster")]
pub use aws_sdk_route53recoverycluster as route53recoverycluster;

#[cfg(feature = "route53recoverycontrolconfig")]
pub use aws_sdk_route53recoverycontrolconfig as route53recoverycontrolconfig;

#[cfg(feature = "route53recoveryreadiness")]
pub use aws_sdk_route53recoveryreadiness as route53recoveryreadiness;

#[cfg(feature = "route53resolver")]
pub use aws_sdk_route53resolver as route53resolver;

#[cfg(feature = "rtbfabric")]
pub use aws_sdk_rtbfabric as rtbfabric;

#[cfg(feature = "rum")]
pub use aws_sdk_rum as rum;

#[cfg(feature = "s3")]
pub use aws_sdk_s3 as s3;

#[cfg(feature = "s3control")]
pub use aws_sdk_s3control as s3control;

#[cfg(feature = "s3files")]
pub use aws_sdk_s3files as s3files;

#[cfg(feature = "s3outposts")]
pub use aws_sdk_s3outposts as s3outposts;

#[cfg(feature = "s3tables")]
pub use aws_sdk_s3tables as s3tables;

#[cfg(feature = "s3vectors")]
pub use aws_sdk_s3vectors as s3vectors;

#[cfg(feature = "sagemaker")]
pub use aws_sdk_sagemaker as sagemaker;

#[cfg(feature = "sagemakera2iruntime")]
pub use aws_sdk_sagemakera2iruntime as sagemakera2iruntime;

#[cfg(feature = "sagemakeredge")]
pub use aws_sdk_sagemakeredge as sagemakeredge;

#[cfg(feature = "sagemakerfeaturestoreruntime")]
pub use aws_sdk_sagemakerfeaturestoreruntime as sagemakerfeaturestoreruntime;

#[cfg(feature = "sagemakergeospatial")]
pub use aws_sdk_sagemakergeospatial as sagemakergeospatial;

#[cfg(feature = "sagemakerjobruntime")]
pub use aws_sdk_sagemakerjobruntime as sagemakerjobruntime;

#[cfg(feature = "sagemakermetrics")]
pub use aws_sdk_sagemakermetrics as sagemakermetrics;

#[cfg(feature = "sagemakerruntime")]
pub use aws_sdk_sagemakerruntime as sagemakerruntime;

#[cfg(feature = "sagemakerruntimehttp2")]
pub use aws_sdk_sagemakerruntimehttp2 as sagemakerruntimehttp2;

#[cfg(feature = "sagemakertrainingsessionruntime")]
pub use aws_sdk_sagemakertrainingsessionruntime as sagemakertrainingsessionruntime;

#[cfg(feature = "savingsplans")]
pub use aws_sdk_savingsplans as savingsplans;

#[cfg(feature = "scheduler")]
pub use aws_sdk_scheduler as scheduler;

#[cfg(feature = "schemas")]
pub use aws_sdk_schemas as schemas;

#[cfg(feature = "secretsmanager")]
pub use aws_sdk_secretsmanager as secretsmanager;

#[cfg(feature = "securityagent")]
pub use aws_sdk_securityagent as securityagent;

#[cfg(feature = "securityhub")]
pub use aws_sdk_securityhub as securityhub;

#[cfg(feature = "securityir")]
pub use aws_sdk_securityir as securityir;

#[cfg(feature = "securitylake")]
pub use aws_sdk_securitylake as securitylake;

#[cfg(feature = "serverlessapplicationrepository")]
pub use aws_sdk_serverlessapplicationrepository as serverlessapplicationrepository;

#[cfg(feature = "servicecatalog")]
pub use aws_sdk_servicecatalog as servicecatalog;

#[cfg(feature = "servicecatalogappregistry")]
pub use aws_sdk_servicecatalogappregistry as servicecatalogappregistry;

#[cfg(feature = "servicediscovery")]
pub use aws_sdk_servicediscovery as servicediscovery;

#[cfg(feature = "servicequotas")]
pub use aws_sdk_servicequotas as servicequotas;

#[cfg(feature = "ses")]
pub use aws_sdk_ses as ses;

#[cfg(feature = "sesv2")]
pub use aws_sdk_sesv2 as sesv2;

#[cfg(feature = "sfn")]
pub use aws_sdk_sfn as sfn;

#[cfg(feature = "shield")]
pub use aws_sdk_shield as shield;

#[cfg(feature = "signer")]
pub use aws_sdk_signer as signer;

#[cfg(feature = "signerdata")]
pub use aws_sdk_signerdata as signerdata;

#[cfg(feature = "signin")]
pub use aws_sdk_signin as signin;

#[cfg(feature = "simpledbv2")]
pub use aws_sdk_simpledbv2 as simpledbv2;

#[cfg(feature = "snowball")]
pub use aws_sdk_snowball as snowball;

#[cfg(feature = "snowdevicemanagement")]
pub use aws_sdk_snowdevicemanagement as snowdevicemanagement;

#[cfg(feature = "sns")]
pub use aws_sdk_sns as sns;

#[cfg(feature = "socialmessaging")]
pub use aws_sdk_socialmessaging as socialmessaging;

#[cfg(feature = "sqs")]
pub use aws_sdk_sqs as sqs;

#[cfg(feature = "ssm")]
pub use aws_sdk_ssm as ssm;

#[cfg(feature = "ssmcontacts")]
pub use aws_sdk_ssmcontacts as ssmcontacts;

#[cfg(feature = "ssmguiconnect")]
pub use aws_sdk_ssmguiconnect as ssmguiconnect;

#[cfg(feature = "ssmincidents")]
pub use aws_sdk_ssmincidents as ssmincidents;

#[cfg(feature = "ssmquicksetup")]
pub use aws_sdk_ssmquicksetup as ssmquicksetup;

#[cfg(feature = "ssmsap")]
pub use aws_sdk_ssmsap as ssmsap;

#[cfg(feature = "sso")]
pub use aws_sdk_sso as sso;

#[cfg(feature = "ssoadmin")]
pub use aws_sdk_ssoadmin as ssoadmin;

#[cfg(feature = "ssooidc")]
pub use aws_sdk_ssooidc as ssooidc;

#[cfg(feature = "storagegateway")]
pub use aws_sdk_storagegateway as storagegateway;

#[cfg(feature = "sts")]
pub use aws_sdk_sts as sts;

#[cfg(feature = "supplychain")]
pub use aws_sdk_supplychain as supplychain;

#[cfg(feature = "support")]
pub use aws_sdk_support as support;

#[cfg(feature = "supportapp")]
pub use aws_sdk_supportapp as supportapp;

#[cfg(feature = "supportauthz")]
pub use aws_sdk_supportauthz as supportauthz;

#[cfg(feature = "sustainability")]
pub use aws_sdk_sustainability as sustainability;

#[cfg(feature = "swf")]
pub use aws_sdk_swf as swf;

#[cfg(feature = "synthetics")]
pub use aws_sdk_synthetics as synthetics;

#[cfg(feature = "taxsettings")]
pub use aws_sdk_taxsettings as taxsettings;

#[cfg(feature = "textract")]
pub use aws_sdk_textract as textract;

#[cfg(feature = "timestreaminfluxdb")]
pub use aws_sdk_timestreaminfluxdb as timestreaminfluxdb;

#[cfg(feature = "timestreamquery")]
pub use aws_sdk_timestreamquery as timestreamquery;

#[cfg(feature = "timestreamwrite")]
pub use aws_sdk_timestreamwrite as timestreamwrite;

#[cfg(feature = "transcribe")]
pub use aws_sdk_transcribe as transcribe;

#[cfg(feature = "transcribestreaming")]
pub use aws_sdk_transcribestreaming as transcribestreaming;

#[cfg(feature = "transfer")]
pub use aws_sdk_transfer as transfer;

#[cfg(feature = "translate")]
pub use aws_sdk_translate as translate;

#[cfg(feature = "trustedadvisor")]
pub use aws_sdk_trustedadvisor as trustedadvisor;

#[cfg(feature = "uxc")]
pub use aws_sdk_uxc as uxc;

#[cfg(feature = "verifiedpermissions")]
pub use aws_sdk_verifiedpermissions as verifiedpermissions;

#[cfg(feature = "voiceid")]
pub use aws_sdk_voiceid as voiceid;

#[cfg(feature = "vpclattice")]
pub use aws_sdk_vpclattice as vpclattice;

#[cfg(feature = "waf")]
pub use aws_sdk_waf as waf;

#[cfg(feature = "wafregional")]
pub use aws_sdk_wafregional as wafregional;

#[cfg(feature = "wafv2")]
pub use aws_sdk_wafv2 as wafv2;

#[cfg(feature = "wellarchitected")]
pub use aws_sdk_wellarchitected as wellarchitected;

#[cfg(feature = "wickr")]
pub use aws_sdk_wickr as wickr;

#[cfg(feature = "wisdom")]
pub use aws_sdk_wisdom as wisdom;

#[cfg(feature = "workdocs")]
pub use aws_sdk_workdocs as workdocs;

#[cfg(feature = "workmail")]
pub use aws_sdk_workmail as workmail;

#[cfg(feature = "workmailmessageflow")]
pub use aws_sdk_workmailmessageflow as workmailmessageflow;

#[cfg(feature = "workspaces")]
pub use aws_sdk_workspaces as workspaces;

#[cfg(feature = "workspacesinstances")]
pub use aws_sdk_workspacesinstances as workspacesinstances;

#[cfg(feature = "workspacesthinclient")]
pub use aws_sdk_workspacesthinclient as workspacesthinclient;

#[cfg(feature = "workspacesweb")]
pub use aws_sdk_workspacesweb as workspacesweb;

#[cfg(feature = "xray")]
pub use aws_sdk_xray as xray;
