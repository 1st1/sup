use pyo3::prelude::*;
use pyo3::types::{PyList, PyBytes};
use grep_regex::RegexMatcher;
use grep_searcher::sinks::UTF8;
use grep_searcher::Searcher;
use ignore::WalkBuilder;
use std::path::Path;
use std::fs;
use std::process::Command;
use std::env;

#[pyclass]
struct RipGrep {
    pattern: String,
}

#[pymethods]
impl RipGrep {
    #[new]
    fn new(pattern: String) -> PyResult<Self> {
        // Validate the regex pattern immediately
        RegexMatcher::new(&pattern)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid regex: {}", e)))?;
        Ok(RipGrep { pattern })
    }

    fn search(&self, path: &str, py: Python) -> PyResult<Py<PyList>> {
        let results = PyList::empty_bound(py);
        let matcher = RegexMatcher::new(&self.pattern)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid regex: {}", e)))?;
        
        let search_path = Path::new(path);
        
        if search_path.is_file() {
            self.search_file_impl(&matcher, search_path, &results)?;
        } else if search_path.is_dir() {
            self.search_directory_impl(&matcher, search_path, &results)?;
        }
        
        Ok(results.into())
    }
}

impl RipGrep {
    fn search_file_impl(&self, matcher: &RegexMatcher, path: &Path, results: &Bound<'_, PyList>) -> PyResult<()> {
        let mut searcher = Searcher::new();
        let mut matches = Vec::new();
        
        let sink = UTF8(|line_num, line| {
            matches.push((path.to_string_lossy().to_string(), line_num, line.to_string()));
            Ok(true)
        });
        
        searcher.search_path(matcher, path, sink)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Search error: {}", e)))?;
        
        for (file_path, line_num, line) in matches {
            Python::with_gil(|py| {
                let dict = pyo3::types::PyDict::new_bound(py);
                dict.set_item("file", file_path)?;
                dict.set_item("line_number", line_num)?;
                dict.set_item("line", line.trim_end())?;
                results.append(dict)?;
                Ok::<_, PyErr>(())
            })?;
        }
        
        Ok(())
    }

    fn search_directory_impl(&self, matcher: &RegexMatcher, path: &Path, results: &Bound<'_, PyList>) -> PyResult<()> {
        let walker = WalkBuilder::new(path)
            .build();
        
        for entry in walker {
            let entry = entry.map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Walk error: {}", e)))?;
            
            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                if let Err(e) = self.search_file_impl(matcher, entry.path(), results) {
                    eprintln!("Error searching {}: {}", entry.path().display(), e);
                }
            }
        }
        
        Ok(())
    }
}

const RIPGREP_BINARY: &[u8] = include_bytes!(env!("RIPGREP_BINARY_PATH"));

#[pyfunction]
fn get_ripgrep_binary(py: Python) -> PyResult<Py<PyBytes>> {
    Ok(PyBytes::new_bound(py, RIPGREP_BINARY).into())
}

#[pyfunction]
fn run_ripgrep(args: Vec<String>, py: Python) -> PyResult<(i32, String, String)> {
    py.allow_threads(|| {
        let temp_dir = env::temp_dir();
        let binary_name = if cfg!(windows) { "rg.exe" } else { "rg" };
        let binary_path = temp_dir.join(format!("sup_ripgrep_{}", binary_name));
        
        // Write binary to temp location if it doesn't exist
        if !binary_path.exists() {
            fs::write(&binary_path, RIPGREP_BINARY)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to write ripgrep binary: {}", e)))?;
        }
        
        // Make it executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to get metadata: {}", e)))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to set permissions: {}", e)))?;
        }
        
        // Run the binary
        let output = Command::new(&binary_path)
            .args(args)
            .output()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to run ripgrep: {}", e)))?;
        
        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        // Don't clean up temp binary - reuse it for performance
        // fs::remove_file(&binary_path).ok();
        
        Ok((exit_code, stdout, stderr))
    })
}

#[pyfunction]
fn get_ripgrep_path() -> PyResult<String> {
    let temp_dir = env::temp_dir();
    let binary_name = if cfg!(windows) { "rg.exe" } else { "rg" };
    let binary_path = temp_dir.join(format!("sup_ripgrep_{}", binary_name));
    
    // Write binary if it doesn't exist
    if !binary_path.exists() {
        fs::write(&binary_path, RIPGREP_BINARY)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to write ripgrep binary: {}", e)))?;
        
        // Make it executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to get metadata: {}", e)))?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to set permissions: {}", e)))?;
        }
    }
    
    Ok(binary_path.to_string_lossy().to_string())
}

#[pymodule]
fn _sup(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RipGrep>()?;
    m.add_function(wrap_pyfunction!(get_ripgrep_binary, m)?)?;
    m.add_function(wrap_pyfunction!(run_ripgrep, m)?)?;
    m.add_function(wrap_pyfunction!(get_ripgrep_path, m)?)?;
    Ok(())
}