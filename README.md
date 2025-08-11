# sup

A Python package with Rust-based ripgrep implementation using maturin. This package bundles the full ripgrep binary alongside a Python search API.

## Features

- **Bundled ripgrep binary**: Full ripgrep CLI built from source and included in the wheel
- **Python search API**: Native Python interface for searching
- **Built from source**: All binaries are compiled from source during wheel building
- **Zero runtime dependencies**: ripgrep binary is embedded, no system installation needed
- Python 3.13+ support
- Cross-platform wheels for Linux, macOS, and Windows

## Installation

```bash
pip install sup
```

## Development

### Prerequisites

- Python 3.13+
- Rust toolchain
- maturin

### Setup

```bash
# Clone the repository
git clone <your-repo-url>
cd sup

# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install development dependencies
pip install maturin pytest

# Build and install locally
maturin develop
```

### Testing

```bash
pytest tests/
```

### Building Wheels

```bash
# Build wheel for current platform
maturin build --release

# Build wheels for all platforms (via CI)
# Push to GitHub and CI will build wheels for all platforms
```

## Usage

### Python API

```python
from sup import search, RipGrep, ripgrep, get_ripgrep_path

# Simple search using Python API
results = search("pattern", "/path/to/search")
for match in results:
    print(f"{match['file']}:{match['line_number']}: {match['line']}")

# Using RipGrep class directly
rg = RipGrep(r"test\d+")  # Regex pattern
matches = rg.search("/path/to/file.txt")

# Using the bundled ripgrep binary
result = ripgrep("pattern", ".", "--type", "py")
print(result.stdout)

# Get path to the ripgrep binary (extracted to temp directory)
rg_path = get_ripgrep_path()
print(f"ripgrep binary available at: {rg_path}")
```

### Command Line

After installation, you can use the bundled ripgrep via:

```bash
# Use the sup-rg command (installed as console script)
sup-rg pattern /path/to/search

# Or invoke via Python
python -m sup pattern /path/to/search
```

## CI/CD

The project uses GitHub Actions to automatically build wheels for multiple platforms:
- Linux (x86_64, aarch64)
- macOS (x86_64, ARM64)
- Windows (x64, x86)

Wheels are built for Python 3.13 and published to PyPI on tagged releases.

## License

MIT