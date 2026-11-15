use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "ivs-realtime", operation: "ListEncoderConfigurations", kind: "AWS::IVS::EncoderConfiguration" },
    OperationMapping { service: "ivs-realtime", operation: "ListStorageConfigurations", kind: "AWS::IVS::StorageConfiguration" },
    OperationMapping { service: "ivschat", operation: "ListRooms", kind: "AWS::IVSChat::Room" },
    OperationMapping { service: "ivschat", operation: "ListLoggingConfigurations", kind: "AWS::IVSChat::LoggingConfiguration" },
    OperationMapping { service: "kafka", operation: "ListClustersV2", kind: "AWS::MSK::Cluster" },
    OperationMapping { service: "kafka", operation: "ListConfigurations", kind: "AWS::MSK::Configuration" },
    OperationMapping { service: "kafka", operation: "ListReplicators", kind: "AWS::MSK::Replicator" },
    OperationMapping { service: "kafka", operation: "ListVpcConnections", kind: "AWS::MSK::VpcConnection" },
    OperationMapping { service: "kafkaconnect", operation: "ListConnectors", kind: "AWS::KafkaConnect::Connector" },
    OperationMapping { service: "kafkaconnect", operation: "ListCustomPlugins", kind: "AWS::KafkaConnect::CustomPlugin" },
    OperationMapping { service: "kafkaconnect", operation: "ListWorkerConfigurations", kind: "AWS::KafkaConnect::WorkerConfiguration" },
    OperationMapping { service: "kafkaconnect", operation: "DescribeCustomPlugin", kind: "AWS::KafkaConnect::CustomPlugin" },
    OperationMapping { service: "kendra", operation: "ListIndices", kind: "AWS::Kendra::Index" },
    OperationMapping { service: "kendra-ranking", operation: "ListRescoreExecutionPlans", kind: "AWS::KendraRanking::ExecutionPlan" },
    OperationMapping { service: "keyspaces", operation: "ListKeyspaces", kind: "AWS::Cassandra::Keyspace" },
    OperationMapping { service: "keyspaces", operation: "ListTables", kind: "AWS::Cassandra::Table" },
    OperationMapping { service: "keyspaces", operation: "GetTable", kind: "AWS::Cassandra::Table" },
    OperationMapping { service: "keyspacesstreams", operation: "ListStreams", kind: "AWS::ApiOnly::KeyspacesStream" },
    OperationMapping { service: "kinesis", operation: "ListStreams", kind: "AWS::Kinesis::Stream" },
    OperationMapping { service: "kinesis", operation: "ListChannels", kind: "AWS::Kinesis::Channel" },
];
