use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use general_sam::{
    table::{BoxBisectTable, HashTransTable, VecBisectTable},
    tokenize::{trie::greedy_tokenize_with_trie, GreedyTokenizer},
    BTreeTransTable, GeneralSAM, TransitionTable, Trie,
};
use rand::{
    distributions::{Alphanumeric, DistString},
    rngs::StdRng,
    Rng, SeedableRng,
};
use tokenizers::{Model, Tokenizer as HFTokenizer};

type Vocab = HashMap<String, u32>;

fn get_rng(xor: u64) -> StdRng {
    let seed: u64 = std::env::var("SEED").map_or(391096, |x| x.parse().expect("invalid SEED"));
    StdRng::seed_from_u64(seed ^ xor)
}

fn gen_normal_vocab() -> Vocab {
    let vocab_size: usize =
        std::env::var("VOCAB_SIZE").map_or(64000, |x| x.parse().expect("invalid VOCAB_SIZE"));
    let max_token_len: usize =
        std::env::var("MAX_TOKEN_LEN").map_or(16, |x| x.parse().expect("invalid MAX_TOKEN_LEN"));

    let mut res = Vocab::new();

    let mut rng = get_rng(107834265081463);
    for _ in 0..vocab_size {
        let len = rng.gen_range(0..max_token_len);
        let token = Alphanumeric.sample_string(&mut rng, len);
        res.insert(token, rng.gen());
    }

    res
}

fn gen_bad_vocab() -> Vocab {
    let vocab_size: usize =
        std::env::var("VOCAB_SIZE").map_or(500, |x| x.parse().expect("invalid VOCAB_SIZE"));

    let mut res = Vocab::new();

    let mut rng = get_rng(107834265081463);
    for s in ["0", "1", "a"] {
        res.insert(s.to_owned(), rng.gen());
    }
    for i in 0..vocab_size {
        let token: Box<[&str]> = (0..(i / 2 + 1))
            .map(|_| ["01", "10"][i % 2])
            .chain([["a"], ["1a"]][i % 2])
            .collect();
        let token = token.join("");
        res.insert(token, rng.gen());
    }

    res
}

fn gen_normal_seq(vocab: &Vocab) -> String {
    let num_of_tokens: usize = std::env::var("SEQ_NUM_TOKENS")
        .map_or(100000, |x| x.parse().expect("invalid SEQ_NUM_TOKENS"));

    let tokens: Box<[&String]> = vocab.keys().collect();
    let mut rng = get_rng(9813467507349067);

    let chosen: Box<[&str]> = (0..num_of_tokens)
        .map(|_| tokens[rng.gen_range(0..tokens.len())].as_str())
        .collect();
    chosen.join("")
}

fn gen_bad_seq(vocab: &Vocab) -> String {
    let num_of_tokens: usize =
        std::env::var("SEQ_NUM_TOKENS").map_or(500, |x| x.parse().expect("invalid SEQ_NUM_TOKENS"));

    let tokens: Box<[&String]> = vocab.keys().collect();
    let mut rng = get_rng(9813467507349067);

    let chosen: Box<[&str]> = (0..num_of_tokens)
        .map(|_| {
            let t = tokens[rng.gen_range(0..tokens.len())].as_str();
            let (bound, _) = t.char_indices().last().unwrap();
            &t[0..bound]
        })
        .collect();
    chosen.join("")
}

fn get_gen_style() -> String {
    std::env::var("STYLE").unwrap_or("bad".to_owned())
}

fn gen_vocab() -> Vocab {
    match get_gen_style().as_ref() {
        "normal" => gen_normal_vocab(),
        "bad" => gen_bad_vocab(),
        style => panic!("unknown STYLE {}", style),
    }
}

fn gen_seq(vocab: &Vocab) -> String {
    match get_gen_style().as_ref() {
        "normal" => gen_normal_seq(vocab),
        "bad" => gen_bad_seq(vocab),
        style => panic!("unknown STYLE {}", style),
    }
}

fn load_hf_tokenizer() -> Option<HFTokenizer> {
    std::env::var_os("HF_TOKENIZER").map(|p| {
        println!("loading {:?}...", &p);
        HFTokenizer::from_file(p).expect("failed to load hf tokenizer")
    })
}

fn tokenize_with_hf(tokenizer: &HFTokenizer, seq: &str) -> Vec<u32> {
    tokenizer
        .get_model()
        .tokenize(seq)
        .unwrap()
        .iter()
        .map(|x| x.id)
        .collect()
}

fn tokenize_with_sam<T: TransitionTable<KeyType = char>>(
    tokenizer: &GreedyTokenizer<T, u32, &GeneralSAM<T>>,
    seq: &str,
) -> Vec<u32> {
    tokenizer
        .tokenize(seq.chars(), &0)
        .iter()
        .map(|x| x.0)
        .collect()
}

fn tokenize_with_trie<T: TransitionTable<KeyType = char>>(
    trie: &Trie<T>,
    trie_to_token: &[u32],
    seq: &str,
) -> Vec<u32> {
    greedy_tokenize_with_trie(trie, seq.chars())
        .iter()
        .map(|x| trie_to_token[x.0])
        .collect()
}

fn build_trie<T: TransitionTable<KeyType = char>>(vocab: &Vocab) -> (Trie<T>, Vec<u32>) {
    let mut trie = Trie::<BTreeTransTable<_>>::default();
    let mut trie_id_and_token_id = Vec::new();
    for (k, v) in vocab.iter() {
        let node_id = trie.insert_iter(k.chars());
        trie_id_and_token_id.push((node_id, *v));
    }
    let mut trie_to_token = vec![0; trie.num_of_nodes()];
    for (u, v) in trie_id_and_token_id.iter() {
        trie_to_token[*u] = *v;
    }
    (trie.alter_trans_table(), trie_to_token)
}

fn criterion_benchmark<TransTable: TransitionTable<KeyType = char>>(c: &mut Criterion) {
    println!("{}", std::any::type_name::<TransTable>());

    println!("building hf_tokenizer...");
    let hf_tokenizer = load_hf_tokenizer();

    println!("building vocab & seq...");
    let vocab = hf_tokenizer
        .as_ref()
        .map_or_else(gen_vocab, |t| t.get_model().get_vocab());
    let seq = gen_seq(&vocab);

    println!("building trie...");
    let (trie, trie_to_token) = build_trie::<TransTable>(&vocab);
    println!("building sam...");
    let sam = GeneralSAM::<BTreeTransTable<_>>::from_trie(trie.get_root_state())
        .alter_trans_table_into::<TransTable>();
    println!("building greedy tokenizer...");
    let tokenizer =
        GreedyTokenizer::build(&sam, trie.get_root_state(), |tn| trie_to_token[tn.node_id]);

    println!("running benchmarks...");
    c.bench_function("GreedyTokenizer", |b| {
        b.iter(|| tokenize_with_sam(black_box(&tokenizer), black_box(seq.as_str())))
    });

    if let Some(ref t) = hf_tokenizer {
        c.bench_function("HFTokenizer", |b| {
            b.iter(|| tokenize_with_hf(black_box(t), black_box(seq.as_str())))
        });
    }

    c.bench_function("Trie", |b| {
        b.iter(|| {
            tokenize_with_trie(
                black_box(&trie),
                black_box(&trie_to_token),
                black_box(seq.as_str()),
            )
        })
    });
}

criterion_group!(
    benches,
    criterion_benchmark<BTreeTransTable<_>>,
    criterion_benchmark<HashTransTable<_>>,
    criterion_benchmark<VecBisectTable<_>>,
    criterion_benchmark<BoxBisectTable<_>>,
);
criterion_main!(benches);
