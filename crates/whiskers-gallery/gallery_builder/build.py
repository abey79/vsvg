"""Build script for whiskers gallery.

Generates HTML pages for each sketch with embedded source code.
Reads sketch metadata from sketches.toml (single source of truth).
"""

import tomllib
from pathlib import Path

from jinja2 import Environment, PackageLoader


def get_crate_root() -> Path:
    """Get the root directory of the whiskers-gallery crate."""
    return Path(__file__).parent.parent


def load_sketches() -> list[dict]:
    """Load sketch metadata from sketches.toml."""
    toml_path = get_crate_root() / "sketches.toml"
    with open(toml_path, "rb") as f:
        data = tomllib.load(f)
    return data.get("sketch", [])


def read_sketch_source(sketch_id: str) -> str:
    """Read the source code for a sketch."""
    source_path = get_crate_root() / "src" / "sketches" / f"{sketch_id}.rs"
    if source_path.exists():
        return source_path.read_text()
    return f"// Source not found: {source_path}"


def build_gallery() -> None:
    """Build all gallery pages."""
    crate_root = get_crate_root()
    web_dir = crate_root / "web"
    sketches_dir = web_dir / "sketches"

    # Load sketch metadata from TOML
    sketches = load_sketches()

    # Ensure output directories exist
    sketches_dir.mkdir(parents=True, exist_ok=True)

    # Set up Jinja environment with package loader
    env = Environment(loader=PackageLoader("gallery_builder", "templates"))
    sketch_template = env.get_template("sketch.html.jinja")
    index_template = env.get_template("index.html.jinja")

    # Build each sketch page
    for sketch in sketches:
        sketch_id = sketch["id"]
        print(f"Building sketch page: {sketch_id}")

        # Read source
        source = read_sketch_source(sketch_id)

        # Render page
        html = sketch_template.render(sketch=sketch, source=source)

        # Write output
        output_dir = sketches_dir / sketch_id
        output_dir.mkdir(parents=True, exist_ok=True)
        (output_dir / "index.html").write_text(html)

    # Build landing page
    print("Building landing page")
    landing_html = index_template.render(sketches=sketches)
    (web_dir / "index.html").write_text(landing_html)

    print(f"Gallery built successfully in {web_dir}")


def main() -> None:
    """Entry point for the gallery-build command."""
    build_gallery()


if __name__ == "__main__":
    main()
