"""Independent SDK feature contract: preserve signing/protocol defaults."""
import importlib.util
import io
from pathlib import Path
import tarfile
import tempfile
import tomllib
import unittest

SPEC = importlib.util.spec_from_file_location(
    "sdk_generator", Path(__file__).resolve().parents[1] / "generate_sdk_catalog.py")
generator = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(generator)

CLIENT_FEATURES = ["sigv4a", "http-1x", "default-https-client", "rt-tokio"]
PROVIDER_MANIFEST = b'''[package]
name = "aws-sdk-example"
version = "1.0.0"
[features]
default = ["sigv4a", "http-1x", "rustls", "default-https-client", "rt-tokio"]
'''


class SdkFeatureContract(unittest.TestCase):
    def test_generated_dependency_preserves_signing_and_protocol_features(self):
        # Given: a provider SDK with independently specified non-TLS defaults.
        service = {"name": "aws-sdk-example", "feature": "example", "version": "1.0.0",
                   "client_features": CLIENT_FEATURES}
        # When: generate the installable Cargo dependency.
        manifest = tomllib.loads(generator.manifest({"services": [service]}))
        dependency = manifest["dependencies"]["aws-sdk-example"]
        # Then: signing/protocol behavior remains enabled with modern TLS only.
        self.assertEqual(dependency["features"], CLIENT_FEATURES)
        self.assertIs(dependency["default-features"], False)
        self.assertEqual(dependency["version"], "=1.0.0")

    def test_archive_oracle_rejects_missing_signing_or_protocol_features(self):
        # Given: the provider's independent manifest in a real package archive.
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "example.crate"
            with tarfile.open(path, "w:gz") as archive:
                entry = tarfile.TarInfo("aws-sdk-example-1.0.0/Cargo.toml")
                entry.size = len(PROVIDER_MANIFEST)
                archive.addfile(entry, io.BytesIO(PROVIDER_MANIFEST))
            service = {"name": "aws-sdk-example", "version": "1.0.0",
                       "client_features": CLIENT_FEATURES}
            generator.verify_client_features(path, service)
            # When/Then: dropping either capability fails the archive comparison.
            for missing in ["sigv4a", "http-1x"]:
                with self.subTest(missing=missing):
                    changed = {**service, "client_features": [value for value in CLIENT_FEATURES
                                                             if value != missing]}
                    with self.assertRaisesRegex(ValueError, "differ from verified archive"):
                        generator.verify_client_features(path, changed)


if __name__ == "__main__":
    unittest.main()
