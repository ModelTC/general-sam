from general_sam import VocabPrefixAutomaton, CountInfo


def test_chinese_chars_vocab_prefix():
    vocab = ['歌曲', '聆听歌曲', '播放歌曲', '歌词', '查看歌词']
    automaton = VocabPrefixAutomaton(vocab, bytes_or_chars='chars')

    # NOTE: CountInfo is related to the sorted vocab:
    _ = ['播放歌曲', '查看歌词', '歌曲', '歌词', '聆听歌曲']

    # 一起 | 聆 | 听 | 歌
    state = automaton.get_root_state()

    # feed 歌
    cnt_info = automaton.prepend_feed(state, '歌')
    assert cnt_info is not None and cnt_info == CountInfo(
        str_cnt=2, tot_cnt_lower=2, tot_cnt_upper=4
    )

    selected_idx = automaton.get_order_slice(cnt_info)
    assert frozenset(selected_idx) == {0, 3}
    selected_vocab = [vocab[i] for i in selected_idx]
    assert frozenset(selected_vocab) == {'歌曲', '歌词'}

    # feed 听
    cnt_info = automaton.prepend_feed(state, '听')
    assert cnt_info is None
    assert not state.is_nil()

    # feed 聆
    cnt_info = automaton.prepend_feed(state, '聆')
    assert cnt_info is not None and cnt_info == CountInfo(
        str_cnt=1, tot_cnt_lower=4, tot_cnt_upper=5
    )

    selected_idx = automaton.get_order_slice(cnt_info)
    assert frozenset(selected_idx) == {1}
    selected_vocab = [vocab[i] for i in selected_idx]
    assert frozenset(selected_vocab) == {'聆听歌曲'}

    # feed 一起
    assert not state.is_nil()
    cnt_info = automaton.prepend_feed(state, '一起')
    assert state.is_nil()

    # 来 | 查看 | 歌词
    state = automaton.get_root_state()

    # feed 歌词
    cnt_info = automaton.prepend_feed(state, '歌词')
    assert cnt_info is not None and cnt_info == CountInfo(
        str_cnt=1, tot_cnt_lower=3, tot_cnt_upper=4
    )

    selected_idx = automaton.get_order_slice(cnt_info)
    assert frozenset(selected_idx) == {3}
    selected_vocab = [vocab[i] for i in selected_idx]
    assert frozenset(selected_vocab) == {'歌词'}

    # feed 查看
    cnt_info = automaton.prepend_feed(state, '查看')
    assert cnt_info is not None and cnt_info == CountInfo(
        str_cnt=1, tot_cnt_lower=1, tot_cnt_upper=2
    )

    selected_idx = automaton.get_order_slice(cnt_info)
    assert frozenset(selected_idx) == {4}
    selected_vocab = [vocab[i] for i in selected_idx]
    assert frozenset(selected_vocab) == {'查看歌词'}

    # feed 来
    assert not state.is_nil()
    cnt_info = automaton.prepend_feed(state, '来')
    assert state.is_nil()
