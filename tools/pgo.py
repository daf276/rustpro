"""Builds with profile-guided optimization on top of the sim profile: the instrumented throughput bench plays Elfnote
mirror and Normal Monster duels and the stack bench clones, observes, determinizes and steps batches, llvm-profdata merges
what they recorded, and cargo builds the given targets with it into target/pgo-use (the default sim build in target/
stays as it is). Needs `rustup component add llvm-tools`. RUSTFLAGS in the environment are kept.

  python tools/pgo.py [cargo build arguments, default: -p bench --bins]
  target/pgo-use/sim/throughput --elfnote 1 --threads 1 --duels 2000 --seed 3

Arguments starting with maturin build the Python module instead, with the python profile unless they name one, trained
on its own smoke tests (the profile names the crates as built with the module's dependencies, so a bench-trained one
matches nothing); pytest and numpy must be installed and maturin must not be given --out:

  python tools/pgo.py maturin develop -m crates/py/Cargo.toml
"""
import os
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TARGET = ROOT / "target"
RAW = TARGET / "pgo-raw"
PROFILE = TARGET / "pgo.profdata"
EXE = ".exe" if os.name == "nt" else ""


def build(command, target_dir, rustflags):
    rustflags = f"{os.environ.get('RUSTFLAGS', '')} {rustflags}".strip()
    env = dict(os.environ, CARGO_TARGET_DIR=str(target_dir), RUSTFLAGS=rustflags)
    subprocess.run(command, cwd=ROOT, env=env, check=True)
    return rustflags


def llvm_profdata():
    sysroot = subprocess.run(["rustc", "--print", "sysroot"], capture_output=True, text=True, check=True).stdout.strip()
    host = next(line.split()[1] for line in subprocess.run(["rustc", "-vV"], capture_output=True, text=True, check=True).stdout.splitlines() if line.startswith("host:"))
    tool = Path(sysroot) / "lib/rustlib" / host / "bin" / f"llvm-profdata{EXE}"
    if not tool.exists():
        sys.exit("llvm-profdata not found: rustup component add llvm-tools")
    return tool


profdata = llvm_profdata()
shutil.rmtree(RAW, ignore_errors=True)
args = sys.argv[1:] or ["-p", "bench", "--bins"]
generate, use = f"-Cprofile-generate={RAW}", f"-Cprofile-use={PROFILE}"
if args[0] == "maturin":
    options = args[2:] if {"--profile", "--release", "-r"} & set(args) else ["--profile", "python", *args[2:]]
    wheels, module = TARGET / "pgo-generate/wheels", TARGET / "pgo-python"
    shutil.rmtree(wheels, ignore_errors=True)
    build(["maturin", "build", *options], TARGET / "pgo-generate", generate)
    zipfile.ZipFile(next(wheels.glob("*.whl"))).extractall(module)
    tests = dict(os.environ, PYTHONPATH=str(module), YGO_SIM_DUELS="256")
    subprocess.run([sys.executable, "-m", "pytest", "-q", "crates/py/tests"], cwd=ROOT, env=tests, check=True)
    command = ["maturin", args[1], *options]
else:
    cargo = ["cargo", "build", "--profile", "sim"]
    build([*cargo, "-p", "bench", "--bin", "throughput", "--bin", "stack"], TARGET / "pgo-generate", generate)
    bench = TARGET / "pgo-generate/sim"
    for workload in [["--elfnote", "1", "--duels", "400"], ["--duels", "1000"]]:
        subprocess.run([bench / f"throughput{EXE}", "--threads", "1", "--seed", "1000", *workload], check=True)
    subprocess.run([bench / f"stack{EXE}", "--threads", "1", "--seed", "1000", "--duels", "100", "--sampled", "100"], check=True)
    command = [*cargo, *args]
subprocess.run([profdata, "merge", "-o", PROFILE, *RAW.glob("*.profraw")], check=True)
rustflags = build(command, TARGET / "pgo-use", use)
print(f"built into {TARGET / 'pgo-use'}; RUSTFLAGS={rustflags}")
