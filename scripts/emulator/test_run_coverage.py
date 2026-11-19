"""A successful prerequisite read cannot stand in for the root scan's read."""

import unittest

from run import type_operation_coverage


class CommandScopedCoverage(unittest.TestCase):
    @staticmethod
    def request(phase, operation, status=200, service="cloudcontrol", name="AWS::EKS::Cluster"):
        return {"phase": phase, "operation": operation, "http_status": status,
                "service": service, "type_name": name}

    def test_parent_success_never_masks_failed_root_read(self):
        rows = [self.request("rust:cloud-control", "ListResources"),
                self.request("rust:cloud-control", "GetResource", 403),
                self.request("rust:parents", "GetResource")]
        descriptors = [{"type_name": "AWS::EKS::Cluster"}]
        root = type_operation_coverage(rows, descriptors, "rust:cloud-control")
        parents = type_operation_coverage(rows, descriptors, "rust:parents")
        self.assertEqual(root, [{"type_name": "AWS::EKS::Cluster", "operations": {
            "ListResources": {"200": 1}, "GetResource": {"403": 1}}}])
        self.assertEqual(parents[0]["operations"]["GetResource"], {"200": 1})

    def test_seeding_probes_other_services_and_other_types_cannot_supply_a_read(self):
        rows = [self.request("seed:seed_catalog", "GetResource"),
                self.request("rust-probe:branch-probes", "GetResource"),
                self.request("rust:parents-extra", "GetResource"),
                self.request("rust:parents", "GetResource", service="eks"),
                self.request("rust:parents", "GetResource", name="AWS::EKS::Addon")]
        result = type_operation_coverage(rows, [{"type_name": "AWS::EKS::Cluster"}], "rust:parents")
        self.assertEqual(result[0]["operations"], {"ListResources": {}, "GetResource": {}})

    def test_retry_histograms_preserve_failures_and_unseen_descriptors(self):
        rows = [self.request("rust:parents", "ListResources", 429),
                self.request("rust:parents", "ListResources"),
                self.request("rust:parents", "ListResources"),
                self.request("rust:parents", "GetResource", None)]
        descriptors = [{"type_name": "AWS::EKS::Cluster"}, {"type_name": "AWS::EKS::Addon"}]
        result = type_operation_coverage(rows, descriptors, "rust:parents")
        self.assertEqual(result[0]["operations"], {"ListResources": {"429": 1, "200": 2},
                                                  "GetResource": {"None": 1}})
        self.assertEqual(result[1], {"type_name": "AWS::EKS::Addon", "operations": {
            "ListResources": {}, "GetResource": {}}})


if __name__ == "__main__":
    unittest.main()
