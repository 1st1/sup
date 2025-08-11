import os
import tempfile
import pytest
import subprocess
from sup import RipGrep, search, ripgrep, ripgrep_cli, get_ripgrep_path


def test_ripgrep_initialization():
    rg = RipGrep("test")
    assert rg is not None


def test_search_in_file():
    with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
        f.write("Hello world\n")
        f.write("This is a test file\n")
        f.write("Testing ripgrep implementation\n")
        f.write("Another test line\n")
        temp_file = f.name
    
    try:
        results = search("test", temp_file)
        assert len(results) == 2
        assert all("test" in r["line"].lower() for r in results)
        assert all(r["file"] == temp_file for r in results)
    finally:
        os.unlink(temp_file)


def test_search_in_directory():
    with tempfile.TemporaryDirectory() as temp_dir:
        file1 = os.path.join(temp_dir, "file1.txt")
        file2 = os.path.join(temp_dir, "file2.txt")
        
        with open(file1, 'w') as f:
            f.write("Pattern match here\n")
            f.write("No match on this line\n")
        
        with open(file2, 'w') as f:
            f.write("Another pattern match\n")
            f.write("Just some text\n")
        
        results = search("pattern", temp_dir)
        # Use case-insensitive search since ripgrep is case-sensitive by default
        results = search("[Pp]attern", temp_dir)
        assert len(results) == 2
        assert all("pattern" in r["line"].lower() for r in results)


def test_regex_pattern():
    with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
        f.write("test123\n")
        f.write("test456\n")
        f.write("test\n")
        f.write("testing789\n")
        temp_file = f.name
    
    try:
        results = search(r"test\d+", temp_file)
        assert len(results) == 2
        assert any("test123" in r["line"] for r in results)
        assert any("test456" in r["line"] for r in results)
    finally:
        os.unlink(temp_file)


def test_empty_results():
    with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
        f.write("Hello world\n")
        f.write("No matches here\n")
        temp_file = f.name
    
    try:
        results = search("nonexistent", temp_file)
        assert len(results) == 0
    finally:
        os.unlink(temp_file)


def test_invalid_regex():
    with pytest.raises(ValueError, match="Invalid regex"):
        RipGrep("[invalid")


def test_ripgrep_binary_execution():
    # Test basic ripgrep binary execution
    with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
        f.write("Hello world\n")
        f.write("Test line\n")
        f.write("Another test\n")
        temp_file = f.name
    
    try:
        # Test with ripgrep function
        result = ripgrep("test", temp_file, "-i")
        assert result.returncode in (0, 1)
        assert "test" in result.stdout.lower()
        assert result.stdout.count('\n') == 2  # Two matching lines
    finally:
        os.unlink(temp_file)


def test_ripgrep_binary_no_matches():
    with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
        f.write("Hello world\n")
        temp_file = f.name
    
    try:
        result = ripgrep("nonexistent", temp_file)
        assert result.returncode == 1  # ripgrep returns 1 for no matches
        assert result.stdout == ""
    finally:
        os.unlink(temp_file)


def test_ripgrep_binary_with_type_filter():
    with tempfile.TemporaryDirectory() as temp_dir:
        # Create Python file
        py_file = os.path.join(temp_dir, "test.py")
        with open(py_file, 'w') as f:
            f.write("def test_function():\n")
            f.write("    pass\n")
        
        # Create text file
        txt_file = os.path.join(temp_dir, "test.txt")
        with open(txt_file, 'w') as f:
            f.write("test content\n")
        
        # Search only in Python files
        result = ripgrep("test", temp_dir, "--type", "py")
        assert result.returncode == 0
        assert "test.py" in result.stdout
        assert "test.txt" not in result.stdout


def test_ripgrep_cli_function():
    # Test the CLI wrapper function
    with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
        f.write("CLI test\n")
        temp_file = f.name
    
    try:
        # Simulate CLI args
        exit_code = ripgrep_cli(["CLI", temp_file])
        assert exit_code == 0
    finally:
        os.unlink(temp_file)


def test_get_ripgrep_path():
    # Test getting the path to the ripgrep binary
    path = get_ripgrep_path()
    assert path
    assert os.path.exists(path)
    
    # Test that the binary is executable
    result = subprocess.run([path, "--version"], capture_output=True, text=True)
    assert result.returncode == 0
    assert "ripgrep" in result.stdout.lower()


def test_ripgrep_binary_regex():
    with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
        f.write("test123\n")
        f.write("test456\n")
        f.write("test\n")
        temp_file = f.name
    
    try:
        # Use regex pattern
        result = ripgrep(r"test\d+", temp_file)
        assert result.returncode == 0
        assert "test123" in result.stdout
        assert "test456" in result.stdout
        assert result.stdout.count('\n') == 2
    finally:
        os.unlink(temp_file)