import pathlib

BASE = pathlib.Path("third-party")

def flip(crate: str, block_old: str, block_new: str) -> None:
    path = BASE / crate / "Cargo.toml"
    text = path.read_text(encoding="utf-8")
    n = text.count(block_old)
    assert n == 1, f"{crate}: expected exactly 1 occurrence, found {n}"
    path.write_text(text.replace(block_old, block_new, 1), encoding="utf-8")
    print(f"flipped {crate}: gpui deps -> gpui-fast 0.1.8")

flip(
    "zorite-editor",
    '[dependencies.gpui]\nversion = "0.3"\npackage = "gpui-pre"',
    '[dependencies.gpui]\nversion = "0.1.8"\npackage = "gpui-fast"',
)
flip(
    "zorite-markdown",
    '[dependencies.gpui]\nversion = "0.3"\noptional = true\npackage = "gpui-pre"',
    '[dependencies.gpui]\nversion = "0.1.8"\noptional = true\npackage = "gpui-fast"',
)
flip(
    "gpui-bidi",
    '[dependencies.gpui]\nversion = "0.3"\npackage = "gpui-pre"',
    '[dependencies.gpui]\nversion = "0.1.8"\npackage = "gpui-fast"',
)
print("OK: 3 manifests flipped")