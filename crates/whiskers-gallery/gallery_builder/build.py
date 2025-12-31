"""Build script for whiskers gallery.

Generates HTML pages for each sketch with embedded source code.
"""

from importlib.resources import files
from pathlib import Path

from jinja2 import Environment, PackageLoader

# Sketch metadata - should match SKETCH_MANIFEST in mod.rs
SKETCHES = [
    {
        "id": "schotter",
        "name": "Schotter",
        "description": "Recreation of Georg Nees' classic 1968-1970 generative art piece",
        "author": "Antoine Beyeler",
    },
    {
        "id": "hello_world",
        "name": "Hello World",
        "description": "A simple introductory sketch demonstrating basic whiskers usage",
        "author": "Antoine Beyeler",
    },
]


def get_crate_root() -> Path:
    """Get the root directory of the whiskers-gallery crate."""
    return Path(__file__).parent.parent


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

    # Ensure output directories exist
    sketches_dir.mkdir(parents=True, exist_ok=True)

    # Set up Jinja environment with package loader
    env = Environment(loader=PackageLoader("gallery_builder", "templates"))
    sketch_template = env.get_template("sketch.html.jinja")
    index_template = env.get_template("index.html.jinja")

    # Build each sketch page
    for sketch in SKETCHES:
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
    landing_html = index_template.render(sketches=SKETCHES)
    (web_dir / "index.html").write_text(landing_html)

    print(f"Gallery built successfully in {web_dir}")


def main() -> None:
    """Entry point for the gallery-build command."""
    build_gallery()


if __name__ == "__main__":
    main()
