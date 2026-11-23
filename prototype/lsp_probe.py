import json, subprocess, sys, os, time, threading, queue
root = sys.argv[1]; path = os.path.join(root, "src/main.rs")
text = open(path).read(); lines = text.split("\n")
p = subprocess.Popen(["rust-analyzer"], cwd=root, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
q = queue.Queue()
def reader():
    while True:
        h = {}
        while True:
            l = p.stdout.readline()
            if not l: return
            l = l.decode().strip()
            if not l: break
            k, v = l.split(":", 1); h[k.lower()] = v.strip()
        q.put(json.loads(p.stdout.read(int(h["content-length"]))))
threading.Thread(target=reader, daemon=True).start()
nid = [0]
def send(m):
    b = json.dumps(m).encode(); p.stdin.write(b"Content-Length: %d\r\n\r\n" % len(b) + b); p.stdin.flush()
def req(method, params, timeout=300):
    nid[0] += 1; i = nid[0]; send({"jsonrpc": "2.0", "id": i, "method": method, "params": params})
    end = time.time() + timeout
    while time.time() < end:
        m = q.get(timeout=end - time.time())
        if m.get("id") == i and "method" not in m: return m.get("result", m.get("error"))
        if "method" in m and "id" in m:  # server request
            send({"jsonrpc": "2.0", "id": m["id"], "result": None})
        if m.get("method") == "experimental/serverStatus": status.append(m["params"])
status = []
uri = "file://" + path
req("initialize", {"processId": None, "rootUri": "file://" + root, "capabilities": {"textDocument": {"hover": {"contentFormat": ["plaintext"]}, "completion": {"completionItem": {}}}, "experimental": {"serverStatusNotification": True}}, "initializationOptions": {"procMacro": {"enable": True}, "cargo": {"buildScripts": {"enable": True}}, "checkOnSave": False}})
send({"jsonrpc": "2.0", "method": "initialized", "params": {}})
send({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {"uri": uri, "languageId": "rust", "version": 1, "text": text}}})
# wait for quiescent
end = time.time() + 600
while time.time() < end:
    try: m = q.get(timeout=5)
    except queue.Empty: continue
    if "method" in m and "id" in m: send({"jsonrpc": "2.0", "id": m["id"], "result": None})
    if m.get("method") == "experimental/serverStatus" and m["params"].get("quiescent"): break
time.sleep(5)
def pos(line1, needle, occ=0, offset=0):
    l = lines[line1 - 1]; c = -1
    for _ in range(occ + 1): c = l.index(needle, c + 1)
    return {"line": line1 - 1, "character": c + offset}
def show(label, r):
    print("##", label); print(json.dumps(r)[:700] if not isinstance(r, dict) or "contents" not in r else r["contents"]["value"][:500]); print()
checks = json.loads(sys.argv[2])
for c in checks:
    kind, label, line, needle = c[:4]; offset = c[4] if len(c) > 4 else 0
    P = {"textDocument": {"uri": uri}, "position": pos(line, needle, 0, offset)}
    if kind == "hover": show(label, req("textDocument/hover", P))
    elif kind == "def":
        r = req("textDocument/definition", P)
        show(label, [(x.get("uri", x.get("targetUri")).split("/")[-1], (x.get("range") or x.get("targetSelectionRange"))["start"]["line"] + 1) for x in (r or [])])
    elif kind == "complete":
        r = req("textDocument/completion", P); items = r["items"] if isinstance(r, dict) else (r or [])
        show(label, [i["label"] + " : " + str(i.get("detail")) for i in items if i["label"].startswith(c[5] if len(c) > 5 else "")][:12])
p.kill()
