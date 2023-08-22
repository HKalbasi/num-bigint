//! Test serialization and deserialization of `BigUint` and `BigInt`
//!
//! The serialized formats should not change, even if we change our
//! internal representation, because we want to preserve forward and
//! backward compatibility of serialized data!
//!
//! This test is in a completely separate crate so its `serde_test`
//! dependency does not "infect" the rest of the build with `serde`'s
//! default features, especially not `serde/std`.

#![cfg(test)]

use num_bigint::{BigInt, BigUint};
use num_traits::{One, Zero};
use serde::{de::DeserializeOwned, Serialize};
use serde_test::{assert_de_tokens, assert_ser_tokens, assert_tokens, Token};
use std::{fmt::Debug, panic::catch_unwind};

#[test]
fn biguint_zero() {
    let tokens = [Token::Seq { len: Some(0) }, Token::SeqEnd];
    assert_tokens(&BigUint::zero(), &tokens);
}

#[test]
fn bigint_zero() {
    let tokens = [
        Token::Tuple { len: 2 },
        Token::I8(0),
        Token::Seq { len: Some(0) },
        Token::SeqEnd,
        Token::TupleEnd,
    ];
    assert_tokens(&BigInt::zero(), &tokens);
}

#[test]
fn biguint_one() {
    let tokens = [Token::Seq { len: Some(1) }, Token::U32(1), Token::SeqEnd];
    assert_tokens(&BigUint::one(), &tokens);
}

#[test]
fn bigint_one() {
    let tokens = [
        Token::Tuple { len: 2 },
        Token::I8(1),
        Token::Seq { len: Some(1) },
        Token::U32(1),
        Token::SeqEnd,
        Token::TupleEnd,
    ];
    assert_tokens(&BigInt::one(), &tokens);
}

#[test]
fn bigint_negone() {
    let tokens = [
        Token::Tuple { len: 2 },
        Token::I8(-1),
        Token::Seq { len: Some(1) },
        Token::U32(1),
        Token::SeqEnd,
        Token::TupleEnd,
    ];
    assert_tokens(&-BigInt::one(), &tokens);
}

// Generated independently from python `hex(factorial(100))`
const FACTORIAL_100: &[u32] = &[
    0x00000000, 0x00000000, 0x00000000, 0x2735c61a, 0xee8b02ea, 0xb3b72ed2, 0x9420c6ec, 0x45570cca,
    0xdf103917, 0x943a321c, 0xeb21b5b2, 0x66ef9a70, 0xa40d16e9, 0x28d54bbd, 0xdc240695, 0x964ec395,
    0x1b30,
];

#[test]
fn biguint_factorial_100() {
    let n: BigUint = (1u8..101).product();

    let mut tokens = vec![];
    tokens.push(Token::Seq {
        len: Some(FACTORIAL_100.len()),
    });
    tokens.extend(FACTORIAL_100.iter().map(|&u| Token::U32(u)));
    tokens.push(Token::SeqEnd);

    assert_tokens(&n, &tokens);
}

#[test]
fn bigint_factorial_100() {
    let n: BigInt = (1i8..101).product();

    let mut tokens = vec![];
    tokens.push(Token::Tuple { len: 2 });
    tokens.push(Token::I8(1));
    tokens.push(Token::Seq {
        len: Some(FACTORIAL_100.len()),
    });
    tokens.extend(FACTORIAL_100.iter().map(|&u| Token::U32(u)));
    tokens.push(Token::SeqEnd);
    tokens.push(Token::TupleEnd);

    assert_tokens(&n, &tokens);
}

#[test]
fn big_digits() {
    // Try a few different lengths for u32/u64 digit coverage
    for len in 1..10 {
        let digits = 1u32..=len;
        let n = BigUint::new(digits.clone().collect());

        let mut tokens = vec![];
        tokens.push(Token::Seq {
            len: Some(len as usize),
        });
        tokens.extend(digits.map(Token::U32));
        tokens.push(Token::SeqEnd);

        assert_tokens(&n, &tokens);

        let n = BigInt::from(n);
        tokens.insert(0, Token::Tuple { len: 2 });
        tokens.insert(1, Token::I8(1));
        tokens.push(Token::TupleEnd);
        assert_tokens(&n, &tokens);

        tokens[1] = Token::I8(-1);
        assert_tokens(&-n, &tokens);
    }
}

#[test]
fn bad_size_hint_int() {
    bad_size_hint::<BigInt>(&[Token::Tuple { len: 2 }, Token::I8(1)], &[Token::TupleEnd]);
}

#[test]
fn bad_size_hint_uint() {
    bad_size_hint::<BigUint>(&[], &[]);
}

fn bad_size_hint<T: Debug + DeserializeOwned + One + PartialEq + Serialize>(
    prefix: &[Token],
    suffix: &[Token],
) {
    let mut tokens = [
        prefix,
        &[Token::Seq { len: Some(1) }, Token::U32(1), Token::SeqEnd],
        suffix,
    ]
    .concat();

    assert_tokens(&T::one(), &tokens);

    tokens[prefix.len()] = Token::Seq {
        len: Some(usize::max_value()),
    };

    catch_unwind(|| assert_ser_tokens(&T::one(), &tokens)).unwrap_err();
    assert_de_tokens(&T::one(), &tokens);
}
