use crate::{
    github::actions::{
        self, cmd, install, install_rust, pull_request, push, rust_toolchain, script, Platform,
        Run, Rust, Step, Workflow,
    },
    WorkflowResult,
};

#[derive(Default)]
pub struct CI(Vec<Tasks>);

impl CI {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn standard_workflow() -> Self {
        Self::new()
            .standard_tests("1.73")
            .standard_release_tests("1.73")
            .standard_lints("nightly-2023-10-14", "0.1.43")
    }

    pub fn standard_lints(self, rustc_version: &str, udeps_version: &str) -> Self {
        self.job(
            Tasks::new(
                "lints",
                Platform::UbuntuLatest,
                rust_toolchain(rustc_version).minimal().default().rustfmt(),
            )
            .lints(udeps_version),
        )
    }

    pub fn standard_tests(mut self, rustc_version: &str) -> Self {
        for platform in Platform::latest() {
            self.0.push(
                Tasks::new(
                    "tests",
                    platform,
                    rust_toolchain(rustc_version).minimal().default().clippy(),
                )
                .tests(),
            );
        }

        self
    }

    pub fn standard_release_tests(mut self, rustc_version: &str) -> Self {
        for platform in Platform::latest() {
            self.0.push(
                Tasks::new(
                    "release-tests",
                    platform,
                    rust_toolchain(rustc_version).minimal().default(),
                )
                .release_tests(),
            );
        }

        self
    }

    pub fn job(mut self, tasks: Tasks) -> Self {
        self.add_job(tasks);
        self
    }

    pub fn add_job(&mut self, tasks: Tasks) {
        self.0.push(tasks);
    }

    pub fn write(self, check: bool) -> WorkflowResult<()> {
        self.into_workflow().write(check)
    }

    pub fn run(self) -> WorkflowResult<()> {
        for task in self.0 {
            task.run()?;
        }

        Ok(())
    }

    fn into_workflow(self) -> Workflow {
        let mut workflow = actions::workflow("ci-tests").on([push(), pull_request()]);

        for task in self.0 {
            workflow.add_job(
                &task.name,
                task.platform,
                task.tasks.into_iter().map(Step::from),
            );
        }

        workflow
    }
}

pub struct Tasks {
    name: String,
    platform: Platform,
    is_nightly: bool,
    tasks: Vec<Task>,
}

impl Tasks {
    pub fn new(name: impl Into<String>, platform: Platform, rust: Rust) -> Self {
        Self {
            name: name.into(),
            platform,
            is_nightly: rust.is_nightly(),
            tasks: Vec::new(),
        }
        .step(install_rust(rust))
    }

    pub fn run(self) -> WorkflowResult<()> {
        if self.platform.is_current() {
            for task in self.tasks.into_iter() {
                if let Task::Run(cmd) = task {
                    cmd.run(self.is_nightly)?;
                }
            }
        }

        Ok(())
    }

    pub fn step(mut self, step: Step) -> Self {
        self.tasks.push(Task::Install(step));
        self
    }

    pub fn cmd(
        mut self,
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.tasks.push(Task::Run(cmd(program, args)));
        self
    }

    pub fn script<Cmds, Cmd, Arg>(mut self, cmds: Cmds) -> Self
    where
        Cmds: IntoIterator<Item = Cmd>,
        Cmd: IntoIterator<Item = Arg>,
        Arg: Into<String>,
    {
        self.tasks.push(Task::Run(script(cmds)));
        self
    }

    pub fn tests(self) -> Self {
        self.cmd("cargo", ["xtask", "codegen", "--check"])
            .cmd(
                "cargo",
                [
                    "clippy",
                    "--all-targets",
                    "--",
                    "-D",
                    "warnings",
                    "-D",
                    "clippy::all",
                ],
            )
            .cmd("cargo", ["test"])
            .cmd("cargo", ["build", "--all-targets"])
            .cmd("cargo", ["doc"])
    }

    pub fn release_tests(self) -> Self {
        self.cmd("cargo", ["test", "--benches", "--tests", "--release"])
    }

    pub fn lints(self, udeps_version: &str) -> Self {
        self.cmd("cargo", ["fmt", "--all", "--", "--check"])
            .step(install("cargo-udeps", udeps_version))
            .cmd("cargo", ["udeps", "--all-targets"])
    }
}

enum Task {
    Install(Step),
    Run(Run),
}

impl From<Task> for Step {
    fn from(value: Task) -> Self {
        match value {
            Task::Install(step) => step,
            Task::Run(cmd) => cmd.into(),
        }
    }
}
