use std::{collections::HashMap, num::NonZero, path::PathBuf};

use anyhow::Context;
use serde::Deserialize;

#[derive(Debug)]
pub struct Grid([char; 9]);

impl<'de> Deserialize<'de> for Grid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut chars = s.trim().chars().filter(|c| !c.is_whitespace());
        let out = std::array::from_fn(|_| chars.next().unwrap_or('_'));
        if chars.next().is_some() {
            return Err(serde::de::Error::custom(
                "Grid must only contain 9 non-whitespace characters",
            ));
        }
        Ok(Self(out))
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    pub fn get(&self, index: usize) -> Option<&T> {
        match self {
            OneOrMany::One(t) if index == 0 => Some(t),
            OneOrMany::One(_) => None,
            OneOrMany::Many(items) => items.get(index),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            OneOrMany::One(_) => 1,
            OneOrMany::Many(items) => items.len(),
        }
    }
}

impl<T> IntoIterator for OneOrMany<T> {
    type Item = T;

    type IntoIter = Either<std::iter::Once<T>, std::vec::IntoIter<T>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            OneOrMany::One(t) => Either::A(std::iter::once(t)),
            OneOrMany::Many(items) => Either::B(items.into_iter()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Either<A, B> {
    A(A),
    B(B),
}

impl<A, B> Iterator for Either<A, B>
where
    A: Iterator,
    B: Iterator<Item = <A as Iterator>::Item>,
{
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::A(a) => a.next(),
            Either::B(b) => b.next(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyItemMethod {
    Frame,
    Random,
    Cycle,
    CycleSlow,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum KeyItem {
    Fixed(String),
    List {
        method: KeyItemMethod,
        items: Vec<String>,
    },
}

impl KeyItem {
    fn get_item(&self, index: usize, count: usize, i: usize) -> Option<&str> {
        match self {
            KeyItem::Fixed(s) => Some(s),
            KeyItem::List { method, items } => items
                .get(match method {
                    KeyItemMethod::Frame => index % items.len(),
                    KeyItemMethod::Random => rand::random_range(0..items.len()),
                    KeyItemMethod::Cycle => (index * count + i) % items.len(),
                    KeyItemMethod::CycleSlow => (index + i) % items.len(),
                })
                .map(String::as_str),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub grid: Grid,
    pub result: OneOrMany<String>,
    pub key: HashMap<char, KeyItem>,
    pub frames: NonZero<u32>,
    pub frame_duration: u16,
}

impl Config {
    pub fn recipe(&self, index: usize) -> anyhow::Result<([Option<PathBuf>; 9], PathBuf)> {
        let mut out = [const { None }; 9];

        for (i, &c) in self.grid.0.iter().enumerate() {
            if c == '_' {
                continue;
            }

            let count = self.grid.0.iter().filter(|n| **n == c).count();

            let textures = self
                .key
                .get(&c)
                .with_context(|| format!("Unknown recipe item '{}'", c))?;
            let texture = textures
                .get_item(index, count, i)
                .with_context(|| format!("No textures declared for item: '{}'", c))?;

            let mut path = PathBuf::from_iter(["textures", texture]);
            path.set_extension("png");
            out[i] = Some(path);
        }

        let result = self
            .result
            .get(index % self.result.len())
            .context("No results declared for recipe")?;
        let mut result = PathBuf::from_iter(["textures", result]);
        result.set_extension("png");

        Ok((out, result))
    }
}
