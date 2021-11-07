#![cfg(test)]

use proc_macro2::{TokenStream, TokenTree};

mod full;
mod process_field;

fn token_stream_equals(ts1: TokenStream, ts2: TokenStream) -> bool {
    let mut ts1 = ts1.into_iter().fuse();
    let mut ts2 = ts2.into_iter().fuse();

    loop {
        match (ts1.next(), ts2.next()) {
            (Some(tt1), Some(tt2)) => match (tt1, tt2) {
                (TokenTree::Ident(i1), TokenTree::Ident(i2)) => {
                    if i1 != i2 {
                        return false;
                    }
                }
                (TokenTree::Punct(p1), TokenTree::Punct(p2)) => {
                    if p1.as_char() != p2.as_char() {
                        return false;
                    }
                }
                (TokenTree::Literal(l1), TokenTree::Literal(l2)) => {
                    if l1.to_string() != l2.to_string() {
                        return false;
                    }
                }
                (TokenTree::Group(g1), TokenTree::Group(g2)) => {
                    if !token_stream_equals(g1.stream(), g2.stream()) {
                        return false;
                    }
                }
                _ => return false,
            },
            (None, None) => return true,
            (Some(tt1), None) => {
                if let TokenTree::Punct(p1) = tt1 {
                    if p1.as_char() == ',' && ts1.next().is_none() {
                        return true;
                    }
                }
                return false;
            }
            (None, Some(tt2)) => {
                if let TokenTree::Punct(p2) = tt2 {
                    if p2.as_char() == ',' && ts2.next().is_none() {
                        return true;
                    }
                }
                return false;
            }
        }
    }
}
