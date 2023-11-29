use crate::argument_tree::*;
use crate::method::Method;
use crate::parser::*;
use std::ffi::OsString;

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
                let pairs = pair
                    .into_inner()
                    .filter_map(|seed| seed.into_inner().next());
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
}
