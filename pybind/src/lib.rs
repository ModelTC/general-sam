extern crate general_sam as general_sam_rs;

use std::{convert::Infallible, sync::Arc};

use general_sam_rs::{
    sam, trie,
    trie_alike::{TravelEvent, TrieNodeAlike},
};
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

    fn num_of_nodes(&self) -> usize {
        self.0.num_of_nodes()
    }

    fn insert_str(&mut self, s: &str) -> usize {
        self.0.insert_iter(s.chars())
    }

    fn get_bfs_order(&self) -> Vec<usize> {
        let state = self.0.get_root_state();
        let mut res = Vec::new();
        state
            .bfs_travel(|event| -> Result<(), Infallible> {
                if let TravelEvent::Push(s, _) = event {
                    res.push(s.node_id);
                }
                Ok(())
            })
            .unwrap();
        res
    }

    fn get_root(&self) -> TrieNode {
        self.get_node(trie::TRIE_ROOT_NODE_ID).unwrap()
    }

    fn get_node(&self, node_id: usize) -> Option<TrieNode> {
        self.0
            .get_node(node_id)
            .map(|node| TrieNode(node_id, node.clone()))
    }

    #[pyo3(signature = (in_stack_callback, out_stack_callback, root_node_id=None))]
    fn dfs_travel(
        &self,
        in_stack_callback: PyObject,
        out_stack_callback: PyObject,
        root_node_id: Option<usize>,
    ) -> Result<(), PyErr> {
        let root_state = self
            .0
            .get_state(root_node_id.unwrap_or(trie::TRIE_ROOT_NODE_ID));
        if root_state.is_nil() {
            return Ok(());
        }
        root_state.dfs_travel(|event| match event {
            TravelEvent::Push(tn, key_opt) => {
                Python::with_gil(|py| in_stack_callback.call1(py, (tn.node_id, key_opt.copied())))
                    .map(|_| ())
            }
            TravelEvent::Pop(tn) => {
                Python::with_gil(|py| out_stack_callback.call1(py, (tn.node_id,))).map(|_| ())
            }
        })
    }

    #[pyo3(signature = (in_stack_callback, out_stack_callback, root_node_id=None))]
    fn bfs_travel(
        &self,
        in_stack_callback: PyObject,
        out_stack_callback: PyObject,
        root_node_id: Option<usize>,
    ) -> Result<(), PyErr> {
        let root_state = self
            .0
            .get_state(root_node_id.unwrap_or(trie::TRIE_ROOT_NODE_ID));
        if root_state.is_nil() {
            return Ok(());
        }
        root_state.bfs_travel(|event| match event {
            TravelEvent::Push(tn, key_opt) => {
                Python::with_gil(|py| in_stack_callback.call1(py, (tn.node_id, key_opt.copied())))
                    .map(|_| ())
            }
            TravelEvent::Pop(tn) => {
                Python::with_gil(|py| out_stack_callback.call1(py, (tn.node_id,))).map(|_| ())
            }
        })
    }
}

#[pyclass]
struct GeneralSAM(Arc<sam::GeneralSAM<char>>);

#[pyclass]
struct GeneralSAMState(Arc<sam::GeneralSAM<char>>, usize);

impl GeneralSAMState {
    fn get_state(&self) -> sam::State<char> {
        self.0.get_state(self.1)
    }
}

#[pymethods]
impl GeneralSAMState {
    fn get_node_id(&self) -> usize {
        self.1
    }

    fn is_nil(&self) -> bool {
        self.get_state().is_nil()
    }

    fn is_root(&self) -> bool {
        self.get_state().is_root()
    }

    fn is_accepting(&self) -> bool {
        self.get_state().is_accepting()
    }

    fn get_suffix_parent_id(&self) -> usize {
        self.get_state()
            .get_node()
            .map(|node| node.get_suffix_parent_id())
            .unwrap_or(sam::SAM_NIL_NODE_ID)
    }

    fn goto_suffix_parent(&mut self) {
        let mut state = self.get_state();
        state.goto_suffix_parent();
        self.1 = state.node_id;
    }

    fn goto(&mut self, t: char) {
        let mut state = self.get_state();
        state.goto(&t);
        self.1 = state.node_id;
    }

    fn feed_str(&mut self, s: &str) {
        let state = self.get_state();
        let state = state.feed_chars(s);
        self.1 = state.node_id;
    }

    #[pyo3(signature = (trie, in_stack_callback, out_stack_callback, trie_node_id=None))]
    fn dfs_along(
        &self,
        trie: &Trie,
        in_stack_callback: PyObject,
        out_stack_callback: PyObject,
        trie_node_id: Option<usize>,
    ) -> Result<(), PyErr> {
        let tn = trie
            .0
            .get_state(trie_node_id.unwrap_or(trie::TRIE_ROOT_NODE_ID));
        self.0.dfs_along(tn, self.1, |event| match event {
            TravelEvent::Push((st, tn), key_opt) => Python::with_gil(|py| {
                in_stack_callback
                    .call1(
                        py,
                        (
                            GeneralSAMState(self.0.clone(), st.node_id),
                            tn.node_id,
                            key_opt.copied(),
                        ),
                    )
                    .map(|_| ())
            })
            .map(|_| ()),
            TravelEvent::Pop((st, tn)) => Python::with_gil(|py| {
                out_stack_callback
                    .call1(
                        py,
                        (GeneralSAMState(self.0.clone(), st.node_id), tn.node_id),
                    )
                    .map(|_| ())
            }),
        })
    }

    #[pyo3(signature = (trie, in_stack_callback, out_stack_callback, trie_node_id=None))]
    fn bfs_along(
        &self,
        trie: &Trie,
        in_stack_callback: PyObject,
        out_stack_callback: PyObject,
        trie_node_id: Option<usize>,
    ) -> Result<(), PyErr> {
        let tn = trie
            .0
            .get_state(trie_node_id.unwrap_or(trie::TRIE_ROOT_NODE_ID));
        self.0.bfs_along(tn, self.1, |event| match event {
            TravelEvent::Push((st, tn), key_opt) => Python::with_gil(|py| {
                in_stack_callback
                    .call1(
                        py,
                        (
                            GeneralSAMState(self.0.clone(), st.node_id),
                            tn.node_id,
                            key_opt.copied(),
                        ),
                    )
                    .map(|_| ())
            })
            .map(|_| ()),
            TravelEvent::Pop((st, tn)) => Python::with_gil(|py| {
                out_stack_callback
                    .call1(
                        py,
                        (GeneralSAMState(self.0.clone(), st.node_id), tn.node_id),
                    )
                    .map(|_| ())
            }),
        })
    }
}

#[pymethods]
impl GeneralSAM {
    #[staticmethod]
    fn construct_from_str(s: &str) -> Self {
        GeneralSAM(Arc::new(sam::GeneralSAM::construct_from_chars(s.chars())))
    }

    #[staticmethod]
    fn construct_from_trie(trie: &Trie) -> Self {
        GeneralSAM(Arc::new(sam::GeneralSAM::construct_from_trie(
            trie.0.get_root_state(),
        )))
    }

    fn num_of_nodes(&self) -> usize {
        self.0.num_of_nodes()
    }

    fn get_root_state(&self) -> GeneralSAMState {
        GeneralSAMState(self.0.clone(), sam::SAM_ROOT_NODE_ID)
    }

    fn get_state(&self, node_id: usize) -> GeneralSAMState {
        GeneralSAMState(self.0.clone(), node_id)
    }

    fn get_topo_order(&self) -> Vec<GeneralSAMState> {
        self.0
            .get_topo_order()
            .map(|s| self.get_state(s.node_id))
            .collect()
    }
}

#[pymodule]
fn general_sam(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<TrieNode>()?;
    m.add_class::<Trie>()?;
    m.add_class::<GeneralSAMState>()?;
    m.add_class::<GeneralSAM>()?;
    Ok(())
}
