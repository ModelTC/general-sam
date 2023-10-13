from general_sam import GeneralSAM, GeneralSAMState, construct_trie_from_chars


def test_bytes_abcbc():
    sam = GeneralSAM.construct_from_bytes(b'abcbc')

    state = sam.get_root_state()
    state.feed_bytes(b'cbc')
    assert state.is_accepting()

    state = sam.get_root_state()
    state.feed_bytes(b'bcb')
    assert not state.is_accepting()


def test_chars_abcbc():
    sam = GeneralSAM.construct_from_chars('abcbc')
    state = sam.get_root_state()

    state.feed_chars('b')
    assert not state.is_accepting()
    state.feed_chars('c')
    assert state.is_accepting()
    state.feed_chars('bc')
    assert state.is_accepting()
    state.feed_chars('bc')
    assert not state.is_accepting() and state.is_nil()


def test_simple_sam_from_trie():
    trie, _ = construct_trie_from_chars(['hello', 'Chielo'])
    sam = GeneralSAM.construct_from_trie(trie)

    def fetch_state(s: str) -> GeneralSAMState:
        state = sam.get_root_state()
        state.feed_chars(s)
        return state

    assert fetch_state('lo').is_accepting()
    assert fetch_state('ello').is_accepting()
    assert fetch_state('elo').is_accepting()

    state = fetch_state('el')
    assert not state.is_accepting() and not state.is_nil()

    state = fetch_state('bye')
    assert not state.is_accepting() and state.is_nil()
