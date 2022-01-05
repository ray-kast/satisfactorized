mod format {
    use std::fmt;

    use lazy_static::lazy_static;
    use regex::Regex;
    use serde::{de, Deserialize};

    use crate::{
        items::{Amount, ItemStack, ItemStacks},
        prelude::*,
        HashMap,
    };

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Config {
        pub raw: Vec<String>,
        pub recipes: HashMap<String, Vec<Recipe>>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Recipe {
        pub make: Items,
        pub from: Items,
        #[serde(rename = "in")]
        pub in_sec: Amount,
        pub name: Option<String>,
    }

    pub struct Items(pub Vec<Item>);
    pub struct Item(pub Amount, pub String);

    impl TryFrom<Items> for ItemStacks {
        type Error = Error;

        fn try_from(items: Items) -> Result<Self> {
            items
                .0
                .into_iter()
                .map(|Item(amt, name)| name.try_into().map(|n| (n, amt)))
                .collect::<Result<HashMap<_, _>, _>>()
                .map(Self::new)
                .map_err(Into::into)
        }
    }

    impl TryFrom<Item> for ItemStack {
        type Error = Error;

        fn try_from(item: Item) -> Result<Self> { Ok(Self(item.1.try_into()?, item.0)) }
    }

    impl<'de> Deserialize<'de> for Items {
        fn deserialize<D: de::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
            struct Visitor;

            impl<'de> de::Visitor<'de> for Visitor {
                type Value = Items;

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str(
                        r#"an item string, such as "1 iron_ingot", or array of item strings"#,
                    )
                }

                fn visit_str<E: de::Error>(self, s: &str) -> Result<Items, E> {
                    s.parse().map_err(E::custom)
                }

                fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Items, A::Error> {
                    let mut vec = vec![];

                    while let Some(s) = seq.next_element::<&str>()? {
                        vec.push(s.parse().map_err(<A::Error as de::Error>::custom)?);
                    }

                    Ok(Items(vec))
                }
            }

            de.deserialize_any(Visitor)
        }
    }

    impl std::str::FromStr for Items {
        type Err = Error;

        fn from_str(s: &str) -> Result<Self> {
            lazy_static! {
                static ref ITEM_REGEX: Regex =
                    Regex::new(r"^\s*([\d.,]+)\s+(\S+)(?:\s+|$)").unwrap();
            }

            let mut offs = 0;
            let mut out = vec![];

            while let Some(caps) = ITEM_REGEX.captures(&s[offs..]) {
                out.push(Item(
                    caps[1]
                        .parse()
                        .with_context(|| anyhow!("Invalid amount for item string {:?}", s))?,
                    caps[2].to_owned(),
                ));
                offs += caps[0].len();
            }

            if offs < s.len() {
                bail!("Unexpected trailing string {:?}", &s[offs..]);
            }

            Ok(Self(out))
        }
    }

    impl std::str::FromStr for Item {
        type Err = Error;

        fn from_str(s: &str) -> Result<Self> {
            lazy_static! {
                static ref REGEX: Regex = Regex::new(r"^\s*([\d.,]+)\s+(\S+)\s*$").unwrap();
            }

            let caps = REGEX
                .captures(s)
                .ok_or_else(|| anyhow!("Invalid item string {:?}", s))?;

            Ok(Self(
                caps[1]
                    .parse()
                    .with_context(|| anyhow!("Invalid amount for item string {:?}", s))?,
                caps[2].to_owned(),
            ))
        }
    }
}

use std::{fs::File, path::Path};

use crate::{
    items::{Amount, Item, ItemStacks, Machine},
    prelude::*,
    HashSet,
};

#[derive(Debug)]
pub struct Config {
    raw: HashSet<Item>,
    recipes: Vec<Recipe>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Recipe {
    inputs: ItemStacks,
    outputs: ItemStacks,
    time: Amount,
    machine: Machine,
    name: Option<String>,
}

impl Config {
    pub fn load(from: impl AsRef<Path>) -> Result<Self> {
        let ret: format::Config =
            serde_yaml::from_reader(File::open(from).context("Failed to open config file")?)
                .context("Failed to parse config file")?;

        Item::registry_ref().register(ret.raw.iter().cloned().chain(
            ret.recipes.values().flat_map(|r| {
                r.iter()
                    .flat_map(|r| r.make.0.iter().map(|format::Item(_, n)| n.clone()))
            }),
        ));

        Machine::registry_ref().register(ret.recipes.keys().cloned());

        Ok(Config {
            raw: ret
                .raw
                .into_iter()
                .map(|s| Item::new(&s))
                .collect::<Result<_, _>>()
                .unwrap(),
            recipes: ret
                .recipes
                .into_iter()
                .flat_map(|(k, v)| {
                    v.into_iter()
                        .zip(std::iter::repeat(k))
                        .map(|(recipe, machine)| Ok(Recipe {
                            inputs: recipe.from.try_into()?,
                            outputs: recipe.make.try_into().unwrap(),
                            time: recipe.in_sec,
                            machine: machine.try_into().unwrap(),
                            name: recipe.name,
                        }))
                })
                .collect::<Result<_>>()?,
        })
    }
}
