#!/usr/bin/env python3
"""Generate dsp-core's singing_materials/data.rs from phonon-peak JSON exports.

Usage:
    python3 gen_data_rs.py                  # every *.json next to this script
    python3 gen_data_rs.py a.json b.json    # just these files, in this order
    python3 gen_data_rs.py -o other/path.rs # write somewhere else

Each JSON file becomes one `Material`. For every temperature listed in
`temps`, the per-species peaks under `by_temp[<temp>].peaks` become one
`PeaksByTemp`, followed by the whole-material peaks from
`by_temp[<temp>].total.peaks.total` as a species named "total". Chart data
is ignored.

Data from: https://github.com/lucydot/SingingMaterialsSite/tree/main/site/data/materials
"""

from __future__ import annotations

import argparse
import json
import math
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
DEFAULT_OUTPUT = (
    SCRIPT_DIR.parent / "dsp-core" / "src" / "engines" / "singing_materials" / "data.rs"
)
RUST_EDITION = "2024"
U16_MAX = 0xFFFF


class DataError(Exception):
    pass


# ---------------------------------------------------------------- literals


def as_float(x: object, where: str) -> float:
    """Validate a JSON number and return it as a finite float."""
    if isinstance(x, bool) or not isinstance(x, (int, float)):
        raise DataError(f"{where}: expected a number, got {x!r}")
    x = float(x)
    if not math.isfinite(x):
        raise DataError(f"{where}: non-finite value {x!r}")
    return x


def rust_f32(x: object, where: str) -> str:
    """Format a number as a Rust float literal (always has '.' or an exponent)."""
    s = repr(as_float(x, where))  # shortest round-tripping form, e.g. "1.0", "0.7386", "1e-05"
    if "." not in s and "e" not in s:
        s += ".0"
    return s


_RUST_ESCAPES = {'"': '\\"', "\\": "\\\\", "\n": "\\n", "\r": "\\r", "\t": "\\t", "\0": "\\0"}


def rust_str(s: object, where: str) -> str:
    """Format a string as a Rust string literal."""
    if not isinstance(s, str):
        raise DataError(f"{where}: expected a string, got {s!r}")
    out = []
    for ch in s:
        if ch in _RUST_ESCAPES:
            out.append(_RUST_ESCAPES[ch])
        elif ord(ch) < 0x20 or ord(ch) == 0x7F:
            out.append(f"\\u{{{ord(ch):x}}}")
        else:
            out.append(ch)
    return '"' + "".join(out) + '"'


def parse_temp(label: str, where: str) -> int:
    """'athermal' -> 0, '300K' -> 300."""
    if label.strip().lower() == "athermal":
        return 0
    m = re.fullmatch(r"\s*(\d+)\s*K\s*", label)
    if not m:
        raise DataError(f"{where}: unrecognized temperature label {label!r}")
    value = int(m.group(1))
    if not 0 < value <= U16_MAX:
        raise DataError(f"{where}: temperature {value} K does not fit in a nonzero u16")
    return value


# ---------------------------------------------------------------- emitters


def emit_species(species: str, data: dict, where: str) -> str:
    try:
        freqs, widths, densities = data["freq"], data["width"], data["D"]
    except KeyError as e:
        raise DataError(f"{where}: missing key {e}") from None
    if not (len(freqs) == len(widths) == len(densities)):
        raise DataError(
            f"{where}: freq/width/D lengths differ "
            f"({len(freqs)}/{len(widths)}/{len(densities)})"
        )

    freqs = [as_float(f, f"{where}.freq[{i}]") for i, f in enumerate(freqs)]
    widths = [as_float(w, f"{where}.width[{i}]") for i, w in enumerate(widths)]
    densities = [as_float(d, f"{where}.D[{i}]") for i, d in enumerate(densities)]

    if freqs:
        # Frequencies become ratios relative to the peak with the largest
        # density (first one wins on a tie), so that peak gets exactly 1.0.
        base_idx = max(range(len(densities)), key=densities.__getitem__)
        base_freq = freqs[base_idx]
        if base_freq <= 0.0:
            raise DataError(f"{where}: base frequency freq[{base_idx}] = {base_freq} is not positive")
        freq_ratios = [1.0 if i == base_idx else f / base_freq for i, f in enumerate(freqs)]
        # Widths are expressed relative to the same base frequency.
        width_ratios = [w / base_freq for w in widths]

        # Densities are rescaled to sum to 1.0, keeping their proportions.
        if any(d < 0.0 for d in densities):
            raise DataError(f"{where}: negative density in D")
        density_sum = math.fsum(densities)
        if density_sum <= 0.0:
            raise DataError(f"{where}: densities sum to {density_sum}, cannot normalize")
        densities = [d / density_sum for d in densities]
    else:
        freq_ratios = []
        width_ratios = []

    peaks = [
        "Peak {"
        f" freq_ratio: {rust_f32(r, f'{where}.freq[{i}] (ratio)')},"
        f" width_ratio: {rust_f32(w, f'{where}.width[{i}] (ratio)')},"
        f" density: {rust_f32(d, f'{where}.D[{i}] (normalized)')} }}"
        for i, (r, w, d) in enumerate(zip(freq_ratios, width_ratios, densities))
    ]
    return (
        f"PeaksBySpecies {{ species: {rust_str(species, where)}, "
        f"peaks: &[{', '.join(peaks)}] }}"
    )


def emit_temp(label: str, entry: dict, species_order: list[str], where: str) -> str:
    temp = parse_temp(label, where)
    peaks = entry.get("peaks")
    if not isinstance(peaks, dict):
        raise DataError(f"{where}: missing 'peaks' object")

    # Follow the file's `labels` order, then any extra species it didn't list.
    order = [s for s in species_order if s in peaks]
    order += [s for s in peaks if s not in order]

    species = [emit_species(s, peaks[s], f"{where}.peaks.{s}") for s in order]

    # Whole-material ("total") projection, appended after the per-species entries.
    total_peaks = (entry.get("total") or {}).get("peaks")
    if not isinstance(total_peaks, dict) or "total" not in total_peaks:
        raise DataError(f"{where}: missing 'total.peaks.total'")
    species.append(emit_species("total", total_peaks["total"], f"{where}.total.peaks.total"))

    return f"PeaksByTemp {{ temp: {temp}, formatted_temp: {rust_str(label, where)}, peaks_by_species: &[{', '.join(species)}] }}"


def emit_material(path: Path) -> str:
    where = path.name
    try:
        doc = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        raise DataError(f"{where}: invalid JSON: {e}") from None

    by_temp = doc.get("by_temp")
    if not isinstance(by_temp, dict) or not by_temp:
        raise DataError(f"{where}: missing or empty 'by_temp'")

    temp_order = doc.get("temps") or list(by_temp)
    missing = [t for t in temp_order if t not in by_temp]
    if missing:
        raise DataError(f"{where}: 'temps' lists {missing} but 'by_temp' has no entry")

    seen = set()
    for t in temp_order:
        k = parse_temp(t, f"{where}.temps")
        if k in seen:
            raise DataError(f"{where}: duplicate temperature {t!r}")
        seen.add(k)

    species_order = doc.get("labels") or []
    temps = [
        emit_temp(t, by_temp[t], species_order, f"{where}.by_temp.{t}") for t in temp_order
    ]

    mp_id = rust_str(doc.get("mp_id"), f"{where}.mp_id")
    name = rust_str(doc.get("name"), f"{where}.name")

    return (
        f"Material {{ mp_id: {mp_id}, name: {name}, "
        f"peaks_by_temp: &[{', '.join(temps)}] }}"
    )


def emit_file(paths: list[Path]) -> str:
    sources = ", ".join(p.name for p in paths)
    materials = ",\n".join(emit_material(p) for p in paths)
    return (
        f"// This file is @generated by singing-materials-data/{Path(__file__).name}\n"
        "// In general, it should not be edited by hand. Rerun that script to regenerate after a change in material data.\n\n"
        "use crate::engines::singing_materials::types::{Material, Peak, PeaksBySpecies, PeaksByTemp};\n\n"
        f"pub const MATERIALS: &[Material] = &[\n{materials},\n];\n"
    )


# ---------------------------------------------------------------- main


def rustfmt(source: str) -> str:
    exe = shutil.which("rustfmt")
    if exe is None:
        print("warning: rustfmt not found; writing unformatted output", file=sys.stderr)
        return source
    result = subprocess.run(
        [exe, "--edition", RUST_EDITION, "--emit", "stdout"],
        input=source,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise DataError(f"rustfmt rejected the generated code:\n{result.stderr}")
    return result.stdout


def write_atomic(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp = tempfile.mkstemp(dir=path.parent, prefix=f".{path.name}.", suffix=".tmp")
    try:
        with os.fdopen(fd, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
        os.replace(tmp, path)
    except BaseException:
        os.unlink(tmp)
        raise


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("inputs", nargs="*", type=Path, help="JSON files (default: *.json next to this script)")
    parser.add_argument("-o", "--output", type=Path, default=DEFAULT_OUTPUT, help=f"default: {DEFAULT_OUTPUT}")
    parser.add_argument("--no-fmt", action="store_true", help="skip running rustfmt")
    args = parser.parse_args()

    inputs = args.inputs or sorted(SCRIPT_DIR.glob("*.json"))
    if not inputs:
        print(f"error: no JSON files found in {SCRIPT_DIR}", file=sys.stderr)
        return 1

    try:
        source = emit_file(inputs)
        if not args.no_fmt:
            source = rustfmt(source)
    except (DataError, OSError) as e:
        print(f"error: {e}", file=sys.stderr)
        return 1

    # Nothing is written unless generation (and formatting) succeeded.
    write_atomic(args.output, source)
    print(f"wrote {len(inputs)} material(s) to {args.output}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
