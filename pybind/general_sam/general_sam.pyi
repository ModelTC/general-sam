from typing import Callable, Mapping, Optional, Sequence


class TrieNode:
    def get_node_id(self) -> int:
        ...

    def is_accepting(self) -> bool:
        ...

    def get_trans(self) -> Mapping[str, int]:
        ...

    def get_parent(self) -> int:
        ...


class Trie:
    def __init__(self) -> None:
        ...

    def insert_str(self, s: str) -> int:
        ...

    def get_bfs_order(self) -> Sequence[int]:
        ...

    def get_root(self) -> TrieNode:
        ...

    def get_node(self, node_id: int) -> Optional[TrieNode]:
        ...

    def dfs_travel(
        self,
        in_stack_callback: Callable[[int, Optional[str]], None],
        out_stack_callback: Callable[[int], None],
        root_node_id: Optional[int] = None,
    ) -> TrieNode:
        ...

    def num_of_nodes(self) -> int:
        ...


class GeneralSAMState:
    def get_node_id(self) -> int:
        ...

    def is_nil(self) -> bool:
        ...

    def is_root(self) -> bool:
        ...

    def is_accepting(self) -> bool:
        ...

    def get_suffix_parent_id(self) -> int:
        ...

    def goto_suffix_parent(self):
        ...

    def goto(self, t: str):
        ...

    def feed_str(self, s: str):
        ...


class GeneralSAM:
    @staticmethod
    def construct_from_str(s: str):
        ...

    @staticmethod
    def construct_from_trie(trie: Trie):
        ...

    def get_root_state(self) -> GeneralSAMState:
        ...

    def get_state(self, node_id: int) -> GeneralSAMState:
        ...

    def num_of_nodes(self) -> int:
        ...
