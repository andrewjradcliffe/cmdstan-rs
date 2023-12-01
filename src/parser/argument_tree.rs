use crate::argument_tree::*;
use crate::method::Method;
use crate::parser::*;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

impl Output {
    fn try_from_pair(pair: Pair<'_, Rule>) -> Result<Self, ParseGrammarError> {
        match pair.as_rule() {
            Rule::output => {
                let pairs = pair.into_inner();
                // We unify terms, storing state in a builder
                let mut builder = Self::builder();
                for pair in pairs {
                    match pair.as_rule() {
                        Rule::file => path_arm!(builder, pair, file),
                        Rule::diagnostic_file => path_arm!(builder, pair, diagnostic_file),
                        Rule::profile_file => path_arm!(builder, pair, profile_file),
                        Rule::sig_figs => number_arm!(builder, pair, sig_figs, i32),
                        Rule::refresh => number_arm!(builder, pair, refresh, i32),
                        _ => unreachable!(),
                    }
                }
                Ok(builder.build())
            }
            r => Err(RuleError(format!("Cannot construct from rule: {r:?}"))),
        }
    }
}

impl FromStr for Output {
    type Err = ParseGrammarError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match GrammarParser::parse(Rule::output_as_type, s) {
            Ok(mut pairs) => {
                let pair = pairs.next().unwrap().into_inner().next().unwrap();
                Self::try_from_pair(pair)
            }
            Err(e) => Err(OutputError(format!("{e:#?}"))),
        }
    }
}

impl Random {
    fn try_from_pair(pair: Pair<'_, Rule>) -> Result<Self, ParseGrammarError> {
        match pair.as_rule() {
            Rule::random => {
                let pairs = pair.into_inner();
                // We can simplify due to the grammar structure.
                let mut seed: Option<i64> = None;
                for pair in pairs {
                    match pair.as_str().parse::<i64>() {
                        Ok(value) => {
                            seed = Some(value);
                        }
                        Err(e) => return Err(RandomError(format!("{e:#?}"))),
                    }
                }
                let x = match seed {
                    Some(seed) => Random { seed },
                    _ => Random::default(),
                };
                Ok(x)
            }
            r => Err(RuleError(format!("Cannot construct from rule: {r:?}"))),
        }
    }
}

impl FromStr for Random {
    type Err = ParseGrammarError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match GrammarParser::parse(Rule::random_as_type, s) {
            Ok(mut pairs) => {
                let pair = pairs.next().unwrap().into_inner().next().unwrap();
                Self::try_from_pair(pair)
            }
            Err(e) => Err(RandomError(format!("{e:#?}"))),
        }
    }
}

impl Data {
    fn try_from_pair(pair: Pair<'_, Rule>) -> Result<Self, ParseGrammarError> {
        match pair.as_rule() {
            Rule::data => {
                let pairs = pair
                    .into_inner()
                    .filter_map(|file| file.into_inner().next());
                // We can simplify due to the grammar structure.
                let x = match pairs.last().map(|pair| OsString::from(pair.as_str())) {
                    Some(file) => Data { file },
                    _ => Data::default(),
                };
                Ok(x)
            }
            r => Err(RuleError(format!("Cannot construct from rule: {r:?}"))),
        }
    }
}

impl FromStr for Data {
    type Err = ParseGrammarError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match GrammarParser::parse(Rule::data_as_type, s) {
            Ok(mut pairs) => {
                let pair = pairs.next().unwrap().into_inner().next().unwrap();
                Self::try_from_pair(pair)
            }
            Err(e) => Err(DataError(format!("{e:#?}"))),
        }
    }
}

macro_rules! once_branch {
    ($B:ident, $P:ident, $state:ident, $T:ident, $F:ident) => {
        if $state {
            return Err(ArgumentTreeError(format!(
                "{} declared more than once",
                stringify!($F)
            )));
        } else {
            $B = $B.$F($T::try_from_pair($P)?);
            $state = true;
        }
    };
}

macro_rules! once_branch_parse_i32 {
    ($B:ident, $P:ident, $state:ident, $F:ident) => {
        if $state {
            return Err(ArgumentTreeError(format!(
                "{} declared more than once",
                stringify!($F)
            )));
        } else {
            match $P.into_inner().next() {
                Some(pair) => match pair.as_str().parse::<i32>() {
                    Ok(value) => {
                        $B = $B.$F(value);
                    }
                    Err(e) => return Err(ArgumentTreeError(format!("{e:#?}"))),
                },
                _ => (),
            }
            $state = true;
        }
    };
}

impl ArgumentTree {
    fn try_from_pair(pair: Pair<'_, Rule>) -> Result<Self, ParseGrammarError> {
        match pair.as_rule() {
            Rule::argument_tree => {
                let pairs = pair.into_inner();
                // To implement the unification, and enforcerules, we must keep count
                // of the declarations. Since only a single declaration is permitted,
                // we can use a binary variable.
                let mut st_method = false;
                let mut st_init = false;
                let mut st_data = false;
                let mut st_random = false;
                let mut st_output = false;
                let mut st_id = false;
                let mut st_num_threads = false;

                let mut builder = ArgumentTree::builder();
                for pair in pairs {
                    match pair.as_rule() {
                        Rule::method_special_case => {
                            once_branch!(builder, pair, st_method, Method, method);
                        }
                        Rule::init => {
                            if st_init {
                                return Err(ArgumentTreeError(
                                    "init declared more than once".into(),
                                ));
                            } else if let Some(pair) = pair.into_inner().next() {
                                builder = builder.init(pair.as_str());
                            }
                            st_init = true;
                        }
                        Rule::data => {
                            once_branch!(builder, pair, st_data, Data, data);
                        }
                        Rule::random => {
                            once_branch!(builder, pair, st_random, Random, random);
                        }
                        Rule::output => {
                            once_branch!(builder, pair, st_output, Output, output);
                        }
                        Rule::id => {
                            once_branch_parse_i32!(builder, pair, st_id, id);
                        }
                        Rule::num_threads => {
                            once_branch_parse_i32!(builder, pair, st_num_threads, num_threads);
                        }
                        _ => unreachable!(),
                    }
                }
                if st_method {
                    Ok(builder.build())
                } else {
                    Err(ArgumentTreeError("A method must be specified!".into()))
                }
            }
            r => Err(RuleError(format!("Cannot construct from rule: {r:?}"))),
        }
    }

    pub fn try_from_stan_csv<P: AsRef<Path>>(
        path: P,
    ) -> io::Result<Result<Self, ParseGrammarError>> {
        let file = File::open(path)?;
        Self::from_reader(file)
    }

    pub fn from_reader<R: Read>(rdr: R) -> io::Result<Result<Self, ParseGrammarError>> {
        fn remove_newline(s: &mut String) {
            if s.ends_with('\n') {
                s.pop();
                if s.ends_with('\r') {
                    s.pop();
                }
            }
        }
        fn consume(s: &mut String, line: &str) -> bool {
            let l = line
                .trim_start_matches('#')
                .trim_start()
                .trim_end_matches("(Default)");
            if let Some((prefix, suffix)) = l.split_once(" = ") {
                s.push_str(prefix);
                s.push('=');
                s.push_str(suffix);
                s.push(' ');
            } else if !s.trim().ends_with(l.trim_end()) {
                s.push_str(l);
                s.push(' ');
            }
            // Are we done?
            // The stop symbol is num_threads, at least under the current Stan format.
            l.starts_with("num_threads")
        }
        let mut file = BufReader::new(rdr);

        // For lines which do not contain values, 256 bytes should be sufficient
        // even for very long paths. Add 64 bytes for the long keywords.
        let mut l = String::with_capacity(320);
        // Worst case scenario: 5 paths at 256 bytes each = 1280 bytes,
        // leaves us 768 bytes for the remaining input.
        let mut s = String::with_capacity(2048);

        // Read until start
        // We try our best to find the start symbol, at the risk
        // of reading arbitrarily large inputs.
        loop {
            if file.read_line(&mut l)? == 0
                || l.trim_start_matches('#').trim_start().starts_with("method")
            {
                break;
            }
            l.clear();
        }
        remove_newline(&mut l);
        consume(&mut s, &l);
        l.clear();
        // Then read until we hit the end of meaningful input
        // If we have iterated through 255 lines, then something is clearly wrong.
        let mut stop = false;
        let mut n: u8 = 0;
        loop {
            if stop | (n == 255) || file.read_line(&mut l)? == 0 {
                break;
            } else {
                remove_newline(&mut l);
                stop = consume(&mut s, &l);
            }
            n += 1;
            l.clear();
        }
        Ok(s.trim().parse::<Self>())
    }
}

impl FromStr for ArgumentTree {
    type Err = ParseGrammarError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match GrammarParser::parse(Rule::argument_tree, s) {
            Ok(mut pairs) => {
                let pair = pairs.next().unwrap();
                Self::try_from_pair(pair)
            }
            Err(e) => Err(ArgumentTreeError(format!("{e:#?}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod output {
        use super::*;

        #[test]
        fn from_str() {
            let rhs = Output::default();
            assert_eq!("output".parse::<Output>().unwrap(), rhs);
            assert_eq!(
                "output file diagnostic_file refresh sig_figs profile_file"
                    .parse::<Output>()
                    .unwrap(),
                rhs
            );
            assert_eq!(
                "output file file=output.csv".parse::<Output>().unwrap(),
                rhs
            );

            let rhs = Output::builder()
                .file("foo")
                .diagnostic_file("bar")
                .profile_file("baz")
                .sig_figs(18)
                .refresh(123)
                .build();

            let s = "output file=bar file=baz file file=foo diagnostic_file=hello diagnostic_file= sig_figs=18 refresh=123 diagnostic_file=bar profile_file=bar profile_file=baz";
            assert_eq!(s.parse::<Output>().unwrap(), rhs);
        }
    }

    mod random {
        use super::*;

        #[test]
        fn from_str() {
            let rhs = Random::default();
            assert_eq!("random".parse::<Random>().unwrap(), rhs);
            assert_eq!("random seed".parse::<Random>().unwrap(), rhs);
            assert_eq!("random seed=-1 seed".parse::<Random>().unwrap(), rhs);

            let s = "random seed=123 seed=456 seed=789 seed";
            assert_eq!(s.parse::<Random>().unwrap(), Random { seed: 789 });
        }
    }

    mod data {
        use super::*;

        #[test]
        fn from_str() {
            let rhs = Data::default();
            assert_eq!("data".parse::<Data>().unwrap(), rhs);
            assert_eq!("data file".parse::<Data>().unwrap(), rhs);
            assert_eq!("data file= file".parse::<Data>().unwrap(), rhs);

            let s = "data file file=foo file=foo.bar file file=foo.bar.baz file=bar.baz file=baz";
            assert_eq!(s.parse::<Data>().unwrap(), Data { file: "baz".into() });
        }
    }

    mod argument_tree {
        use super::*;

        #[test]
        fn from_str() {
            let rhs = ArgumentTree::default();
            assert_eq!("sample".parse::<ArgumentTree>().unwrap(), rhs);
            assert_eq!("method=sample".parse::<ArgumentTree>().unwrap(), rhs);
            assert_eq!("method method=sample".parse::<ArgumentTree>().unwrap(), rhs);
            assert_eq!("method".parse::<ArgumentTree>().unwrap(), rhs);

            // Simple error: method unspecified
            assert!("id".parse::<ArgumentTree>().is_err());
            assert!("init".parse::<ArgumentTree>().is_err());
            assert!("random".parse::<ArgumentTree>().is_err());
            assert!("output".parse::<ArgumentTree>().is_err());
            assert!("num_threads".parse::<ArgumentTree>().is_err());
            assert!("data".parse::<ArgumentTree>().is_err());

            let methods = [
                "optimize",
                "variational",
                "generate_quantities",
                "diagnose",
                "pathfinder",
                "log_prob",
                "laplace",
            ];
            for m in methods {
                let t = m.parse::<ArgumentTree>().unwrap();
                assert_ne!(t, rhs);
            }
        }
    }
}
