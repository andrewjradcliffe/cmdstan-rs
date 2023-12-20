use crate::sample::*;
use std::ffi::{OsStr, OsString};

static SPACE1: &str = " ";
static SPACE2: &str = "  ";

pub trait Translate {
    fn to_args(&self) -> Vec<OsString>;
    fn to_tree(&self) -> OsString;

    /// Provided
    fn to_stmt(&self) -> OsString {
        let v = self.to_args();
        let n = v.len();
        if n != 0 {
            let cap: usize = v.iter().map(|s| s.len()).sum();
            let mut s = OsString::with_capacity(cap + n - 1);
            let mut iter = v.iter();
            s.push(iter.next().unwrap());
            for x in iter {
                s.push(SPACE1);
                s.push(x);
            }
            s
        } else {
            OsString::new()
        }
    }
}

fn split_at_newline_and_append(acc: &mut OsString, s: &OsStr) {
    let bytes = s.as_encoded_bytes();
    let lines = bytes.split(|b| *b == b'\n');
    for line in lines {
        // SAFETY:
        // - Each `line` only contains content that originated from `OsStr::as_encoded_bytes`
        // - Only split with ASCII newline which is a non-empty UTF-8 substring
        let line = unsafe { OsStr::from_encoded_bytes_unchecked(line)};
        if !line.is_empty() {
            acc.push(space);
            acc.push(line);
            acc.push("\n");
        }
    }
}
impl Translate for SampleAdapt {
    fn to_args(&self) -> Vec<OsString> {
        vec![
            "adapt".into(),
            format!("engaged={}", self.engaged as u8).into(),
            format!("gamma={}", self.gamma).into(),
            format!("delta={}", self.delta).into(),
            format!("kappa={}", self.kappa).into(),
            format!("t0={}", self.t0).into(),
            format!("init_buffer={}", self.init_buffer).into(),
            format!("term_buffer={}", self.term_buffer).into(),
            format!("window={}", self.window).into(),
        ]
    }
    fn to_tree(&self) -> OsString {
        format!("adapt\n  engaged = {}\n  gamma = {}\n  delta = {}\n  kappa = {}\n  t0 = {}\n  init_buffer = {}\n  term_buffer = {}\n  window = {}", self.engaged as u8, self.gamma, self.delta, self.kappa, self.t0, self.init_buffer, self.term_buffer, self.window).into()
    }
}

impl Translate for SampleAlgorithm {
    fn to_args(&self) -> Vec<OsString> {
        match &self {
            Self::Hmc {
                engine,
                metric,
                metric_file,
                stepsize,
                stepsize_jitter,
            } => {
                let mut engine = engine.to_args();
                let mut metric = metric.to_args();
                let mut v = Vec::with_capacity(4 + engine.len() + metric.len());
                v.push("algorithm=hmc".into());
                v.append(&mut engine);
                v.append(&mut metric);
                if !metric_file.is_empty() {
                    let mut s = OsString::with_capacity(12 + metric_file.len());
                    s.push("metric_file=");
                    s.push(metric_file);
                    v.push(s);
                }
                v.push(format!("stepsize={}", stepsize).into());
                v.push(format!("stepsize_jitter={}", stepsize_jitter).into());
                v
            }
            Self::FixedParam => vec!["algorithm=fixed_param".into()],
        }
    }
    fn to_tree(&self) -> OsString {
        match &self {
            Self::Hmc {
                engine,
                metric,
                metric_file,
                stepsize,
                stepsize_jitter,
            } => {
                let mut s = OsString::from("algorithm = hmc\n  hmc\n");
                let engine = engine.to_tree();
                let metric = metric.to_tree();
                split_at_newline_and_append(&mut s, &engine, "  ");
                split_at_newline_and_append(&mut s, &metric, "  ");
                s.push("  metric_file = ");
                s.push(metric_file);
                s.push("\n");
                s.push(&format!("  stepsize = {}\n", stepsize));
                s.push(&format!("  stepsize_jitter = {}", stepsize_jitter));
                s
            }
            Self::FixedParam => OsString::from("algorithm = fixed_param\n  fixed_param")
        }
    }
}


impl Translate for Engine {
    fn to_args(&self) -> Vec<OsString> {
        match &self {
            Engine::Nuts { max_depth } => {
                vec![
                    "engine=nuts".into(),
                    format!("max_depth={}", max_depth).into(),
                ]
            }
            Engine::Static { int_time } => {
                vec![
                    "engine=static".into(),
                    format!("int_time={}", int_time).into(),
                ]
            }
        }
    }
    fn to_tree(&self) -> OsString {
        match &self {
            Engine::Nuts { max_depth } => {
                let mut s = OsString::from("engine = nuts\n");
                s.push("  nuts\n");
                s.push("    ");
                s.push(&format!("max_depth = {}", max_depth));
                s
            }
            Engine::Static { int_time } => {
                let mut s = OsString::from("engine = static\n");
                s.push("  static\n");
                s.push("    ");
                s.push(&format!("int_time = {}", int_time));
                s
            }
        }
    }
}


impl Metric {
    fn as_str(&self) -> &'static str {
        match self {
            Metric::UnitE => "unit_e",
            Metric::DiagE => "diag_e",
            Metric::DenseE => "dense_e",
        }
    }
}

impl Translate for Metric {
    fn to_args(&self) -> Vec<OsString> {
        let s = format!("metric={}", self.as_str()).into();
        vec![s]
    }
    fn to_tree(&self) -> OsString {
        format!("metric = {}", self.as_str()).into()
    }
    fn to_stmt(&self) -> OsString {
        format!("metric={}", self.as_str()).into()
    }
}
