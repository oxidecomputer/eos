use crate::VERSION;
use std::io::Result;

/// A ninja build specification.
#[derive(Default)]
pub struct Spec {
    /// Variables to include in the ninja build spec.
    pub variables: Vec<Variable>,
    /// Rules to include in the ninja build spec.
    pub rules: Vec<RuleDefinition>,
    /// Statements to include in the ninja build spec.
    pub statements: Vec<BuildStatement>,
}

pub enum Rules {
    /// Used for compiling kernel modules.
    ModCompile,
    /// Used for linking kernel modules.
    ModLink,
    /// Used for linking genunix.
    GenunixLink,
}

impl ToString for Rules {
    fn to_string(&self) -> String {
        match self {
            Rules::ModCompile => "cc_kernel".into(),
            Rules::ModLink => "ld_kmod".into(),
            Rules::GenunixLink => "ld_genunix".into(),
        }
    }
}

impl Spec {
    /// Create and initialize a new ninja build spec. Initializes base rules and
    /// variables.
    pub fn new() -> Spec {
        let mut spec = Spec::default();
        spec.init();
        spec
    }

    /// Initialize base rules and variables.
    fn init(&mut self) {
        self.init_rules();
        self.init_variables();
    }

    /// Compiler flags used when compiling kernel objects.
    fn kernel_cflags() -> String {
        vec![
            "-std=gnu99",
            "-O3",
            "-g",
            "-gdwarf-2",
            "-gstrict-dwarf",
            "-m64",
            "-mcmodel=kernel",
            "-mindirect-branch-register",
            "-mindirect-branch=thunk-extern",
            "-mno-mmx",
            "-mno-red-zone",
            "-mno-sse",
            "-msave-args",
            "-D__sun",
            "-D__SVR4",
            "-D_ASM_INLINES",
            "-D_DDI_STRICT",
            "-D_ELF64",
            "-D_KERNEL",
            "-D_MACHDEP",
            "-D_SYSCALL32",
            "-D_SYSCALL32_IMPL",
            "-Dlint",
            "-Dsun",
            "-U__i386",
            "-Ui386",
            "-Iusr/src/uts/intel",
            "-Iusr/src/uts/common",
            "-Iusr/src/common",
            "-Iusr/src/uts/i86pc",
            "-Iusr/src/uts/common/fs/zfs",
            "-ffreestanding",
            "-fno-inline-small-functions",
            "-fno-inline-functions-called-once",
            "-fno-ipa-cp",
            "-fno-ipa-icf",
            "-fno-clone-functions",
            "-fno-reorder-functions",
            "-fno-reorder-blocks-and-partition",
            "-fno-aggressive-loop-optimizations",
            "-fno-shrink-wrap",
            "-fno-asynchronous-unwind-tables",
            "-fstack-protector-strong",
            "-fdiagnostics-color=always",
            "--param=max-inline-insns-single=450",
        ]
        .join(" ")
    }

    /// Flags to use when linking kernel components.
    fn kernel_ldflags() -> String {
        vec!["-ztype=kmod"].join(" ")
    }

    fn init_variables(&mut self) {
        self.variables.push(Variable {
            name: "kernel_cflags".into(),
            value: Self::kernel_cflags(),
        });
        self.variables.push(Variable {
            name: "kernel_ldflags".into(),
            value: Self::kernel_ldflags(),
        });
    }

    fn init_rules(&mut self) {
        self.rules.push(RuleDefinition {
            name: Rules::ModCompile.to_string(),
            command: vec![
                "gcc-10 $kernel_cflags -c $in -o $out",
                "ctfconvert -X -l '5.11' $out",
                "strip $out",
            ]
            .join(" && "),
        });
        self.rules.push(RuleDefinition {
            name: Rules::ModLink.to_string(),
            command: vec![
                "ld $kernel_ldflags $mod_deps -o $out $in",
                &format!(
                    "ctfmerge -l '{}' -d bld/genunix -o $out $in",
                    VERSION
                ),
            ]
            .join(" && "),
        });
        self.rules.push(RuleDefinition {
            name: Rules::GenunixLink.to_string(),
            command: vec![
                "ld $kernel_ldflags -o $out $in",
                &format!("ctfmerge -l '{}' -o $out $in", VERSION),
            ]
            .join(" && "),
        });
    }

    /// Emit this ninja spec as a string.
    fn emit(&self) -> String {
        let s = self.emit_variables();
        let s = s + &self.emit_rules();
        s + &self.emit_statements()
    }

    /// Emit this ninja spec to the file build.ninja.
    pub fn emit_file(&self) -> Result<()> {
        let out = self.emit();
        std::fs::write("build.ninja", out)?;
        Ok(())
    }

    /// Emit the variables in this spec in text form.
    fn emit_variables(&self) -> String {
        let mut s = String::new();
        for d in &self.variables {
            s += &d.emit();
        }
        s
    }

    /// Emit the rules in this spec in text form.
    fn emit_rules(&self) -> String {
        let mut s = String::new();
        for r in &self.rules {
            s += &r.emit();
        }
        s
    }

    /// Emit the build statements in this spec in text form.
    fn emit_statements(&self) -> String {
        let mut s = String::new();
        for stmt in &self.statements {
            s += &stmt.emit();
        }
        s
    }
}

/// A ninja variable
pub struct Variable {
    /// Name of the variable
    pub name: String,
    /// Value of the variable
    pub value: String,
}

impl Variable {
    /// Emit this variable in text form.
    fn emit(&self) -> String {
        format!("{} = {}\n", self.name, self.value)
    }
}

/// A ninja rule definition.
pub struct RuleDefinition {
    /// Name of the rule
    pub name: String,
    /// Command text
    pub command: String,
}

impl RuleDefinition {
    /// Emit this rule in text form.
    fn emit(&self) -> String {
        format!("rule {}\n  command = {}\n", self.name, self.command)
    }
}

/// A ninja build statement.
#[derive(Default)]
pub struct BuildStatement {
    /// Explicit inputs.
    pub input: String,
    /// What to produce.
    pub output: String,
    /// What build rule to use.
    pub rule: String,
    /// Optional variables for this specific build statement.
    pub variables: Vec<Variable>,
    /// Optional implicit dependencies.
    pub implicit_deps: Vec<String>,
}

impl BuildStatement {
    /// Emit this build statement in text form.
    fn emit(&self) -> String {
        let mut s =
            format!("build {}: {} {}\n", self.output, self.rule, self.input,);
        for d in &self.variables {
            s += &format!("  {}", d.emit());
        }
        s
    }
}
