#!/usr/bin/env python3
import argparse, json, os, re
from collections import defaultdict, Counter

# Heuristics for categories (expand as needed)
CATS = [
    ("unused_imports", re.compile(r"warning: unused import")),
    ("unused_variables", re.compile(r"warning: unused variable")),
    ("deprecated", re.compile(r"warning: use of deprecated")),
    ("unexpected_cfgs", re.compile(r"warning: unexpected `cfg` condition")),
    ("static_mut_refs", re.compile(r"warning: .*mutable static")),
    ("lifetime", re.compile(r"warning: lifetime")),
    ("unreachable_code", re.compile(r"warning: unreachable code")),
    ("style", re.compile(r"warning: (non_snake_case|non_upper_case_globals|clippy)")),
    ("other", re.compile(r"^warning: ")),
]

ERR_RE = re.compile(r"^error(\[E[0-9]+\])?:", re.MULTILINE)
WARN_RE = re.compile(r"^warning:", re.MULTILINE)

# rustc line location pattern
LOC_RE = re.compile(r'--> (.+?):(\d+):(\d+)')

def classify(line):
    for name, rx in CATS[:-1]:
        if rx.search(line):
            return name
    if CATS[-1][1].search(line):
        return "other"
    return None

def scan_file(path):
    data = open(path, "r", errors="ignore").read()
    # Strip ANSI color codes
    ansi_escape = re.compile(r'\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])')
    data = ansi_escape.sub('', data)
    # Split by lines to find locations + messages
    lines = data.splitlines()

    file_warns = []
    file_errs = []
    # Capture contiguous blocks starting with warning/error
    buf = []
    current = None
    for ln in lines:
        if ln.startswith("warning:"):
            if buf and current:
                if current == "warning":
                    file_warns.append("\n".join(buf))
                elif current == "error":
                    file_errs.append("\n".join(buf))
            buf = [ln]
            current = "warning"
        elif ln.startswith("error"):
            if buf and current:
                if current == "warning":
                    file_warns.append("\n".join(buf))
                elif current == "error":
                    file_errs.append("\n".join(buf))
            buf = [ln]
            current = "error"
        else:
            if current:
                buf.append(ln)
    if buf and current:
        if current == "warning":
            file_warns.append("\n".join(buf))
        else:
            file_errs.append("\n".join(buf))

    # Build per-file histogram
    by_file = defaultdict(Counter)
    by_cat = Counter()

    for w in file_warns:
        cat = classify(w) or "other"
        by_cat[cat] += 1
        # find file:line
        m = LOC_RE.search(w)
        if m:
            src = m.group(1)
        else:
            src = "<unknown>"
        by_file[src][cat] += 1

    return {
        "warnings": len(file_warns),
        "errors": len(file_errs),
        "by_category": dict(by_cat),
        "by_file": {k: dict(v) for k,v in by_file.items()},
    }

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--input", required=True, help="raw log dir")
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-md", required=True)
    ap.add_argument("--tag", default="")
    args = ap.parse_args()

    # Map build names from filenames
    # e.g., build-dev.log, clippy-dev.log, fmt.log
    entries = {}
    for fn in sorted(os.listdir(args.input)):
        if not fn.endswith(".log"):
            continue
        path = os.path.join(args.input, fn)
        if fn.startswith("build-"):
            name = fn[len("build-"):-len(".log")]
            entries[f"build:{name}"] = scan_file(path)
        elif fn.startswith("clippy-"):
            name = fn[len("clippy-"):-len(".log")]
            entries[f"clippy:{name}"] = scan_file(path)
        elif fn == "fmt.log":
            # fmt doesn't emit warnings in same style; keep presence
            entries["fmt"] = {"warnings":0,"errors":0,"by_category":{},"by_file":{}}

    # Totals
    totals = {"warnings":0,"errors":0}
    for k,v in entries.items():
        totals["warnings"] += v.get("warnings",0)
        totals["errors"] += v.get("errors",0)

    summary = {
        "tag": args.tag,
        "totals": totals,
        "configs": entries,
        "schema": {
            "version": 1,
            "categories": [c[0] for c in CATS],
        }
    }

    with open(args.output_json, "w") as f:
        json.dump(summary, f, indent=2, sort_keys=True)

    # Markdown roll-up
    def sect(title): return f"\n## {title}\n"
    md = [f"# SIS Kernel — Baseline Lint Scan ({args.tag})",
          "",
          f"- Total warnings: **{totals['warnings']}**",
          f"- Total errors: **{totals['errors']}**",
          "",
          "Configurations scanned:",
          *(f"- `{k}`" for k in sorted(entries.keys()))]

    for k in sorted(entries.keys()):
        v = entries[k]
        md.append(sect(k))
        md.append(f"- Warnings: **{v['warnings']}**  |  Errors: **{v['errors']}**")
        if v["by_category"]:
            md.append("**By category:**")
            for cat, cnt in sorted(v["by_category"].items(), key=lambda x:-x[1]):
                md.append(f"- {cat}: {cnt}")
        # Top files
        if v["by_file"]:
            md.append("\n**Top files by warnings:**")
            # flatten totals per file
            totals_by_file = []
            for src, cats in v["by_file"].items():
                totals_by_file.append( (src, sum(cats.values())) )
            for src, cnt in sorted(totals_by_file, key=lambda x:-x[1])[:10]:
                md.append(f"- {src}: {cnt}")

    with open(args.output_md, "w") as f:
        f.write("\n".join(md))

if __name__ == "__main__":
    main()