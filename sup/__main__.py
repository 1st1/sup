"""
Command-line entry point for sup package.
Allows running as: python -m sup [args]
"""

import sys

from sup import ripgrep_cli

if __name__ == "__main__":
    sys.exit(ripgrep_cli())
