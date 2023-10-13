from dataclasses import dataclass
from typing import Collection, Sequence, Tuple

from .general_sam import Trie


def construct_trie_from_chars(
    strings: Collection[str],
) -> Tuple[Trie, Sequence[int]]:
    trie = Trie.in_chars()
    node_ids = [trie.insert_chars(s) for s in strings]
    return trie, node_ids


def construct_trie_from_bytes(
    strings: Collection[bytes],
) -> Tuple[Trie, Sequence[int]]:
    trie = Trie.in_bytes()
    node_ids = [trie.insert_bytes(s) for s in strings]
    return trie, node_ids


@dataclass
class CountInfo:
    str_cnt: int
    tot_cnt_lower: int
    tot_cnt_upper: int


@dataclass
class SortResult:
    trie: Trie
    node_ids: Sequence[int]
    cnt_info_on_nodes: Sequence[CountInfo]
    cnt_info_on_strings: Sequence[CountInfo]
    order: Sequence[int]
    rank: Sequence[int]


def sort_chars(strings: Collection[str]) -> SortResult:
    trie, node_ids = construct_trie_from_chars(strings)
    return sort_seq_via_trie(trie, node_ids)


def sort_bytes(strings: Collection[bytes]) -> SortResult:
    trie, node_ids = construct_trie_from_bytes(strings)
    return sort_seq_via_trie(trie, node_ids)


def sort_seq_via_trie(trie: Trie, node_ids: Sequence[int]) -> SortResult:
    num_of_seq = len(node_ids)

    cnt_info_on_nodes = [CountInfo(0, 0, 0) for _ in range(trie.num_of_nodes())]
    for k in node_ids:
        cnt_info_on_nodes[k].str_cnt += 1

    tot_str_cnt = 0

    def in_stack(node_id: int, _):
        nonlocal tot_str_cnt
        node_info = cnt_info_on_nodes[node_id]
        node_info.tot_cnt_lower = tot_str_cnt
        tot_str_cnt += node_info.str_cnt

    def out_stack(node_id: int):
        nonlocal tot_str_cnt
        node_info = cnt_info_on_nodes[node_id]
        node_info.tot_cnt_upper = tot_str_cnt

    trie.dfs_travel(in_stack, out_stack)

    cnt_info_on_strings = [cnt_info_on_nodes[node_ids[i]] for i in range(num_of_seq)]

    order = sorted(
        range(num_of_seq),
        key=lambda i: cnt_info_on_strings[i].tot_cnt_lower,
    )
    rank = [0] * num_of_seq
    for k, i in enumerate(order):
        rank[i] = k

    return SortResult(
        trie=trie,
        node_ids=node_ids,
        cnt_info_on_nodes=cnt_info_on_nodes,
        cnt_info_on_strings=cnt_info_on_strings,
        order=order,
        rank=rank,
    )
