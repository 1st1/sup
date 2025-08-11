use grep_regex::RegexMatcher;
use grep_searcher::sinks::UTF8;
use grep_searcher::Searcher;
use ignore::WalkBuilder;
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::path::Path;
use std::process::Command;

#[pyclass]
struct RipGrep {
    pattern: String,
}

#[pymethods]
impl RipGrep {
    #[new]
    fn new(pattern: String) -> PyResult<Self> {
        // Validate the regex pattern immediately
        RegexMatcher::new(&pattern).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid regex: {}", e))
        })?;
        Ok(RipGrep { pattern })
    }

    fn search(&self, path: &str, py: Python) -> PyResult<Py<PyList>> {
        let results = PyList::empty_bound(py);
        let matcher = RegexMatcher::new(&self.pattern).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid regex: {}", e))
        })?;

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
    fn search_file_impl(
        &self,
        matcher: &RegexMatcher,
        path: &Path,
        results: &Bound<'_, PyList>,
    ) -> PyResult<()> {
        let mut searcher = Searcher::new();
        let mut matches = Vec::new();

        let sink = UTF8(|line_num, line| {
            matches.push((
                path.to_string_lossy().to_string(),
                line_num,
                line.to_string(),
            ));
            Ok(true)
        });

        searcher.search_path(matcher, path, sink).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Search error: {}", e))
        })?;

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

    fn search_directory_impl(
        &self,
        matcher: &RegexMatcher,
        path: &Path,
        results: &Bound<'_, PyList>,
    ) -> PyResult<()> {
        let walker = WalkBuilder::new(path).build();

        for entry in walker {
            let entry = entry.map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Walk error: {}", e))
            })?;

            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                if let Err(e) = self.search_file_impl(matcher, entry.path(), results) {
                    eprintln!("Error searching {}: {}", entry.path().display(), e);
                }
            }
        }

        Ok(())
    }
}

// Binary is now shipped as a file in the package, not embedded

#[pyfunction]
fn run_ripgrep(args: Vec<String>) -> PyResult<(i32, String, String)> {
    let binary_path = get_binary_path()?;

    // Run the binary
    let output = Command::new(&binary_path)
        .args(args)
        .output()
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to run ripgrep: {}", e))
        })?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok((exit_code, stdout, stderr))
}

fn get_binary_path() -> PyResult<std::path::PathBuf> {
    Python::with_gil(|py| {
        let sup_module = py.import_bound("sup")?;
        let file_attr = sup_module.getattr("__file__")?;
        let file_path = file_attr.extract::<String>()?;

        let binary_name = if cfg!(windows) { "rg.exe" } else { "rg" };
        let module_path = std::path::Path::new(&file_path);
        let binary_path = module_path
            .parent()
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyIOError, _>("Failed to get parent directory")
            })?
            .join("bin")
            .join(binary_name);

        if !binary_path.exists() {
            return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Ripgrep binary not found at: {}",
                binary_path.display()
            )));
        }

        Ok(binary_path)
    })
}

#[pyfunction]
fn get_ripgrep_path() -> PyResult<String> {
    let binary_path = get_binary_path()?;
    Ok(binary_path.to_string_lossy().to_string())
}

#[pymodule]
fn _sup(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RipGrep>()?;
    m.add_function(wrap_pyfunction!(run_ripgrep, m)?)?;
    m.add_function(wrap_pyfunction!(get_ripgrep_path, m)?)?;
    Ok(())
}
