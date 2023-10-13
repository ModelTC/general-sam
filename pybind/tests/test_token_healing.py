from typing import Collection, Iterable, Optional, Sequence, Union

from general_sam import (
    CountInfo,
    GeneralSAMState,
    VocabPrefixAutomaton,
    VocabPrefixBytesOrChars,
)


def _test_token_healing_batch(
    vocab: Collection[Union[str, bytes]],
    token_sequences: Iterable[Union[Sequence[str], Sequence[bytes]]],
    bytes_or_chars: VocabPrefixBytesOrChars,
):
    automaton = VocabPrefixAutomaton(vocab, bytes_or_chars=bytes_or_chars)

    vocab_sorted = sorted(vocab)

    def validate(
        query: Union[str, bytes], state: GeneralSAMState, cnt_info: Optional[CountInfo]
    ):
        import bisect

        expected_l = bisect.bisect_left(
            vocab_sorted, query, key=lambda x: x[: len(query)]
        )
        expected_r = bisect.bisect_right(
            vocab_sorted, query, key=lambda x: x[: len(query)]
        )

        if expected_l < expected_r:
            expected_cnt_info = CountInfo(
                str_cnt=expected_r - expected_l,
                tot_cnt_lower=expected_l,
                tot_cnt_upper=expected_r,
            )
        else:
            expected_cnt_info = None

        assert cnt_info == expected_cnt_info, (query, cnt_info, expected_cnt_info)

        assert state.is_nil() ^ any(query in i for i in vocab)  # pyright: ignore

    def check(tokens: Sequence[Union[str, bytes]]):
        state = automaton.get_root_state()
        query = '' if isinstance(tokens[0], str) else b''

        # NOTE: tokens are prepended in the reverse order
        for token in reversed(tokens):
            query = token + query  # pyright: ignore
            cnt_info = automaton.prepend_feed(state, token)
            validate(query, state, cnt_info)

    for tokens in token_sequences:
        check(tokens)


def _test_batch(
    vocab: Collection[str],
    token_sequences: Iterable[Union[Sequence[str], Sequence[bytes]]],
):
    _test_token_healing_batch(
        vocab,
        tuple(filter(lambda x: isinstance(x[0], str), token_sequences)),
        VocabPrefixBytesOrChars.CHARS,
    )
    _test_token_healing_batch(
        tuple(i.encode() for i in vocab),
        tuple(
            tuple(i.encode() if isinstance(i, str) else i for i in s)
            for s in token_sequences
        ),
        VocabPrefixBytesOrChars.BYTES,
    )


def test_simple_token_healing():
    _test_batch(
        ['bb', 'ca', 'ab', 'c', 'aa', 'bbaa', 'a', 'cc', 'b'],
        [
            ['bb', 'a'],
            ['b', 'b', 'b'],
            ['b', 'b', 'a'],
            ['b', 'ba'],
            ['ca', 'c', 'ab'],
            ['c', 'c', 'c'],
        ],
    )


def test_simple_chinese_token_healing():
    _test_batch(
        ['歌曲', '聆听歌曲', '播放歌曲', '歌词', '查看歌词'],
        [
            ['歌曲'],
            ['聆听歌曲'],
            ['聆听', '歌曲'],
            ['聆', '听', '歌曲'],
            ['播放歌曲'],
            ['播', '放歌曲'],
            ['播放', '歌曲'],
            ['歌词'],
            ['查看歌词'],
            ['查看', '歌词'],
            ['听歌曲'],
            ['听', '歌曲'],
            ['放歌曲'],
            ['听歌'],
            ['放歌'],
            ['词'],
            ['查看'],
            ['bb', 'a'],
            ['b', 'b', 'b'],
            ['b', 'b', 'a'],
            ['b', 'ba'],
            ['ca', 'c', 'ab'],
            ['c', 'c', 'c'],
        ],
    )


def test_simple_utf8_token_healing():
    # '䨻'.encode('utf8') == b'\xe4\xa8\xbb'
    _test_batch(
        ['䨻'],
        [
            ['䨻'],
            [b'\xe4', b'\xa8', b'\xbb'],
            [b'\xe4', b'\xa8\xbb'],
            [b'\xe4\xa8', b'\xbb'],
            [b'\xe4\xa8\xbb'],
        ],
    )
