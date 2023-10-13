from typing import Callable, Mapping, Optional, Sequence, Union


class TrieNode:
    def is_in_chars(self) -> bool:
        ...

    def is_in_bytes(self) -> bool:
        ...

    def get_node_id(self) -> int:
        ...

    def is_accepting(self) -> bool:
        ...

    def get_trans(self) -> Mapping[Union[str, int], int]:
        ...

    def get_parent(self) -> int:
        ...


class Trie:
    @staticmethod
    def in_chars() -> 'Trie':
        ...

    @staticmethod
    def in_bytes() -> 'Trie':
        ...

    def is_in_chars(self) -> bool:
        ...

    def is_in_bytes(self) -> bool:
        ...

    def num_of_nodes(self) -> int:
        ...

    def insert_chars(self, s: str) -> int:
        ...

    def insert_bytes(self, s: bytes) -> int:
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

    def bfs_travel(
        self,
        in_queue_callback: Callable[[int, Optional[str]], None],
        out_queue_callback: Callable[[int], None],
        root_node_id: Optional[int] = None,
    ) -> TrieNode:
        ...


class GeneralSAMState:
    def is_in_str(self) -> bool:
        ...

    def is_in_bytes(self) -> bool:
        ...

    def get_node_id(self) -> int:
        ...

    def is_nil(self) -> bool:
        ...

    def is_root(self) -> bool:
        ...

    def is_accepting(self) -> bool:
        ...

    def get_trans(self) -> Mapping[Union[str, int], int]:
        ...

    def get_suffix_parent_id(self) -> int:
        ...

    def copy(self) -> 'GeneralSAMState':
        ...

    def goto_suffix_parent(self):
        ...

    def goto_char(self, t: str):
        ...

    def goto_byte(self, t: int):
        ...

    def feed_chars(self, s: str):
        ...

    def feed_bytes(self, s: bytes):
        ...

    def dfs_along(
        self,
        trie: Trie,
        in_stack_callback: Callable[['GeneralSAMState', int, Optional[str]], None],
        out_stack_callback: Callable[['GeneralSAMState', int], None],
        trie_node_id: Optional[int] = None,
    ) -> TrieNode:
        ...

    def bfs_along(
        self,
        trie: Trie,
        in_queue_callback: Callable[['GeneralSAMState', int, Optional[str]], None],
        out_queue_callback: Callable[['GeneralSAMState', int], None],
        trie_node_id: Optional[int] = None,
    ) -> TrieNode:
        ...


class GeneralSAM:
    @staticmethod
    def construct_from_chars(s: str) -> 'GeneralSAM':
        ...

    @staticmethod
    def construct_from_bytes(s: bytes) -> 'GeneralSAM':
        ...

    @staticmethod
    def construct_from_trie(trie: Trie) -> 'GeneralSAM':
        ...

    def is_in_str(self) -> bool:
        ...

    def is_in_bytes(self) -> bool:
        ...

    def num_of_nodes(self) -> int:
        ...

    def get_root_state(self) -> GeneralSAMState:
        ...

    def get_state(self, node_id: int) -> GeneralSAMState:
        ...

    def get_topo_order(self) -> Sequence[GeneralSAMState]:
        ...
