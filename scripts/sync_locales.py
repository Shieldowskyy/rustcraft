#!/usr/bin/env python3
"""
RustCraft Localization Synchronizer & Translation Helper.

Usage:
    python3 scripts/sync_locales.py                # Synchronizes all existing locale files against en.json
    python3 scripts/sync_locales.py --new de       # Creates a new locale file locales/de.json from template
    python3 scripts/sync_locales.py --check        # Checks for missing keys without modifying files
"""

import sys
import json
import argparse
from pathlib import Path

LOCALES_DIR = Path(__file__).parent.parent / "locales"
SOURCE_LOCALE = LOCALES_DIR / "en.json"

def load_json(path: Path) -> dict:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)

def save_json(path: Path, data: dict):
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)
        f.write("\n")

def sync_locale(target_path: Path, source_data: dict, dry_run: bool = False) -> tuple[int, int]:
    """Syncs target_path against source_data. Returns (added_count, removed_count)."""
    target_data = load_json(target_path) if target_path.exists() else {}
    
    updated_data = {}
    added = 0
    
    for key, source_val in source_data.items():
        if key in target_data:
            updated_data[key] = target_data[key]
        else:
            updated_data[key] = f"[TODO: {source_val}]"
            added += 1
            
    removed = sum(1 for key in target_data if key not in source_data)
    
    if not dry_run and (added > 0 or removed > 0 or list(target_data.keys()) != list(updated_data.keys())):
        save_json(target_path, updated_data)
        
    return added, removed

def main():
    parser = argparse.ArgumentParser(description="Synchronize RustCraft localization files.")
    parser.add_argument("--new", type=str, help="Language code to create (e.g. de, es, fr, ja)")
    parser.add_argument("--check", action="store_true", help="Only check for missing keys without modifying files")
    args = parser.parse_args()

    if not SOURCE_LOCALE.exists():
        print(f"Error: Source locale {SOURCE_LOCALE} not found!", file=sys.stderr)
        sys.exit(1)

    source_data = load_json(SOURCE_LOCALE)
    print(f"Loaded {len(source_data)} source keys from {SOURCE_LOCALE.name}")

    if args.new:
        lang_code = args.new.lower().strip()
        new_file = LOCALES_DIR / f"{lang_code}.json"
        if new_file.exists():
            print(f"Locale file {new_file.name} already exists. Running sync...")
        else:
            print(f"Creating new locale file: {new_file.name}")
        added, _ = sync_locale(new_file, source_data, dry_run=False)
        print(f"Created/updated {new_file.name} with {len(source_data)} keys ({added} marked for translation).")
        return

    locale_files = sorted(LOCALES_DIR.glob("*.json"))
    has_issues = False

    for file_path in locale_files:
        if file_path == SOURCE_LOCALE:
            continue
        
        added, removed = sync_locale(file_path, source_data, dry_run=args.check)
        if added > 0 or removed > 0:
            has_issues = True
            action = "Found" if args.check else "Updated"
            print(f"[{file_path.name}] {action}: {added} missing keys added, {removed} obsolete keys removed.")
        else:
            print(f"[{file_path.name}] Up to date ({len(source_data)} keys).")

    if args.check and has_issues:
        print("\nRun `python3 scripts/sync_locales.py` to synchronize missing keys.")
        sys.exit(1)
    else:
        print("\nLocalization sync complete!")

if __name__ == "__main__":
    main()
