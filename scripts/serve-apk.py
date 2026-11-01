"""Serve the Alux APK to a phone on the same Wi-Fi.

- "/" redirects straight to the APK
- threaded, so parallel/重复 requests don't stall each other
- supports HTTP Range so a dropped download can resume instead of corrupting
"""
import http.server
import os
import re
import socketserver

ROOT = r"D:\app\Web Browser\alux\release"
APK = "IVEC-SR2-0.1.0-arm64-debug.apk"
PORT = 8765


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt, *args):
        print("%s - %s" % (self.address_string(), fmt % args), flush=True)

    def do_HEAD(self):
        self._serve(head_only=True)

    def do_GET(self):
        if self.path in ("/", "/index.html"):
            self.send_response(302)
            self.send_header("Location", "/" + APK)
            self.send_header("Content-Length", "0")
            self.end_headers()
            return
        if not self.path.startswith("/" + APK):
            self.send_error(404)
            return
        self._serve()

    def _serve(self, head_only=False):
        path = os.path.join(ROOT, APK)
        size = os.path.getsize(path)
        start, end = 0, size - 1

        rng = self.headers.get("Range")
        m = re.match(r"bytes=(\d*)-(\d*)$", rng or "")
        if m:
            if m.group(1):
                start = int(m.group(1))
            if m.group(2):
                end = min(int(m.group(2)), size - 1)
            if start > end:
                self.send_error(416)
                return

        length = end - start + 1
        self.send_response(206 if rng else 200)
        self.send_header("Content-Type", "application/vnd.android.package-archive")
        self.send_header("Content-Disposition", 'attachment; filename="%s"' % APK)
        self.send_header("Content-Length", str(length))
        self.send_header("Accept-Ranges", "bytes")
        if rng:
            self.send_header("Content-Range", "bytes %d-%d/%d" % (start, end, size))
        self.end_headers()

        if head_only:
            return
        with open(path, "rb") as f:
            f.seek(start)
            remaining = length
            while remaining > 0:
                chunk = f.read(min(256 * 1024, remaining))
                if not chunk:
                    break
                try:
                    self.wfile.write(chunk)
                except (ConnectionResetError, BrokenPipeError):
                    print("client aborted at %d/%d bytes" % (length - remaining, length), flush=True)
                    return
                remaining -= len(chunk)
        print("served %s bytes %d-%d" % (APK, start, end), flush=True)


class Server(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True


with Server(("0.0.0.0", PORT), Handler) as httpd:
    print("serving on port %d" % PORT, flush=True)
    httpd.serve_forever()
