// Explicit dispositions from .local/aws-models/models/fsx/service/2018-03-01/fsx-2018-03-01.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    ("UpdateFileSystem", "Requires FileSystemId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Use this operation to update the configuration of an existing Amazon FSx file system."),
    ("UpdateSharedVpcConfiguration", "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Configures whether participant accounts in your organization can create Amazon FSx for NetApp ONTAP Multi-AZ file systems in subnets that are shared by a virtual private cloud (VPC) owner."),
    ("UpdateSnapshot", "Requires Name, SnapshotId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the name of an Amazon FSx for OpenZFS snapshot."),
    ("UpdateStorageVirtualMachine", "Requires StorageVirtualMachineId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates an FSx for ONTAP storage virtual machine (SVM)."),
    ("UpdateVolume", "Requires VolumeId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the configuration of an Amazon FSx for NetApp ONTAP or Amazon FSx for OpenZFS volume."),
];
