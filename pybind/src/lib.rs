extern crate general_sam as general_sam_rs;

use std::{convert::Infallible, str::from_utf8, sync::Arc};

use either::{for_both, Either};
use pyo3::{prelude::*, types::PyDict};

use general_sam_rs::{
    sam, trie,
    trie_alike::{TravelEvent, TrieNodeAlike},
};

#[pyclass]
struct Trie(Either<trie::Trie<char>, trie::Trie<u8>>);

#[pyclass]
struct TrieNode(usize, Either<trie::Node<char>, trie::Node<u8>>);

#[pymethods]
impl TrieNode {
    fn is_in_chars(&self) -> bool {
        self.1.is_left()
    }

    fn is_in_bytes(&self) -> bool {
        self.1.is_right()
    }

    fn get_node_id(&self) -> usize {
        self.0
    }

    fn is_accepting(&self) -> bool {
        for_both!(self.1.as_ref(), x => x.accept)
    }

    fn get_trans(&self) -> PyObject {
        Python::with_gil(|py| {
            for_both!(self.1.as_ref(), x => {
                x.get_trans().clone().into_py(py)
            })
        })
    }

    fn get_parent(&self) -> usize {
        for_both!(self.1.as_ref(), x => x.get_parent())
    }
}

#[pymethods]
impl Trie {
    #[staticmethod]
    fn in_chars() -> Self {
        Trie(Either::Left(trie::Trie::default()))
    }

    #[staticmethod]
    fn in_bytes() -> Self {
        Trie(Either::Right(trie::Trie::default()))
    }

    fn is_in_chars(&self) -> bool {
        self.0.is_left()
    }

    fn is_in_bytes(&self) -> bool {
        self.0.is_right()
    }

    fn num_of_nodes(&self) -> usize {
        for_both!(self.0.as_ref(), x => x.num_of_nodes())
    }

    fn insert_chars(&mut self, s: &str) -> usize {
        match self.0.as_mut() {
            Either::Left(trie_chars) => trie_chars.insert_iter(s.chars()),
            Either::Right(trie_bytes) => trie_bytes.insert_ref_iter(s.as_bytes().iter()),
        }
    }

    fn insert_bytes(&mut self, b: &[u8]) -> usize {
        match self.0.as_mut() {
            Either::Left(trie_chars) => trie_chars.insert_iter(from_utf8(b).unwrap().chars()),
            Either::Right(trie_bytes) => trie_bytes.insert_ref_iter(b.iter()),
        }
    }

    fn get_bfs_order(&self) -> Vec<usize> {
        for_both!(self.0.as_ref(), trie => {
            let state = trie.get_root_state();
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
        })
    }

    fn get_root(&self) -> TrieNode {
        self.get_node(trie::TRIE_ROOT_NODE_ID).unwrap()
    }

    fn get_node(&self, node_id: usize) -> Option<TrieNode> {
        match self.0.as_ref() {
            Either::Left(trie) => trie
                .get_node(node_id)
                .map(|node| TrieNode(node_id, Either::Left(node.clone()))),
            Either::Right(trie) => trie
                .get_node(node_id)
                .map(|node| TrieNode(node_id, Either::Right(node.clone()))),
        }
    }

    #[pyo3(signature = (in_stack_callback, out_stack_callback, root_node_id=None))]
    fn dfs_travel(
        &self,
        in_stack_callback: PyObject,
        out_stack_callback: PyObject,
        root_node_id: Option<usize>,
    ) -> Result<(), PyErr> {
        for_both!(self.0.as_ref(), trie => {
            let root_state = trie.get_state(root_node_id.unwrap_or(trie::TRIE_ROOT_NODE_ID));
            if root_state.is_nil() {
                return Ok(());
            }
            root_state.dfs_travel(|event| match event {
                TravelEvent::Push(tn, key_opt) => Python::with_gil(|py| {
                    in_stack_callback.call1(py, (tn.node_id, key_opt.copied()))
                })
                .map(|_| ()),
                TravelEvent::Pop(tn) => {
                    Python::with_gil(|py| out_stack_callback.call1(py, (tn.node_id,))).map(|_| ())
                }
            })
        })
    }

    #[pyo3(signature = (in_stack_callback, out_stack_callback, root_node_id=None))]
    fn bfs_travel(
        &self,
        in_stack_callback: PyObject,
        out_stack_callback: PyObject,
        root_node_id: Option<usize>,
    ) -> Result<(), PyErr> {
        for_both!(self.0.as_ref(), trie => {
            let root_state = trie.get_state(root_node_id.unwrap_or(trie::TRIE_ROOT_NODE_ID));
            if root_state.is_nil() {
                return Ok(());
            }
            root_state.bfs_travel(|event| match event {
                TravelEvent::Push(tn, key_opt) => Python::with_gil(|py| {
                    in_stack_callback.call1(py, (tn.node_id, key_opt.copied()))
                })
                .map(|_| ()),
                TravelEvent::Pop(tn) => {
                    Python::with_gil(|py| out_stack_callback.call1(py, (tn.node_id,))).map(|_| ())
                }
            })
        })
    }
}

#[pyclass]
struct GeneralSAM(Arc<Either<sam::GeneralSAM<char>, sam::GeneralSAM<u8>>>);

#[pyclass]
#[derive(Clone)]
struct GeneralSAMState(
    Arc<Either<sam::GeneralSAM<char>, sam::GeneralSAM<u8>>>,
    usize,
);

impl GeneralSAMState {
    fn get_state(&self) -> Either<sam::State<char>, sam::State<u8>> {
        self.0
            .as_ref()
            .as_ref()
            .map_either(|x| x.get_state(self.1), |x| x.get_state(self.1))
    }
}

#[pymethods]
impl GeneralSAMState {
    fn is_in_chars(&self) -> bool {
        self.0.is_left()
    }

    fn is_in_bytes(&self) -> bool {
        self.0.is_right()
    }

    fn get_node_id(&self) -> usize {
        self.1
    }

    fn is_nil(&self) -> bool {
        for_both!(self.get_state().as_ref(), x => x.is_nil())
    }

    fn is_root(&self) -> bool {
        for_both!(self.get_state().as_ref(), x => x.is_root())
    }

    fn is_accepting(&self) -> bool {
        for_both!(self.get_state().as_ref(), x => x.is_accepting())
    }

    fn get_trans(&self) -> PyObject {
        Python::with_gil(|py| {
            for_both!(self.get_state().as_ref(), state => {
                if let Some(node) = state.get_node() {
                    node.get_trans().clone().into_py(py)
                } else {
                    PyDict::new(py).into_py(py)
                }
            })
        })
    }

    fn get_suffix_parent_id(&self) -> usize {
        for_both!(self.get_state().as_ref() , x => {
            x.get_node()
                .map(|node| node.get_suffix_parent_id())
                .unwrap_or(sam::SAM_NIL_NODE_ID)
        })
    }

    fn copy(&self) -> Self {
        self.clone()
    }

    fn goto_suffix_parent(&mut self) {
        for_both!(self.get_state(), mut state => {
            state.goto_suffix_parent();
            self.1 = state.node_id;
        })
    }

    fn goto_char(&mut self, t: char) {
        let mut state = self.get_state().left().unwrap();
        state.goto(&t);
        self.1 = state.node_id;
    }

    fn goto_byte(&mut self, t: u8) {
        let mut state = self.get_state().right().unwrap();
        state.goto(&t);
        self.1 = state.node_id;
    }

    fn feed_chars(&mut self, s: &str) {
        match self.get_state() {
            Either::Left(state_chars) => {
                let state_chars = state_chars.feed_chars(s);
                self.1 = state_chars.node_id;
            }
            Either::Right(state_bytes) => {
                let state_bytes = state_bytes.feed_ref_iter(s.as_bytes().iter());
                self.1 = state_bytes.node_id;
            }
        }
    }

    fn feed_bytes(&mut self, s: &[u8]) {
        match self.get_state() {
            Either::Left(state_chars) => {
                let state_chars = state_chars.feed_iter(from_utf8(s).unwrap().chars());
                self.1 = state_chars.node_id;
            }
            Either::Right(state_bytes) => {
                let state_bytes = state_bytes.feed_ref_iter(s.iter());
                self.1 = state_bytes.node_id;
            }
        }
    }

    #[pyo3(signature = (trie, in_stack_callback, out_stack_callback, trie_node_id=None))]
    fn dfs_along(
        &self,
        trie: &Trie,
        in_stack_callback: PyObject,
        out_stack_callback: PyObject,
        trie_node_id: Option<usize>,
    ) -> Result<(), PyErr> {
        assert!(trie.is_in_chars() == self.is_in_chars());
        let sam_and_trie = self.0.as_ref().as_ref().map_either(
            |sam_chars| (sam_chars, trie.0.as_ref().left().unwrap()),
            |sam_bytes| (sam_bytes, trie.0.as_ref().right().unwrap()),
        );
        for_both!(sam_and_trie, (sam, trie) => {
            let tn = trie.get_state(trie_node_id.unwrap_or(trie::TRIE_ROOT_NODE_ID));
            sam.dfs_along(tn, self.1, |event| match event {
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
        assert!(trie.is_in_chars() == self.is_in_chars());
        let sam_and_trie = self.0.as_ref().as_ref().map_either(
            |sam_chars| (sam_chars, trie.0.as_ref().left().unwrap()),
            |sam_bytes| (sam_bytes, trie.0.as_ref().right().unwrap()),
        );
        for_both!(sam_and_trie, (sam, trie) => {
            let tn = trie.get_state(trie_node_id.unwrap_or(trie::TRIE_ROOT_NODE_ID));
            sam.bfs_along(tn, self.1, |event| match event {
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
        })
    }
}

#[pymethods]
impl GeneralSAM {
    #[staticmethod]
    fn construct_from_chars(s: &str) -> Self {
        GeneralSAM(Arc::new(Either::Left(
            sam::GeneralSAM::construct_from_chars(s.chars()),
        )))
    }

    #[staticmethod]
    fn construct_from_bytes(s: &[u8]) -> Self {
        GeneralSAM(Arc::new(Either::Right(
            sam::GeneralSAM::construct_from_bytes(s),
        )))
    }

    #[staticmethod]
    fn construct_from_trie(trie: &Trie) -> Self {
        match trie.0.as_ref() {
            Either::Left(trie_chars) => GeneralSAM(Arc::new(Either::Left(
                sam::GeneralSAM::construct_from_trie(trie_chars.get_root_state()),
            ))),
            Either::Right(trie_bytes) => GeneralSAM(Arc::new(Either::Right(
                sam::GeneralSAM::construct_from_trie(trie_bytes.get_root_state()),
            ))),
        }
    }

    fn is_in_chars(&self) -> bool {
        self.0.is_left()
    }

    fn is_in_bytes(&self) -> bool {
        self.0.is_right()
    }

    fn num_of_nodes(&self) -> usize {
        for_both!(self.0.as_ref(), x => x.num_of_nodes())
    }

    fn get_root_state(&self) -> GeneralSAMState {
        GeneralSAMState(self.0.clone(), sam::SAM_ROOT_NODE_ID)
    }

    fn get_state(&self, node_id: usize) -> GeneralSAMState {
        GeneralSAMState(self.0.clone(), node_id)
    }

    fn get_topo_order(&self) -> Vec<GeneralSAMState> {
        for_both!(self.0.as_ref(), x => {
            x.get_topo_order()
                .map(|s| self.get_state(s.node_id))
                .collect()
        })
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
