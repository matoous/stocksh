use serde::Deserializer;
use serde::de;
use serde::de::Visitor;
use std::fmt;
use std::fmt::Display;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::str::FromStr;

static PLAIN_TEXT_AGENTS: &'static [&str] = &[
    "curl",
    "httpie",
    "lwp-request",
    "wget",
    "python-requests",
    "openbsd ftp",
    "powershell",
    "fetch",
    "aiohttp",
];

pub fn is_plaintext_agent(agent: &str) -> bool {
    return PLAIN_TEXT_AGENTS.iter().any(
        |s| agent.to_lowercase().contains(s)
    );
}


pub fn comma_separated<'de, V, T, D>(deserializer: D) -> Result<V, D::Error>
where
    V: FromIterator<T>,
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    struct CommaSeparated<V, T>(PhantomData<V>, PhantomData<T>);

    impl<'de, V, T> Visitor<'de> for CommaSeparated<V, T>
    where
        V: FromIterator<T>,
        T: FromStr,
        T::Err: Display,
    {
        type Value = V;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string containing comma-separated elements")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let iter = s.split(",").map(FromStr::from_str);
            Result::from_iter(iter).map_err(de::Error::custom)
        }
    }

    let visitor = CommaSeparated(PhantomData, PhantomData);
    deserializer.deserialize_str(visitor)
}

