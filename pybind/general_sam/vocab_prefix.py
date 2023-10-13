import enum
from dataclasses import replace
from typing import (
    Callable,
    Iterable,
    List,
    Optional,
    Sequence,
    Tuple,
    Union,
    cast,
)

from .general_sam import GeneralSAM, GeneralSAMState, Trie
from .trie_utils import (
    CountInfo,
    SortResult,
    construct_trie_from_bytes,
    construct_trie_from_chars,
    sort_bytes,
    sort_chars,
)


class VocabPrefixBytesOrChars(enum.Enum):
    BYTES = enum.auto()
    CHARS = enum.auto()


class VocabPrefixAutomaton(object):
    def __init__(
        self,
        vocab: Iterable[Union[str, bytes]],
        bytes_or_chars: Union[
            str, VocabPrefixBytesOrChars
        ] = VocabPrefixBytesOrChars.CHARS,
    ) -> None:
        if isinstance(bytes_or_chars, str):
            bytes_or_chars = getattr(VocabPrefixBytesOrChars, bytes_or_chars.upper())

        self.bytes_or_chars = cast(VocabPrefixBytesOrChars, bytes_or_chars)

        self.vocab: Sequence[Union[str, bytes]] = list(vocab)

        if self.bytes_or_chars == VocabPrefixBytesOrChars.BYTES and isinstance(
            self.vocab[0], str
        ):
            self.vocab = list(cast(str, i).encode() for i in self.vocab)
        if self.bytes_or_chars == VocabPrefixBytesOrChars.CHARS and isinstance(
            self.vocab[0], bytes
        ):
            self.vocab = list(cast(bytes, i).decode() for i in self.vocab)

        self.vocab_rev: Sequence[Union[str, bytes]] = list(s[::-1] for s in vocab)

        sort_seq, construct_trie = {
            VocabPrefixBytesOrChars.BYTES: (sort_bytes, construct_trie_from_bytes),
            VocabPrefixBytesOrChars.CHARS: (sort_chars, construct_trie_from_chars),
        }[self.bytes_or_chars]
        self.vocab_sort_res = cast(SortResult, sort_seq(self.vocab))
        self.trie_rev, self.trie_rev_node_ids = cast(
            Tuple[Trie, Sequence[int]],
            construct_trie(self.vocab_rev),
        )

        self.sam_rev = GeneralSAM.construct_from_trie(self.trie_rev)
        self._gen_cnt_info_in_sam()

    @property
    def _state_feed_fn(self) -> Callable[[GeneralSAMState, Union[bytes, str]], None]:
        return {
            VocabPrefixBytesOrChars.BYTES: GeneralSAMState.feed_bytes,
            VocabPrefixBytesOrChars.CHARS: GeneralSAMState.feed_chars,
        }[self.bytes_or_chars]

    def _gen_cnt_info_in_sam(self):
        self.cnt_info_in_sam: List[Optional[CountInfo]] = [
            None for _ in range(self.sam_rev.num_of_nodes())
        ]

        for token_rev, cnt_info in zip(
            self.vocab_rev, self.vocab_sort_res.cnt_info_on_strings
        ):
            state = self.sam_rev.get_root_state()
            self._state_feed_fn(state, token_rev)
            state_id = state.get_node_id()
            self.cnt_info_in_sam[state_id] = replace(cnt_info, str_cnt=1)

        for sam_state in reversed(self.sam_rev.get_topo_order()):
            assert not sam_state.is_nil()
            if sam_state.is_root():
                continue

            state_id = sam_state.get_node_id()
            state_cnt_info = self.cnt_info_in_sam[state_id]
            if state_cnt_info is None:
                continue

            link_id = sam_state.get_suffix_parent_id()
            link_cnt_info = self.cnt_info_in_sam[link_id]

            if link_cnt_info is None:
                self.cnt_info_in_sam[link_id] = replace(state_cnt_info)
                continue

            link_cnt_info.str_cnt += state_cnt_info.str_cnt
            link_cnt_info.tot_cnt_lower = min(
                link_cnt_info.tot_cnt_lower,
                state_cnt_info.tot_cnt_lower,
            )
            link_cnt_info.tot_cnt_upper = max(
                link_cnt_info.tot_cnt_upper,
                state_cnt_info.tot_cnt_upper,
            )

        for state_id in range(self.sam_rev.num_of_nodes()):
            sam_state = self.sam_rev.get_state(state_id)
            state_cnt_info = self.cnt_info_in_sam[state_id]
            if sam_state.is_nil() or sam_state.is_root() or state_cnt_info is None:
                continue

            link_id = sam_state.get_suffix_parent_id()
            link_cnt_info = self.cnt_info_in_sam[link_id]

            assert link_cnt_info is not None
            assert link_cnt_info.tot_cnt_lower <= state_cnt_info.tot_cnt_lower
            assert link_cnt_info.tot_cnt_upper >= state_cnt_info.tot_cnt_upper

    def get_root_state(self) -> GeneralSAMState:
        return self.sam_rev.get_root_state()

    def prepend_feed(
        self, state: GeneralSAMState, token: Union[str, bytes]
    ) -> Optional[CountInfo]:
        if self.bytes_or_chars == VocabPrefixBytesOrChars.BYTES and isinstance(
            token, str
        ):
            token = token.encode()
        self._state_feed_fn(state, token[::-1])
        return self.cnt_info_in_sam[state.get_node_id()]

    def get_order(self) -> Sequence[int]:
        return self.vocab_sort_res.order

    def get_order_slice(self, cnt_info: CountInfo) -> Sequence[int]:
        return self.vocab_sort_res.order[
            cnt_info.tot_cnt_lower : cnt_info.tot_cnt_upper
        ]
