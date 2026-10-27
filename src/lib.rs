use pyo3::prelude::*;
use kolibrie_core::SparqlDatabase; // Core engine mapping straight from upstream

#[pyclass]
struct LocalSparqlNode {
    // Wrap upstream native database engine cleanly inside our pyclass structure
    db: SparqlDatabase,
}

#[pymethods]
impl LocalSparqlNode {
    #[new]
    fn new() -> Self {
        LocalSparqlNode {
            db: SparqlDatabase::new(),
        }
    }

    fn load_turtle_string(&mut self, data: &str) -> PyResult<String> {
        self.db.parse_turtle(data);
        Ok("Turtle triple string registered successfully.".to_string())
    }

    fn query(&mut self, query_str: &str) -> PyResult<Vec<Vec<String>>> {
        // Direct execution path using the volcano cost-based query optimizer paths
        let rows = kolibrie_core::execute_query::execute_query(query_str, &mut self.db);
        Ok(rows)
    }
}

/// Native PyO3 module interface definition
#[pymodule]
fn kolibrie(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<LocalSparqlNode>()?;
    Ok(())
}
