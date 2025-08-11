from sup._sup import RipGrep, run_ripgrep, get_ripgrep_binary, get_ripgrep_path
import subprocess
import sys
from typing import Optional, List, Union

__version__ = "0.1.0"
__all__ = ["RipGrep", "search", "ripgrep", "ripgrep_cli", "get_ripgrep_path"]


def search(pattern: str, path: str = ".") -> list[dict]:
    """
    Search for a pattern in files using ripgrep.
    
    Args:
        pattern: Regular expression pattern to search for
        path: File or directory path to search in (default: current directory)
    
    Returns:
        List of dictionaries containing:
            - file: Path to the file
            - line_number: Line number of the match
            - line: The matching line content
    """
    rg = RipGrep(pattern)
    return rg.search(path)


def ripgrep(*args: str, check: bool = True) -> subprocess.CompletedProcess:
    """
    Run the bundled ripgrep binary with the given arguments.
    
    Args:
        *args: Command line arguments to pass to ripgrep
        check: If True, raises CalledProcessError if ripgrep returns non-zero
    
    Returns:
        CompletedProcess object with returncode, stdout, and stderr
    
    Example:
        >>> result = ripgrep("pattern", ".", "--type", "py")
        >>> print(result.stdout)
    """
    exit_code, stdout, stderr = run_ripgrep(list(args))
    
    result = subprocess.CompletedProcess(
        args=["rg"] + list(args),
        returncode=exit_code,
        stdout=stdout,
        stderr=stderr
    )
    
    if check and exit_code not in (0, 1):  # ripgrep returns 1 for no matches
        raise subprocess.CalledProcessError(exit_code, result.args, stdout, stderr)
    
    return result


def ripgrep_cli(args: Optional[List[str]] = None) -> int:
    """
    Run ripgrep as a CLI tool, using sys.argv if no args provided.
    This is useful for creating a 'rg' command-line entry point.
    
    Args:
        args: Optional list of arguments. If None, uses sys.argv[1:]
    
    Returns:
        Exit code from ripgrep
    """
    if args is None:
        args = sys.argv[1:]
    
    exit_code, stdout, stderr = run_ripgrep(args)
    
    if stdout:
        sys.stdout.write(stdout)
        sys.stdout.flush()
    if stderr:
        sys.stderr.write(stderr)
        sys.stderr.flush()
    
    return exit_code