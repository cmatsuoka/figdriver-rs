use std::ffi::OsString;

pub struct Args {
    args: Vec<OsString>,
    inner: Option<pico_args::Arguments>,
}

impl Args {
    pub fn from_env() -> Self {
        let mut args: Vec<OsString> = std::env::args_os().collect();
        if !args.is_empty() {
            let _progname = args.remove(0);
        }
        Self {
            args: args.clone(),
            inner: Some(pico_args::Arguments::from_vec(args)),
        }
    }

    /// Returns the value whose alias appeared last in the original args.
    /// Handles combined short flags (e.g., `-cp`) and consumes matched flags.
    pub fn last_of<'a, V: Clone>(&mut self, groups: &[(V, &[&'a str])]) -> Option<V>
    where
        'a: 'static,
    {
        let mut result: Option<V> = None;
        for arg in &self.args {
            let s = arg.to_string_lossy();
            if s == "--" {
                break;
            }
            if !s.starts_with('-') {
                continue;
            }
            if s == "-" {
                continue;
            }
            if s.starts_with("--") {
                let key = s.split('=').next().unwrap_or("");
                for (value, aliases) in groups {
                    if aliases.contains(&key) {
                        result = Some(value.clone());
                    }
                }
            } else {
                for c in s.chars().skip(1) {
                    let flag = format!("-{}", c);
                    for (value, aliases) in groups {
                   if aliases.iter().any(|a| flag == *a) {
                        result = Some(value.clone());
                    }
                    }
                }
            }
        }
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

    /// Collect all values for a repeated flag.
    pub fn collect_values(&mut self, keys: impl Into<pico_args::Keys> + Clone) -> Vec<String> {
        let mut values = Vec::new();
        while let Ok(Some(val)) = self.inner.as_mut().unwrap().opt_value_from_str::<_, String>(keys.clone().into()) {
            values.push(val);
        }
        values
    }

    /// Returns the index in the raw args vector of the last occurrence of a given alias.
    pub fn last_index_of(&self, aliases: &[&str]) -> Option<usize> {
        let mut result: Option<usize> = None;
        for (i, arg) in self.args.iter().enumerate() {
            let s = arg.to_string_lossy();
            if s == "--" {
                break;
            }
            if !s.starts_with('-') {
                continue;
            }
            if s.starts_with("--") {
                let key = s.split('=').next().unwrap_or("");
                if aliases.contains(&key) {
                    result = Some(i);
                }
            } else {
                for c in s.chars().skip(1) {
                    let flag = format!("-{}", c);
                    if aliases.iter().any(|a| flag == *a) {
                        result = Some(i);
                    }
                }
            }
        }
        result
    }

 }
