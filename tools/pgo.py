"""Builds with profile-guided optimization on top of the sim profile: an instrumented throughput bench plays Elfnote mirror
and Normal Monster duels, llvm-profdata merges what they recorded, and cargo builds the given targets with it into
target/pgo-use (the default sim build in target/ stays as it is). Needs `rustup component add llvm-tools`.

  python tools/pgo.py [cargo build arguments, default: -p bench --bins]
  target/pgo-use/sim/throughput --elfnote 1 --threads 1 --duels 2000 --seed 3

For the Python module, pass the printed RUSTFLAGS to its maturin build.
"""
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TARGET = ROOT / "target"
RAW = TARGET / "pgo-raw"
PROFILE = TARGET / "pgo.profdata"
EXE = ".exe" if os.name == "nt" else ""


def cargo_build(target_dir, rustflags, args):
    env = dict(os.environ, CARGO_TARGET_DIR=str(target_dir), RUSTFLAGS=rustflags)
    subprocess.run(["cargo", "build", "--profile", "sim", *args], cwd=ROOT, env=env, check=True)


def llvm_profdata():
    sysroot = subprocess.run(["rustc", "--print", "sysroot"], capture_output=True, text=True, check=True).stdout.strip()
    host = next(line.split()[1] for line in subprocess.run(["rustc", "-vV"], capture_output=True, text=True, check=True).stdout.splitlines() if line.startswith("host:"))
    tool = Path(sysroot) / "lib/rustlib" / host / "bin" / f"llvm-profdata{EXE}"
    if not tool.exists():
        sys.exit("llvm-profdata not found: rustup component add llvm-tools")
    return tool


profdata = llvm_profdata()
shutil.rmtree(RAW, ignore_errors=True)
cargo_build(TARGET / "pgo-generate", f"-Cprofile-generate={RAW}", ["-p", "bench", "--bin", "throughput"])
bench = TARGET / "pgo-generate/sim" / f"throughput{EXE}"
for workload in [["--elfnote", "1", "--duels", "400"], ["--duels", "1000"]]:
    subprocess.run([bench, "--threads", "1", "--seed", "1000", *workload], check=True)
subprocess.run([profdata, "merge", "-o", PROFILE, *RAW.glob("*.profraw")], check=True)
rustflags = f"-Cprofile-use={PROFILE}"
cargo_build(TARGET / "pgo-use", rustflags, sys.argv[1:] or ["-p", "bench", "--bins"])
print(f"built into {TARGET / 'pgo-use/sim'}; RUSTFLAGS={rustflags}")
