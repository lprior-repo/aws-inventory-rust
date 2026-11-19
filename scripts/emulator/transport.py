"""Bounded, loopback-only TLS recorder for real SDK requests to AWS Emulator.

The proxy preserves the signed Host/header/body bytes. It records operation names,
HTTP statuses and response sizes, never Authorization, credentials, or bodies.
Seeding and Rust collection use separate phase labels so seed calls cannot be
mistaken for exercised collector calls. Both TLS hops verify their certificates.
"""

from __future__ import annotations

import hashlib
import http.client
import http.server
import ipaddress
import json
import re
import ssl
import subprocess
import threading
import urllib.parse
from pathlib import Path

MAX_BODY = 32 * 1024 * 1024
MAX_REQUESTS = 100_000
MAX_ACTIVE = 64
SERVICE_NAMES = {
    "elasticloadbalancing": "elbv2",
    "cloudformation": "cloudcontrol",
    "resource-explorer-2": "resource-explorer-2",
}


def require_loopback(endpoint: str) -> urllib.parse.SplitResult:
    parsed = urllib.parse.urlsplit(endpoint)
    if (
        parsed.scheme != "https"
        or not parsed.hostname
        or not ipaddress.ip_address(parsed.hostname).is_loopback
        or not parsed.port
        or parsed.username
        or parsed.password
        or parsed.path not in ("", "/")
        or parsed.query
        or parsed.fragment
    ):
        raise ValueError("emulator endpoint must be HTTPS on a literal loopback IP and port")
    return parsed


class Recorder:
    def __init__(self, botocore_session):
        self.session = botocore_session
        self.entries = []
        self.phase = "startup"
        self.lock = threading.Lock()
        self.models = {}

    def set_phase(self, phase):
        with self.lock:
            self.phase = phase

    def current_phase(self):
        with self.lock:
            return self.phase

    def classify(self, method, target, headers, body):
        authorization = headers.get("Authorization", "")
        # Only the public SigV4 service/region scope is retained; never the key ID.
        matched = re.search(r"Credential=[^/ ,]+/\d{8}/([^/]+)/([^/]+)/aws4_request", authorization)
        region, signing = matched.groups() if matched else ("unknown", "unknown")
        service = SERVICE_NAMES.get(signing, signing)
        request = urllib.parse.urlsplit(target)
        query = urllib.parse.parse_qs(request.query, keep_blank_values=True)
        request_json = None
        if "json" in headers.get("Content-Type", ""):
            try:
                request_json = json.loads(body)
            except (ValueError, UnicodeDecodeError):
                pass
        if headers.get("X-Amz-Target"):
            operation = headers["X-Amz-Target"].rsplit(".", 1)[-1]
            if "CloudApiService" in headers["X-Amz-Target"]:
                service = "cloudcontrol"
        elif headers.get("Content-Type", "").startswith("application/x-www-form-urlencoded"):
            values = urllib.parse.parse_qs(body.decode("utf-8"), keep_blank_values=True)
            operation = values.get("Action", ["unclassified"])[0]
            query.update(values)
        elif "Action" in query:
            operation = query["Action"][0]
        else:
            operation = self._rest_operation(service, method, request.path, query)
        # TypeName is a public schema name. Tokens/ARNs/identifiers stay out of the log.
        typename = request_json.get("TypeName") if isinstance(request_json, dict) else None
        token_keys = {"nexttoken", "next_token", "marker", "continuation-token", "startrecordname", "exclusivestarttablename"}
        keys = set(query) | (set(request_json) if isinstance(request_json, dict) else set())
        has_cursor = any(key.lower() in token_keys for key in keys)
        if service == "route53" and operation == "ListResourceRecordSets":
            has_cursor = has_cursor or "name" in query
        return {
            "phase": self.current_phase(),
            "service": service,
            "operation": operation,
            "region": region,
            "method": method,
            "type_name": typename,
            "pagination_request": has_cursor,
        }

    def _rest_operation(self, service, method, path, query):
        if service == "s3" and method == "GET" and path in ("", "/"):
            return "ListBuckets"
        with self.lock:
            if service not in self.models:
                try:
                    model = self.session.get_service_model(service)
                    routes = []
                    for name in model.operation_names:
                        http = model.operation_model(name).http
                        uri = urllib.parse.urlsplit(http.get("requestUri", "/"))
                        pattern = re.escape(uri.path)
                        pattern = re.sub(r"\\\{[^}]+\\\}", "[^/]+", pattern)
                        required = urllib.parse.parse_qs(uri.query, keep_blank_values=True)
                        routes.append((name, http.get("method", "POST"), re.compile("^" + pattern + "/?$"), required, len(uri.path)))
                    self.models[service] = routes
                except Exception:
                    self.models[service] = []
            routes = self.models[service]
        matches = []
        for name, verb, pattern, required, specificity in routes:
            if verb != method or not pattern.fullmatch(path):
                continue
            if not all(key in query and (values == [""] or query[key] == values) for key, values in required.items()):
                continue
            matches.append((len(required), specificity, name))
        return max(matches)[2] if matches else "unclassified"

    def append(self, entry):
        with self.lock:
            if len(self.entries) >= MAX_REQUESTS:
                raise RuntimeError("emulator request evidence capacity exceeded")
            entry.setdefault("phase", self.phase)
            entry["sequence"] = len(self.entries) + 1
            self.entries.append(entry)

    def snapshot(self):
        with self.lock:
            return list(self.entries)


class _Server(http.server.ThreadingHTTPServer):
    daemon_threads = False
    block_on_close = True
    request_queue_size = MAX_ACTIVE

    def __init__(self, *args, **kwargs):
        self.capacity = threading.BoundedSemaphore(MAX_ACTIVE)
        super().__init__(*args, **kwargs)

    def process_request(self, request, client_address):
        if not self.capacity.acquire(timeout=5):
            request.close()
            return
        try:
            super().process_request(request, client_address)
        except BaseException:
            self.capacity.release()
            raise

    def get_request(self):
        request, address = super().get_request()
        request.settimeout(35)
        return request, address

    def process_request_thread(self, request, client_address):
        try:
            super().process_request_thread(request, client_address)
        finally:
            self.capacity.release()


class RecordingProxy:
    def __init__(self, endpoint, ca_file, work, recorder):
        self.upstream = require_loopback(endpoint)
        self.recorder = recorder
        self.work = Path(work)
        self.work.mkdir(parents=True, exist_ok=True)
        self.cert = self.work / "proxy-ca.pem"
        ca_key = self.work / "proxy-ca-key.pem"
        key = self.work / "proxy-server-key.pem"
        leaf = self.work / "proxy-server.pem"
        request = self.work / "proxy-server.csr"
        extensions = self.work / "proxy-server.ext"
        subprocess.run(
            ["openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1",
             "-subj", "/CN=AWS inventory emulator test CA", "-addext", "basicConstraints=critical,CA:TRUE",
             "-keyout", str(ca_key), "-out", str(self.cert)],
            check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=30,
        )
        subprocess.run(
            ["openssl", "req", "-new", "-newkey", "rsa:2048", "-nodes",
             "-subj", "/CN=AWS inventory emulator recorder", "-keyout", str(key), "-out", str(request)],
            check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=30,
        )
        extensions.write_text("basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\nsubjectAltName=IP:127.0.0.1\n")
        subprocess.run(
            ["openssl", "x509", "-req", "-in", str(request), "-CA", str(self.cert), "-CAkey", str(ca_key),
             "-CAcreateserial", "-out", str(leaf), "-days", "1", "-sha256", "-extfile", str(extensions)],
            check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=30,
        )
        ca_key.chmod(0o600)
        key.chmod(0o600)
        upstream_tls = ssl.create_default_context(cafile=str(ca_file))
        outer = self

        class Handler(http.server.BaseHTTPRequestHandler):
            protocol_version = "HTTP/1.1"

            def setup(self):
                self.accepted_phase = outer.recorder.current_phase()
                super().setup()

            def log_message(self, _format, *_args):
                pass

            def _forward(self):
                self.connection.settimeout(35)
                if self.headers.get("Transfer-Encoding"):
                    self.send_error(501, "test recorder does not accept chunked request bodies")
                    return
                length = int(self.headers.get("Content-Length", "0"))
                if length < 0 or length > MAX_BODY:
                    self.send_error(413)
                    return
                body = self.rfile.read(length)
                entry = outer.recorder.classify(self.command, self.path, self.headers, body)
                entry["phase"] = self.accepted_phase
                connection = http.client.HTTPSConnection(outer.upstream.hostname, outer.upstream.port, context=upstream_tls, timeout=30)
                recorded = False
                try:
                    headers = dict(self.headers)
                    headers.pop("Connection", None)
                    connection.request(self.command, self.path, body=body, headers=headers)
                    response = connection.getresponse()
                    payload = response.read(MAX_BODY + 1)
                    if len(payload) > MAX_BODY:
                        raise RuntimeError("emulator response exceeded recorder byte budget")
                    entry.update({"http_status": response.status, "response_bytes": len(payload), "response_sha256": hashlib.sha256(payload).hexdigest()})
                    outer.recorder.append(entry)
                    recorded = True
                    self.send_response(response.status)
                    for name, value in response.getheaders():
                        if name.lower() not in {"connection", "transfer-encoding", "content-length"}:
                            self.send_header(name, value)
                    self.send_header("Content-Length", str(len(payload)))
                    self.send_header("Connection", "close")
                    self.end_headers()
                    if self.command != "HEAD":
                        self.wfile.write(payload)
                except Exception as error:
                    if not recorded:
                        entry.update({"http_status": 502, "transport_error": type(error).__name__})
                        outer.recorder.append(entry)
                        self.send_error(502, "local emulator transport failure")
                finally:
                    connection.close()
                    self.close_connection = True

            do_GET = _forward
            do_POST = _forward
            do_PUT = _forward
            do_DELETE = _forward
            do_HEAD = _forward
            do_PATCH = _forward

        self.server = _Server(("127.0.0.1", 0), Handler)
        tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        tls.load_cert_chain(str(leaf), str(key))
        self.server.socket = tls.wrap_socket(self.server.socket, server_side=True, do_handshake_on_connect=False)
        self.endpoint = f"https://127.0.0.1:{self.server.server_port}"
        self.thread = threading.Thread(target=self.server.serve_forever, name="emulator-recorder")

    def __enter__(self):
        self.thread.start()
        return self

    def __exit__(self, *_args):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=35)
        if self.thread.is_alive():
            raise RuntimeError("emulator recorder failed to drain")
