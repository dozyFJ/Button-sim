#!/usr/bin/env python3
"""Fetch locally signed release blobs and validate them before CI publication."""
import base64, hashlib, json, os, pathlib, re, subprocess, urllib.request, zipfile
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
def artifact_release(config):
    artifact=config["artifact_id"]
    if not isinstance(artifact,int) or artifact<=0:raise ValueError("Invalid artifact")
    target=pathlib.Path("build/source-artifact.zip")
    with target.open("wb") as out:
        subprocess.run(["gh","api",f"repos/{REPO}/actions/artifacts/{artifact}/zip"],stdout=out,check=True)
    folder=pathlib.Path("build/game-ota");folder.mkdir(parents=True,exist_ok=True)
    manifest_bytes=base64.b64decode(config["manifest"],validate=True)
    manifest=json.loads(manifest_bytes)
    with zipfile.ZipFile(target) as source:
        original=source.read("build/button-sim-bootstrap-unsigned.apk")
        if hashlib.sha256(original).hexdigest()!=config["base_apk_sha256"]:raise ValueError("APK base mismatch")
        patch=config["apk_patch"];prefix=patch["prefix"];suffix=patch["suffix"]
        if not all(isinstance(n,int) and n>=0 for n in (prefix,suffix)) or prefix+suffix>len(original):raise ValueError("Invalid patch bounds")
        signed=original[:prefix]+base64.b64decode(patch["data"],validate=True)+(original[len(original)-suffix:] if suffix else b"")
        if len(signed)>32*1024*1024 or hashlib.sha256(signed).hexdigest()!=config["apk_sha256"]:raise ValueError("Signed APK mismatch")
        pathlib.Path("build/button-sim-release.apk").write_bytes(signed)
        archive=folder/"game-runtime.zip"
        with zipfile.ZipFile(archive,"w",zipfile.ZIP_STORED) as output:
            for entry in sorted(manifest["files"],key=lambda f:f["name"]):
                name=entry["name"]
                if not re.fullmatch(r"lib[a-zA-Z0-9_]+\.so",name):raise ValueError("Invalid native library")
                content=source.read("android-shell/app/src/main/jniLibs/arm64-v8a/"+name)
                if len(content)!=entry["size"] or hashlib.sha256(content).hexdigest()!=entry["sha256"]:raise ValueError("Native hash mismatch")
                info=zipfile.ZipInfo(name,(2026,1,1,0,0,0));info.compress_type=zipfile.ZIP_STORED;info.external_attr=0o444<<16
                output.writestr(info,content)
        if hashlib.sha256(archive.read_bytes()).hexdigest()!=manifest["sha256"]:raise ValueError("Archive hash mismatch")
    (folder/"game-ota.json").write_bytes(manifest_bytes)
    (folder/"game-ota.sig").write_bytes(base64.b64decode(config["signature"],validate=True))

def main():
    reference=os.environ.get("RELEASE_CONFIG_BLOB","")
    if not reference:reference=json.loads(pathlib.Path("release-staging.json").read_text())["config_blob"]
    config=json.loads(blob(reference))
    if not re.fullmatch(r"[0-9a-f]{40}",config["revision"]):raise ValueError("Invalid revision")
    cert=base64.b64decode(config["certificate"])
    if hashlib.sha256(cert).hexdigest()!=FINGERPRINT:raise ValueError("Unexpected signer")
    pathlib.Path("build").mkdir(exist_ok=True)
    pathlib.Path("build/release-cert.der").write_bytes(cert)
    if "artifact_id" in config:
        artifact_release(config)
    else:
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

