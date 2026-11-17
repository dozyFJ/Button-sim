#!/usr/bin/env python3
"""Fetch locally signed release blobs and validate them before CI publication."""
import base64, hashlib, json, os, pathlib, re, subprocess, urllib.request
REPO="dozyFJ/Button-sim"
FINGERPRINT="f57b73cec072c6ca688126dfb34c8eb42f2780f357d8146b01fdce7feb2b0477"
def blob(sha):
    if not re.fullmatch(r"[0-9a-f]{40}",sha): raise ValueError("Invalid blob")
    request=urllib.request.Request("https://api.github.com/repos/"+REPO+"/git/blobs/"+sha,
        headers={"Authorization":"Bearer "+os.environ["GH_TOKEN"],"Accept":"application/vnd.github+json","User-Agent":"ButtonSim-release"})
    with urllib.request.urlopen(request,timeout=60) as response:
        data=json.load(response)
    if data.get("encoding")!="base64":raise ValueError("Invalid encoding")
    return base64.b64decode(data["content"])
def main():
    config=json.loads(blob(os.environ["RELEASE_CONFIG_BLOB"]))
    if not re.fullmatch(r"[0-9a-f]{40}",config["revision"]):raise ValueError("Invalid revision")
    cert=base64.b64decode(config["certificate"])
    if hashlib.sha256(cert).hexdigest()!=FINGERPRINT:raise ValueError("Unexpected signer")
    pathlib.Path("build").mkdir(exist_ok=True)
    pathlib.Path("build/release-cert.der").write_bytes(cert)
    allowed={"build/button-sim-release.apk","build/game-ota/game-runtime.zip","build/game-ota/game-ota.json","build/game-ota/game-ota.sig"}
    if set(config["files"])!=allowed:raise ValueError("Unexpected release files")
    for name,meta in config["files"].items():
        content=blob(meta["blob"])
        if len(content)>32*1024*1024 or hashlib.sha256(content).hexdigest()!=meta["sha256"]:raise ValueError("File hash mismatch")
        path=pathlib.Path(name);path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(content)
    manifest=json.loads(pathlib.Path("build/game-ota/game-ota.json").read_text())
    if manifest["build"]!=config["revision"]:raise ValueError("Core revision mismatch")
    shell=subprocess.check_output(["python3","tools/shell_hash.py","game"],text=True).strip()
    if manifest["shell_hash"]!=shell:raise ValueError("Shell hash mismatch")
    with open(os.environ["GITHUB_ENV"],"a") as out:out.write("PUBLISHED_CORE_REVISION="+config["revision"]+"\n")
    print("Release blobs and pinned signer verified:",config["revision"])
if __name__=="__main__":main()

