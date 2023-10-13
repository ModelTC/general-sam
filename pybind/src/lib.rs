extern crate general_sam as general_sam_rs;

use general_sam_rs::{sam, trie, trie_alike::TrieNodeAlike};
use pyo3::prelude::*;

#[pyclass]
struct Trie(trie::Trie<char>);

#[pyclass]
struct TrieNode(usize, trie::Node<char>);

#[pymethods]
impl TrieNode {
    fn get_node_id(&self) -> usize {
        self.0
    }

    fn is_accepting(&self) -> bool {
        self.1.accept
    }

    fn get_trans(&self) -> PyObject {
        Python::with_gil(|py| self.1.get_trans().clone().into_py(py))
    }

    fn get_parent(&self) -> usize {
        self.1.get_parent()
    }
}

#[pymethods]
impl Trie {
    #[new]
    fn new() -> Self {
        Trie(trie::Trie::default())
    }

    fn insert_chars(&mut self, s: &str) -> usize {
        self.0.insert_iter(s.chars())
    }

    fn get_bfs_order(&self) -> Vec<usize> {
        self.0.get_bfs_order()
    }

    fn get_node(&self, node_id: usize) -> TrieNode {
        TrieNode(node_id, self.0.get_node(node_id).clone())
    }

    fn dfs_travel(
        &self,
        in_stack_callback: PyObject,
        out_stack_callback: PyObject,
    ) -> Result<(), PyErr> {
        let mut stack = Vec::new();
        let in_stack = |node_id: usize, parent_id: Option<usize>, key: Option<char>| {
            Python::with_gil(|py| in_stack_callback.call1(py, (node_id, parent_id, key)))
                .map(|_| ())
        };
        let out_stack = |node_id: usize| {
            Python::with_gil(|py| out_stack_callback.call1(py, (node_id,))).map(|_| ())
        };

        stack.push(self.0.get_root_state().next_states());
        in_stack(trie::TRIE_ROOT_NODE_ID, None, None)?;

        while let Some(iter) = stack.last_mut() {
            let node_id = iter.get_state().node_id;
            if let Some((key, next_state)) = iter.next() {
                let next_node_id = next_state.node_id;
                stack.push(next_state.next_states());
                in_stack(next_node_id, Some(node_id), Some(key))?;
            } else {
                out_stack(node_id)?;
                stack.pop();
            }
        }
        Ok(())
    }
}

#[pyclass]
struct GeneralSAM(sam::GeneralSAM<char>);

#[pymethods]
impl GeneralSAM {
    #[staticmethod]
    fn construct_from_chars(s: &str) -> Self {
        GeneralSAM(sam::GeneralSAM::construct_from_chars(s.chars()))
    }

    #[staticmethod]
    fn construct_from_trie(trie: &Trie) -> Self {
        GeneralSAM(sam::GeneralSAM::construct_from_trie(
            trie.0.get_root_state(),
        ))
    }
}

#[pymodule]
fn general_sam(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Trie>()?;
    m.add_class::<GeneralSAM>()?;
    Ok(())
}
