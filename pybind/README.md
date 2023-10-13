# general-sam-py

![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-informational?style=flat-square)

Python bindings for [`general-sam`](https://github.com/ModelTC/general-sam)
and some utilities.

|         [![the suffix automaton of abcbc][sam-of-abcbc]][sam-oi-wiki]          |
| :----------------------------------------------------------------------------: |
| The suffix automaton of abcbc, image from [后缀自动机 - OI Wiki][sam-oi-wiki]. |

[sam-of-abcbc]: https://oi-wiki.org/string/images/SAM/SA_suffix_links.svg
[sam-oi-wiki]: https://oi-wiki.org/string/sam/

## Usage

### `GeneralSAM`

```python
from general_sam import GeneralSAM


sam = GeneralSAM.construct_from_bytes(b'abcbc')

state = sam.get_root_state()
state.feed_bytes(b'cbc')
assert state.is_accepting()

state = sam.get_root_state()
state.feed_bytes(b'bcb')
assert not state.is_accepting()
```

```python
from general_sam import GeneralSAM


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
```

```python
from general_sam import GeneralSAM, GeneralSAMState, construct_trie_from_chars


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
```

### `VocabPrefixAutomaton`

```python
from general_sam import VocabPrefixAutomaton, CountInfo


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
```

## License

- &copy; 2023 Chielo Newctle \<ChieloNewctle@gmail.com\>
- &copy; 2023 ModelTC Team

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) ([`LICENSE-APACHE`](LICENSE-APACHE))
- [MIT license](https://opensource.org/licenses/MIT) ([`LICENSE-MIT`](LICENSE-MIT))

at your option.

The [SPDX](https://spdx.dev) license identifier for this project is `MIT OR Apache-2.0`.
