use std::ffi::OsString;

pub struct Args {
    args: Vec<OsString>,
    inner: Option<pico_args::Arguments>,
}

impl Args {
    pub fn from_env() -> Self {
        let mut args: Vec<OsString> = std::env::args_os().collect();
        let _progname = args.remove(0);
        Self {
            args: args.clone(),
            inner: Some(pico_args::Arguments::from_vec(args)),
        }
    }

    /// Returns the value whose alias appeared last in the original args.
    /// Only matches flag-like arguments (starting with `-`).
    pub fn last_of<'a, V: Clone>(&mut self, groups: &[(V, &[&'a str])]) -> Option<V>
    where
        'a: 'static,
    {
        let mut result: Option<V> = None;
        for arg in &self.args {
            let s = arg.to_string_lossy();
            if !s.starts_with('-') {
                continue;
            }
            for (value, aliases) in groups {
                if aliases.iter().any(|a| s == *a) {
                    result = Some(value.clone());
                }
            }
        }
        // Consume all occurrences of each alias from pico_args
        for (_, aliases) in groups {
            for alias in *aliases {
                while self.inner.as_mut().unwrap().contains(*alias) {}
            }
        }
        result
    }

    pub fn contains(&mut self, keys: impl Into<pico_args::Keys>) -> bool {
        self.inner.as_mut().unwrap().contains(keys)
    }

    pub fn opt_value_from_str<T>(&mut self, keys: impl Into<pico_args::Keys>)
        -> Result<Option<T>, pico_args::Error>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        self.inner.as_mut().unwrap().opt_value_from_str(keys)
    }

    pub fn finish(&mut self) -> Vec<OsString> {
        self.inner.take().unwrap().finish()
    }
}
