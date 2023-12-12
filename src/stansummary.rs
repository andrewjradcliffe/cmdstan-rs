use std::ffi::OsString;

/// Options for the `stansummary` tool. See
/// <https://mc-stan.org/docs/cmdstan-guide/stansummary.html> for more
/// information.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct StanSummaryOptions {
    /// Display the chain autocorrelation for the n-th input file, in
    /// addition to statistics.
    pub autocorr: Option<i32>,
    /// Write statistics to a csv file.
    pub csv_filename: Option<OsString>,
    /// Percentiles to report as ordered set of comma-separated
    /// integers from (1,99), inclusive. Default is 5,50,95.
    pub percentiles: Vec<f64>,
    /// Significant figures reported. Default is 2. Must be an integer
    /// from (1, 18), inclusive.
    pub sig_figs: u8,
}
impl StanSummaryOptions {
    pub fn builder() -> StanSummaryOptionsBuilder {
        StanSummaryOptionsBuilder::new()
    }

    pub fn command_fragment(&self) -> Vec<OsString> {
        let mut v = Vec::with_capacity(4);
        if let Some(n) = &self.autocorr {
            v.push(format!("--autocorr={}", n).into());
        }
        if let Some(file) = &self.csv_filename {
            let mut s = OsString::with_capacity(15 + file.len());
            s.push("--csv_filename=");
            s.push(file);
            v.push(s);
        }
        let mut s = OsString::with_capacity(14 + 3 * self.percentiles.len());
        s.push("--percentiles=");
        let mut values = self.percentiles.iter();
        if let Some(p) = values.next() {
            s.push(format!("{}", p));
        }
        for p in values {
            s.push(",");
            s.push(format!("{}", p));
        }
        v.push(s);
        v.push(format!("--sig_figs={}", self.sig_figs).into());
        v
    }
}

impl From<StanSummaryOptionsBuilder> for StanSummaryOptions {
    fn from(x: StanSummaryOptionsBuilder) -> Self {
        x.build()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StanSummaryOptionsBuilder {
    autocorr: Option<i32>,
    csv_filename: Option<OsString>,
    percentiles: Option<Vec<f64>>,
    sig_figs: Option<u8>,
}
impl StanSummaryOptionsBuilder {
    insert_field!(autocorr, i32);
    insert_into_field!(csv_filename, OsString);
    insert_into_field!(percentiles, Vec<f64>);
    insert_field!(sig_figs, u8);

    pub fn new() -> Self {
        Self {
            autocorr: None,
            csv_filename: None,
            percentiles: None,
            sig_figs: None,
        }
    }
    pub fn build(self) -> StanSummaryOptions {
        let percentiles = self.percentiles.unwrap_or_else(|| vec![5.0, 50.0, 95.0]);
        let sig_figs = self.sig_figs.unwrap_or(2);
        StanSummaryOptions {
            autocorr: self.autocorr,
            csv_filename: self.csv_filename,
            percentiles,
            sig_figs,
        }
    }
}
impl Default for StanSummaryOptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod options {
        use super::*;

        #[test]
        fn command_fragment() {
            let x = StanSummaryOptions {
                autocorr: None,
                csv_filename: Some("stansummary.csv".into()),
                percentiles: vec![5.0, 25.0, 50.0, 75.0, 95.0],
                sig_figs: 6,
            };
            assert_eq!(
                x.command_fragment(),
                vec![
                    "--csv_filename=stansummary.csv",
                    "--percentiles=5,25,50,75,95",
                    "--sig_figs=6",
                ]
            );

            let x = StanSummaryOptions {
                autocorr: Some(1),
                csv_filename: None,
                percentiles: vec![50.0, 75.0],
                sig_figs: 2,
            };
            assert_eq!(
                x.command_fragment(),
                vec!["--autocorr=1", "--percentiles=50,75", "--sig_figs=2"]
            );

            let x = StanSummaryOptions {
                autocorr: Some(1),
                csv_filename: Some("hello.csv".into()),
                percentiles: vec![50.0],
                sig_figs: 4,
            };
            assert_eq!(
                x.command_fragment(),
                vec![
                    "--autocorr=1",
                    "--csv_filename=hello.csv",
                    "--percentiles=50",
                    "--sig_figs=4"
                ]
            );

            let x = StanSummaryOptions {
                autocorr: None,
                csv_filename: Some("hello.csv".into()),
                percentiles: vec![50.0],
                sig_figs: 3,
            };
            assert_eq!(
                x.command_fragment(),
                vec![
                    "--csv_filename=hello.csv",
                    "--percentiles=50",
                    "--sig_figs=3"
                ]
            );
        }
    }

    mod builder {
        use super::*;

        #[test]
        fn default() {
            let x = StanSummaryOptions::builder().build();
            assert_eq!(
                x,
                StanSummaryOptions {
                    autocorr: None,
                    csv_filename: None,
                    percentiles: vec![5.0, 50.0, 95.0],
                    sig_figs: 2
                }
            );
        }
    }
}
