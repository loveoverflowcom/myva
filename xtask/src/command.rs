use std::{collections::BTreeMap, path::PathBuf};

use crate::config::Workspace;

/// An executable plan, not a shell string. Display quoting never affects execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: BTreeMap<String, String>,
    pub stdout: Option<PathBuf>,
}

impl CommandSpec {
    pub fn new(workspace: &Workspace, program: &str, args: &[&str]) -> Self {
        Self {
            program: program.into(),
            args: args.iter().map(|s| (*s).into()).collect(),
            cwd: workspace.root.clone(),
            env: BTreeMap::new(),
            stdout: None,
        }
    }

    pub fn env(mut self, key: &str, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = String>) -> Self {
        self.args.extend(args);
        self
    }

    pub fn stdout(mut self, path: PathBuf) -> Self {
        self.stdout = Some(path);
        self
    }

    pub fn display(&self) -> String {
        let mut parts: Vec<String> = self
            .env
            .iter()
            .map(|(k, v)| format!("{k}={}", quote(v)))
            .collect();
        parts.extend(
            std::iter::once(&self.program)
                .chain(&self.args)
                .map(|arg| quote(arg)),
        );
        let mut display = parts.join(" ");
        if let Some(path) = &self.stdout {
            display.push_str(&format!(" > {}", quote(&path.to_string_lossy())));
        }
        display
    }
}

fn quote(arg: &str) -> String {
    if !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_./:=,-".contains(c))
    {
        arg.into()
    } else {
        format!("'{}'", arg.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_metacharacters_stay_in_separate_arguments() {
        let w = Workspace {
            root: "/tmp/my va".into(),
        };
        let command = CommandSpec::new(&w, "echo", &["$(touch x)", "a'b", ""]);
        assert_eq!(command.args, ["$(touch x)", "a'b", ""]);
        assert_eq!(command.display(), "echo '$(touch x)' 'a'\\''b' ''");
    }
}
