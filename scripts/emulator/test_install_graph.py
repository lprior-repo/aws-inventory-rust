"""Installer ownership and hostile archive tests; no network, compiler or server."""

from __future__ import annotations

import hashlib
import io
import json
from pathlib import Path
import signal
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import Mock, call, patch

import install_graph as installer


class InstallerFixture(unittest.TestCase):
    def setUp(self):
        # The workspace may have much more room than the execution host's /tmp.
        self.directory = tempfile.TemporaryDirectory(prefix="installer-test-",
                                                     dir=Path(__file__).resolve().parents[2])
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)

    def archive(self, members):
        path = self.root / "input.tar.gz"
        with tarfile.open(path, "w:gz") as output:
            for name, kind, value in members:
                member = tarfile.TarInfo(name)
                member.type = kind
                if kind == tarfile.REGTYPE:
                    member.size = len(value)
                    output.addfile(member, io.BytesIO(value))
                else:
                    member.linkname = value
                    output.addfile(member)
        return path


class DownloadTests(InstallerFixture):
    def test_owned_temporary_never_removes_preexisting_downloading_file(self):
        destination = self.root / "module.so"
        unowned = self.root / "module.so.downloading"
        unowned.write_bytes(b"belongs to another invocation")
        data = b"pinned module"
        with patch.object(installer.urllib.request, "urlopen", return_value=io.BytesIO(data)):
            installer.download("https://example.invalid/module", destination, hashlib.sha256(data).hexdigest())
        self.assertEqual(destination.read_bytes(), data)
        self.assertEqual(unowned.read_bytes(), b"belongs to another invocation")
        self.assertEqual(sorted(path.name for path in self.root.iterdir()),
                         ["module.so", "module.so.downloading"])

    def test_download_failure_cleans_only_owned_bytes(self):
        destination = self.root / "module.so"
        sentinel = self.root / "module.so.downloading"
        sentinel.write_bytes(b"keep")
        for response, maximum, expected in [(b"wrong", 100, "0" * 64), (b"four", 3, "0" * 64)]:
            with self.subTest(maximum=maximum), \
                 patch.object(installer.urllib.request, "urlopen", return_value=io.BytesIO(response)), \
                 patch.object(installer, "MAX_DOWNLOAD_BYTES", maximum):
                with self.assertRaises(ValueError):
                    installer.download("https://example.invalid/module", destination, expected)
                self.assertFalse(destination.exists())
                self.assertEqual(list(self.root.iterdir()), [sentinel])

    def test_network_failure_closes_and_removes_owned_temporary(self):
        with patch.object(installer.urllib.request, "urlopen", side_effect=OSError("offline")):
            with self.assertRaisesRegex(OSError, "offline"):
                installer.download("https://example.invalid/module", self.root / "module.so", "0" * 64)
        self.assertEqual(list(self.root.iterdir()), [])

    def test_concurrent_publisher_is_never_overwritten(self):
        destination = self.root / "module.so"
        data = b"pinned module"
        for existing in (data, b"unrelated winner"):
            with self.subTest(existing=existing):
                destination.unlink(missing_ok=True)

                def publish_race(_source, target):
                    target.write_bytes(existing)
                    raise FileExistsError("other invocation won")

                with patch.object(installer.urllib.request, "urlopen", return_value=io.BytesIO(data)), \
                     patch.object(installer.os, "link", side_effect=publish_race):
                    if existing == data:
                        installer.download("https://example.invalid/module", destination, hashlib.sha256(data).hexdigest())
                    else:
                        with self.assertRaisesRegex(ValueError, "different digest"):
                            installer.download("https://example.invalid/module", destination, hashlib.sha256(data).hexdigest())
                self.assertEqual(destination.read_bytes(), existing)
                self.assertEqual(list(self.root.iterdir()), [destination])

    def test_existing_mismatch_oversize_or_symlink_is_not_replaced(self):
        destination = self.root / "module.so"
        destination.write_bytes(b"unrelated")
        with patch.object(installer.urllib.request, "urlopen") as network:
            with self.assertRaises(ValueError):
                installer.download("https://example.invalid/module", destination, "0" * 64)
            self.assertEqual(destination.read_bytes(), b"unrelated")
            destination.unlink()
            destination.symlink_to(self.root / "missing")
            with self.assertRaisesRegex(ValueError, "regular file"):
                installer.download("https://example.invalid/module", destination, "0" * 64)
            self.assertTrue(destination.is_symlink())
            network.assert_not_called()


class ExtractionTests(InstallerFixture):
    def test_expected_relative_symlinks_are_allowed(self):
        members = [(installer.REDIS_ROOT, tarfile.DIRTYPE, ""),
                   (installer.REDIS_ROOT + "/src/server.c", tarfile.REGTYPE, b"fixture")]
        members.extend((name, tarfile.SYMTYPE, target) for name, target in installer.EXPECTED_SYMLINKS.items())
        source = installer.extract_redis(self.archive(members), self.root)
        self.assertEqual((source / "src/server.c").read_bytes(), b"fixture")
        for name, target in installer.EXPECTED_SYMLINKS.items():
            self.assertEqual(str((self.root / name).readlink()), target)

    def test_unexpected_paths_links_and_types_roll_back_only_owned_tree(self):
        sentinel = self.root / "unrelated.txt"
        sentinel.write_text("keep")
        expected_link = next(iter(installer.EXPECTED_SYMLINKS))
        bad = [
            ("../escape", tarfile.REGTYPE, b"no"),
            ("/absolute", tarfile.REGTYPE, b"no"),
            ("other-root/file", tarfile.REGTYPE, b"no"),
            (installer.REDIS_ROOT + "/device", tarfile.CHRTYPE, ""),
            (installer.REDIS_ROOT + "/hardlink", tarfile.LNKTYPE, "outside"),
            (expected_link, tarfile.SYMTYPE, "../../../../../../outside"),
            (expected_link, tarfile.SYMTYPE, "/outside"),
            (installer.REDIS_ROOT + "/unexpected-link", tarfile.SYMTYPE, "src"),
        ]
        for member in bad:
            with self.subTest(member=member):
                archive = self.archive([(installer.REDIS_ROOT + "/good", tarfile.REGTYPE, b"good"), member])
                with self.assertRaises(ValueError):
                    installer.extract_redis(archive, self.root)
                self.assertFalse((self.root / installer.REDIS_ROOT).exists())
                self.assertEqual(sentinel.read_text(), "keep")

    def test_duplicate_members_or_writes_through_allowed_links_are_rejected(self):
        name, target = next(iter(installer.EXPECTED_SYMLINKS.items()))
        for members in [
            [(installer.REDIS_ROOT + "/same", tarfile.REGTYPE, b"one"),
             (installer.REDIS_ROOT + "/same", tarfile.REGTYPE, b"two")],
            [(name, tarfile.SYMTYPE, target), (name + "/child", tarfile.REGTYPE, b"alias")],
        ]:
            with self.subTest(members=members), self.assertRaises(ValueError):
                installer.extract_redis(self.archive(members), self.root)
            self.assertFalse((self.root / installer.REDIS_ROOT).exists())

    def test_member_count_and_declared_bytes_are_bounded_before_extraction(self):
        for limit, value, members in [
            ("MAX_ARCHIVE_MEMBERS", 1, [(installer.REDIS_ROOT + "/one", tarfile.REGTYPE, b"a"),
                                       (installer.REDIS_ROOT + "/two", tarfile.REGTYPE, b"b")]),
            ("MAX_EXTRACT_BYTES", 1, [(installer.REDIS_ROOT + "/one", tarfile.REGTYPE, b"ab")]),
        ]:
            with self.subTest(limit=limit), patch.object(installer, limit, value):
                with self.assertRaisesRegex(ValueError, "bound"):
                    installer.extract_redis(self.archive(members), self.root)
                self.assertFalse((self.root / installer.REDIS_ROOT).exists())

    def test_decompressed_extended_headers_are_bounded_before_parsing(self):
        archive = self.root / "extended.tar.gz"
        with tarfile.open(archive, "w:gz", format=tarfile.PAX_FORMAT) as output:
            member = tarfile.TarInfo(installer.REDIS_ROOT + "/file")
            member.pax_headers = {"comment": "x" * 8192}
            output.addfile(member, io.BytesIO())
        with patch.object(installer, "MAX_TAR_BYTES", 2048):
            with self.assertRaisesRegex(ValueError, "decompressed byte bound"):
                installer.extract_redis(archive, self.root)
        self.assertFalse((self.root / installer.REDIS_ROOT).exists())

    def test_existing_source_directory_or_dangling_link_is_never_removed(self):
        source = self.root / installer.REDIS_ROOT
        source.mkdir()
        sentinel = source / "keep"
        sentinel.write_text("unrelated")
        with self.assertRaisesRegex(ValueError, "already exists"):
            installer.extract_redis(self.root / "not-opened", self.root)
        self.assertEqual(sentinel.read_text(), "unrelated")
        sentinel.unlink()
        source.rmdir()
        source.symlink_to(self.root / "missing")
        with self.assertRaisesRegex(ValueError, "already exists"):
            installer.extract_redis(self.root / "not-opened", self.root)
        self.assertTrue(source.is_symlink())


class RepeatedInstallTests(InstallerFixture):
    def fixture_receipt(self):
        data = b"pinned fixture"
        digest = hashlib.sha256(data).hexdigest()
        for path in (self.root / f"{installer.REDIS_ROOT}.tar.gz", self.root / "falkordb.so",
                     self.root / installer.REDIS_ROOT / "src/redis-server",
                     self.root / installer.REDIS_ROOT / "src/redis-cli"):
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        receipt = {"schema_version": 1, "redis_version": installer.REDIS_VERSION,
                   "falkordb_version": installer.FALKOR_VERSION, "redis_url": installer.REDIS_URL,
                   "redis_archive_sha256": digest, "module_url": installer.FALKOR_URL,
                   "module_sha256": digest,
                   "redis_server": str(self.root / installer.REDIS_ROOT / "src/redis-server"),
                   "redis_cli": str(self.root / installer.REDIS_ROOT / "src/redis-cli"),
                   "falkor_module": str(self.root / "falkordb.so"),
                   "redis_server_sha256": digest, "redis_cli_sha256": digest}
        (self.root / "installation.json").write_text(json.dumps(receipt))
        return receipt, digest

    def test_successful_repeat_validates_all_bytes_without_network_or_build(self):
        receipt, digest = self.fixture_receipt()
        executables = [Path(receipt[key]) for key in ("redis_server", "redis_cli", "falkor_module")]
        for path in executables:
            path.chmod(0o600)
        with patch.object(installer, "REDIS_SHA256", digest), \
             patch.object(installer, "FALKOR_SHA256", digest), \
             patch.object(installer.urllib.request, "urlopen") as network, \
             patch.object(installer.subprocess, "run") as build:
            self.assertEqual(installer.install(self.root, 1), receipt)
            network.assert_not_called()
            build.assert_not_called()
        for path in executables:
            self.assertEqual(path.stat().st_mode & 0o777, 0o755)
            self.assertEqual(installer.sha256(path), digest)

    def test_modified_binary_or_receipt_is_rejected_without_rebuilding(self):
        receipt, digest = self.fixture_receipt()
        Path(receipt["redis_server"]).write_bytes(b"modified")
        module = Path(receipt["falkor_module"])
        module.chmod(0o600)
        with patch.object(installer, "REDIS_SHA256", digest), \
             patch.object(installer, "FALKOR_SHA256", digest), \
             patch.object(installer.subprocess, "run") as build:
            with self.assertRaisesRegex(ValueError, "binary differs"):
                installer.install(self.root, 1)
            self.assertEqual(module.stat().st_mode & 0o777, 0o600)
            receipt["redis_server"] = str(self.root.parent / "unrelated")
            (self.root / "installation.json").write_text(json.dumps(receipt))
            with self.assertRaisesRegex(ValueError, "pins or directory"):
                installer.install(self.root, 1)
            build.assert_not_called()

    def test_partial_install_is_explicitly_rejected_and_left_untouched(self):
        source = self.root / installer.REDIS_ROOT
        source.mkdir()
        (source / "keep").write_text("partial source")
        with patch.object(installer.urllib.request, "urlopen") as network:
            with self.assertRaisesRegex(ValueError, "fresh directory"):
                installer.install(self.root, 1)
            network.assert_not_called()
        self.assertEqual((source / "keep").read_text(), "partial source")


class BuildOwnershipTests(unittest.TestCase):
    def test_timeout_and_cancellation_kill_the_group_and_reap_make(self):
        for failure in (subprocess.TimeoutExpired(["make"], 1800), KeyboardInterrupt()):
            with self.subTest(failure=type(failure).__name__):
                process = Mock(pid=12345)
                process.wait.side_effect = [failure, -9]
                with patch.object(installer.subprocess, "Popen", return_value=process) as spawn, \
                     patch.object(installer.os, "killpg") as kill:
                    with self.assertRaises(type(failure)):
                        installer.compile_redis(["make", "fixture"])
                spawn.assert_called_once_with(["make", "fixture"], start_new_session=True)
                kill.assert_called_once_with(12345, signal.SIGKILL)
                self.assertEqual(process.wait.call_args_list, [call(timeout=1800), call(timeout=10)])

    def test_exited_group_is_reaped_and_cleanup_failure_keeps_primary_cause(self):
        failure = subprocess.TimeoutExpired(["make"], 1800)
        for cleanup in (ProcessLookupError(), PermissionError("cannot kill group")):
            with self.subTest(cleanup=type(cleanup).__name__):
                process = Mock(pid=12345)
                process.wait.side_effect = [failure, -9]
                with patch.object(installer.subprocess, "Popen", return_value=process), \
                     patch.object(installer.os, "killpg", side_effect=cleanup):
                    if isinstance(cleanup, ProcessLookupError):
                        with self.assertRaises(subprocess.TimeoutExpired):
                            installer.compile_redis(["make"])
                        self.assertEqual(process.wait.call_count, 2)
                    else:
                        with self.assertRaisesRegex(RuntimeError, "build cleanup failed") as caught:
                            installer.compile_redis(["make"])
                        self.assertIs(caught.exception.__cause__, failure)

    def test_success_and_nonzero_exit_are_reaped_without_group_kill(self):
        for code in (0, 2):
            with self.subTest(code=code):
                process = Mock(pid=12345)
                process.wait.return_value = code
                with patch.object(installer.subprocess, "Popen", return_value=process), \
                     patch.object(installer.os, "killpg") as kill:
                    if code == 0:
                        installer.compile_redis(["make"])
                    else:
                        with self.assertRaises(subprocess.CalledProcessError) as caught:
                            installer.compile_redis(["make"])
                        self.assertEqual(caught.exception.returncode, code)
                    kill.assert_not_called()
                process.wait.assert_called_once_with(timeout=1800)


if __name__ == "__main__":
    unittest.main()
