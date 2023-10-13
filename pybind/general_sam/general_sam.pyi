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


class GeneralSAM:
    @staticmethod
    def construct_from_str(s: str):
        ...

    @staticmethod
    def construct_from_trie(trie: Trie):
        ...
