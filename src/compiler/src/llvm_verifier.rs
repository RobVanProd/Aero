//! External LLVM 22 module verification.
//!
//! This module deliberately owns only verifier selection and process execution.
//! Callers remain responsible for choosing [`LlvmVerificationMode`] and for
//! invoking verification after all LLVM text transformations but before cache,
//! artifact, native-tool, or trace publication.

use std::env;
use std::ffi::OsString;
use std::fmt;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// The only LLVM major admitted by the CORE-010 verifier contract.
pub const REQUIRED_LLVM_MAJOR: u32 = 22;

/// The production subprocess deadline.
///
/// CLI tests use a longer outer deadlock guard; this deadline is the actual
/// production bound applied independently to version probes and verification.
pub const LLVM_VERIFIER_TIMEOUT: Duration = Duration::from_secs(10);

/// Whether absence of an external verifier is permitted for this invocation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LlvmVerificationMode {
    /// A genuinely absent verifier permits a fresh internally verified path.
    #[default]
    PreferExternal,
    /// A compatible external verifier must be present and accept the module.
    Required,
}

/// The external program used to verify LLVM text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LlvmVerifierKind {
    Opt,
    LlvmAs,
}

impl fmt::Display for LlvmVerifierKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Opt => formatter.write_str("opt"),
            Self::LlvmAs => formatter.write_str("llvm-as"),
        }
    }
}

/// A compatible verifier selected for one verification operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LlvmVerifierIdentity {
    pub kind: LlvmVerifierKind,
    pub path: PathBuf,
    pub version: String,
    pub major: u32,
}

/// Successful verification disposition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LlvmVerificationStatus {
    /// The exact supplied bytes were accepted by the recorded LLVM 22 tool.
    ExternalVerified(LlvmVerifierIdentity),
    /// No candidate executable existed and the caller allowed internal-only use.
    InternalOnly { attempted: Vec<PathBuf> },
}

impl LlvmVerificationStatus {
    pub fn external_verifier(&self) -> Option<&LlvmVerifierIdentity> {
        match self {
            Self::ExternalVerified(verifier) => Some(verifier),
            Self::InternalOnly { .. } => None,
        }
    }
}

impl fmt::Display for LlvmVerificationStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExternalVerified(verifier) => write!(
                formatter,
                "ExternalVerified: {} {} at {}",
                verifier.kind,
                verifier.version,
                display_path(&verifier.path)
            ),
            Self::InternalOnly { attempted } => write!(
                formatter,
                "InternalOnly: no LLVM {REQUIRED_LLVM_MAJOR} opt/llvm-as verifier found ({})",
                display_attempts(attempted)
            ),
        }
    }
}

/// The subprocess phase in which an adapter failure occurred.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LlvmVerifierOperation {
    VersionProbe,
    ModuleVerification,
}

impl fmt::Display for LlvmVerifierOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VersionProbe => formatter.write_str("version probe"),
            Self::ModuleVerification => formatter.write_str("module verification"),
        }
    }
}

/// Structured external-verification failure.
///
/// Every display form retains the stable `LLVM Verification Error:` phase
/// identity required by trusted library and CLI callers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LlvmVerificationError {
    EmptyModule,
    ToolNotFound {
        tool: PathBuf,
        override_variable: Option<&'static str>,
    },
    NoCompatibleVerifier {
        attempted: Vec<PathBuf>,
    },
    VersionProbeFailed {
        tool: PathBuf,
        exit_code: Option<i32>,
        details: String,
    },
    UnrecognizedVersion {
        tool: PathBuf,
        output: String,
    },
    WrongVersion {
        tool: PathBuf,
        discovered: String,
        required_major: u32,
    },
    LaunchFailed {
        tool: PathBuf,
        operation: LlvmVerifierOperation,
        details: String,
    },
    TimedOut {
        tool: PathBuf,
        operation: LlvmVerifierOperation,
        timeout: Duration,
    },
    StdinFailed {
        tool: PathBuf,
        details: String,
    },
    Rejected {
        tool: PathBuf,
        exit_code: Option<i32>,
        details: String,
    },
}

impl fmt::Display for LlvmVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("LLVM Verification Error: ")?;
        match self {
            Self::EmptyModule => formatter.write_str("refusing to verify empty LLVM input"),
            Self::ToolNotFound {
                tool,
                override_variable,
            } => {
                if let Some(variable) = override_variable {
                    write!(
                        formatter,
                        "authoritative {variable} verifier `{}` was not found",
                        display_path(tool)
                    )
                } else {
                    write!(formatter, "verifier `{}` was not found", display_path(tool))
                }
            }
            Self::NoCompatibleVerifier { attempted } => write!(
                formatter,
                "required LLVM {REQUIRED_LLVM_MAJOR} opt/llvm-as verifier was not found ({})",
                display_attempts(attempted)
            ),
            Self::VersionProbeFailed {
                tool,
                exit_code,
                details,
            } => write!(
                formatter,
                "verifier `{}` rejected its version probe{}: {}",
                display_path(tool),
                display_exit_code(*exit_code),
                display_details(details)
            ),
            Self::UnrecognizedVersion { tool, output } => write!(
                formatter,
                "could not parse the LLVM major reported by `{}`: {}",
                display_path(tool),
                display_details(output)
            ),
            Self::WrongVersion {
                tool,
                discovered,
                required_major,
            } => write!(
                formatter,
                "verifier `{}` reported LLVM {discovered}; LLVM {required_major} is required",
                display_path(tool)
            ),
            Self::LaunchFailed {
                tool,
                operation,
                details,
            } => write!(
                formatter,
                "failed to launch verifier `{}` for {operation}: {}",
                display_path(tool),
                display_details(details)
            ),
            Self::TimedOut {
                tool,
                operation,
                timeout,
            } => write!(
                formatter,
                "verifier `{}` timed out during {operation} after {:.3}s",
                display_path(tool),
                timeout.as_secs_f64()
            ),
            Self::StdinFailed { tool, details } => write!(
                formatter,
                "failed to send complete LLVM input to verifier `{}`: {}",
                display_path(tool),
                display_details(details)
            ),
            Self::Rejected {
                tool,
                exit_code,
                details,
            } => write!(
                formatter,
                "verifier `{}` rejected the module{}: {}",
                display_path(tool),
                display_exit_code(*exit_code),
                display_details(details)
            ),
        }
    }
}

impl std::error::Error for LlvmVerificationError {}

/// Verify final LLVM text according to the selected availability policy.
///
/// The module bytes are forwarded unchanged. A successful `InternalOnly` result
/// means every external candidate was genuinely absent; all other discovery or
/// verifier failures are errors in both modes.
pub fn verify_llvm_module(
    llvm: &str,
    mode: LlvmVerificationMode,
) -> Result<LlvmVerificationStatus, LlvmVerificationError> {
    verify_llvm_module_with_host(llvm, mode, &SystemLlvmVerifierHost, LLVM_VERIFIER_TIMEOUT)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LlvmVerifierProcessRequest {
    pub program: PathBuf,
    pub arguments: Vec<OsString>,
    pub stdin: Option<Vec<u8>>,
    pub timeout: Duration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LlvmVerifierProcessOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LlvmVerifierProcessError {
    NotFound,
    Launch(String),
    TimedOut,
    Stdin(String),
    Io(String),
}

/// Dependency seam for environment and subprocess control.
///
/// Keeping both operations in one host prevents tests from mutating process-global
/// environment variables merely to exercise discovery order.
pub(crate) trait LlvmVerifierHost {
    fn environment_override(&self, name: &'static str) -> Option<OsString>;

    fn run_process(
        &self,
        request: LlvmVerifierProcessRequest,
    ) -> Result<LlvmVerifierProcessOutput, LlvmVerifierProcessError>;
}

/// Injectable form used by module tests and later binary/library wiring tests.
pub(crate) fn verify_llvm_module_with_host(
    llvm: &str,
    mode: LlvmVerificationMode,
    host: &dyn LlvmVerifierHost,
    timeout: Duration,
) -> Result<LlvmVerificationStatus, LlvmVerificationError> {
    if llvm.is_empty() {
        return Err(LlvmVerificationError::EmptyModule);
    }

    let discovery = discover_verifier(host, timeout)?;
    let Some(verifier) = discovery.verifier else {
        return match mode {
            LlvmVerificationMode::PreferExternal => Ok(LlvmVerificationStatus::InternalOnly {
                attempted: discovery.attempted,
            }),
            LlvmVerificationMode::Required => Err(LlvmVerificationError::NoCompatibleVerifier {
                attempted: discovery.attempted,
            }),
        };
    };

    let arguments = match verifier.kind {
        LlvmVerifierKind::Opt => ["-passes=verify", "-disable-output", "-"]
            .into_iter()
            .map(OsString::from)
            .collect(),
        LlvmVerifierKind::LlvmAs => ["-o", "-", "-"].into_iter().map(OsString::from).collect(),
    };
    let request = LlvmVerifierProcessRequest {
        program: verifier.path.clone(),
        arguments,
        stdin: Some(llvm.as_bytes().to_vec()),
        timeout,
    };

    match host.run_process(request) {
        Ok(output) if output.success => Ok(LlvmVerificationStatus::ExternalVerified(verifier)),
        Ok(output) => Err(LlvmVerificationError::Rejected {
            tool: verifier.path,
            exit_code: output.exit_code,
            details: process_details(&output),
        }),
        Err(LlvmVerifierProcessError::NotFound) => Err(LlvmVerificationError::ToolNotFound {
            tool: verifier.path,
            override_variable: None,
        }),
        Err(LlvmVerifierProcessError::TimedOut) => Err(LlvmVerificationError::TimedOut {
            tool: verifier.path,
            operation: LlvmVerifierOperation::ModuleVerification,
            timeout,
        }),
        Err(LlvmVerifierProcessError::Stdin(details)) => Err(LlvmVerificationError::StdinFailed {
            tool: verifier.path,
            details,
        }),
        Err(LlvmVerifierProcessError::Launch(details) | LlvmVerifierProcessError::Io(details)) => {
            Err(LlvmVerificationError::LaunchFailed {
                tool: verifier.path,
                operation: LlvmVerifierOperation::ModuleVerification,
                details,
            })
        }
    }
}

struct VerifierDiscovery {
    verifier: Option<LlvmVerifierIdentity>,
    attempted: Vec<PathBuf>,
}

struct VerifierCandidate {
    kind: LlvmVerifierKind,
    path: PathBuf,
    override_variable: Option<&'static str>,
}

impl VerifierCandidate {
    fn authoritative(&self) -> bool {
        self.override_variable.is_some()
    }
}

fn discover_verifier(
    host: &dyn LlvmVerifierHost,
    timeout: Duration,
) -> Result<VerifierDiscovery, LlvmVerificationError> {
    if let Some(path) = host.environment_override("AERO_LLVM_OPT") {
        let candidate = VerifierCandidate {
            kind: LlvmVerifierKind::Opt,
            path: PathBuf::from(path),
            override_variable: Some("AERO_LLVM_OPT"),
        };
        return Ok(VerifierDiscovery {
            verifier: probe_candidate(host, &candidate, timeout)?,
            attempted: vec![candidate.path],
        });
    }

    if let Some(path) = host.environment_override("AERO_LLVM_AS") {
        let candidate = VerifierCandidate {
            kind: LlvmVerifierKind::LlvmAs,
            path: PathBuf::from(path),
            override_variable: Some("AERO_LLVM_AS"),
        };
        return Ok(VerifierDiscovery {
            verifier: probe_candidate(host, &candidate, timeout)?,
            attempted: vec![candidate.path],
        });
    }

    let candidates = [
        VerifierCandidate {
            kind: LlvmVerifierKind::Opt,
            path: PathBuf::from("opt-22"),
            override_variable: None,
        },
        VerifierCandidate {
            kind: LlvmVerifierKind::LlvmAs,
            path: PathBuf::from("llvm-as-22"),
            override_variable: None,
        },
        VerifierCandidate {
            kind: LlvmVerifierKind::Opt,
            path: PathBuf::from("opt"),
            override_variable: None,
        },
        VerifierCandidate {
            kind: LlvmVerifierKind::LlvmAs,
            path: PathBuf::from("llvm-as"),
            override_variable: None,
        },
    ];

    let mut attempted = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        attempted.push(candidate.path.clone());
        if let Some(verifier) = probe_candidate(host, &candidate, timeout)? {
            return Ok(VerifierDiscovery {
                verifier: Some(verifier),
                attempted,
            });
        }
    }

    Ok(VerifierDiscovery {
        verifier: None,
        attempted,
    })
}

fn probe_candidate(
    host: &dyn LlvmVerifierHost,
    candidate: &VerifierCandidate,
    timeout: Duration,
) -> Result<Option<LlvmVerifierIdentity>, LlvmVerificationError> {
    let request = LlvmVerifierProcessRequest {
        program: candidate.path.clone(),
        arguments: vec![OsString::from("--version")],
        stdin: None,
        timeout,
    };

    let output = match host.run_process(request) {
        Ok(output) => output,
        Err(LlvmVerifierProcessError::NotFound) if !candidate.authoritative() => return Ok(None),
        Err(LlvmVerifierProcessError::NotFound) => {
            return Err(LlvmVerificationError::ToolNotFound {
                tool: candidate.path.clone(),
                override_variable: candidate.override_variable,
            });
        }
        Err(LlvmVerifierProcessError::TimedOut) => {
            return Err(LlvmVerificationError::TimedOut {
                tool: candidate.path.clone(),
                operation: LlvmVerifierOperation::VersionProbe,
                timeout,
            });
        }
        Err(
            LlvmVerifierProcessError::Launch(details)
            | LlvmVerifierProcessError::Stdin(details)
            | LlvmVerifierProcessError::Io(details),
        ) => {
            return Err(LlvmVerificationError::LaunchFailed {
                tool: candidate.path.clone(),
                operation: LlvmVerifierOperation::VersionProbe,
                details,
            });
        }
    };

    if !output.success {
        return Err(LlvmVerificationError::VersionProbeFailed {
            tool: candidate.path.clone(),
            exit_code: output.exit_code,
            details: process_details(&output),
        });
    }

    let version_output = combined_process_text(&output);
    let Some((major, version)) = parse_llvm_version(&version_output) else {
        return Err(LlvmVerificationError::UnrecognizedVersion {
            tool: candidate.path.clone(),
            output: version_output,
        });
    };
    if major != REQUIRED_LLVM_MAJOR {
        return Err(LlvmVerificationError::WrongVersion {
            tool: candidate.path.clone(),
            discovered: version,
            required_major: REQUIRED_LLVM_MAJOR,
        });
    }

    Ok(Some(LlvmVerifierIdentity {
        kind: candidate.kind,
        path: candidate.path.clone(),
        version,
        major,
    }))
}

fn parse_llvm_version(output: &str) -> Option<(u32, String)> {
    for line in output.lines() {
        let lowercase = line.to_ascii_lowercase();
        let Some(llvm_index) = lowercase.find("llvm") else {
            continue;
        };
        let after_llvm = &lowercase[llvm_index + "llvm".len()..];
        let Some(version_index) = after_llvm.find("version") else {
            continue;
        };
        let version_start = llvm_index + "llvm".len() + version_index + "version".len();
        let remainder = &line[version_start..];
        let Some(first_digit) = remainder.find(|character: char| character.is_ascii_digit()) else {
            continue;
        };
        let version = remainder[first_digit..]
            .chars()
            .take_while(|character| character.is_ascii_digit() || *character == '.')
            .collect::<String>()
            .trim_end_matches('.')
            .to_string();
        let Some(major) = version
            .split('.')
            .next()
            .and_then(|major| major.parse().ok())
        else {
            continue;
        };
        return Some((major, version));
    }
    None
}

fn process_details(output: &LlvmVerifierProcessOutput) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        return stderr;
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return stdout;
    }
    "no diagnostic output".to_string()
}

fn combined_process_text(output: &LlvmVerifierProcessOutput) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    match (stdout.trim().is_empty(), stderr.trim().is_empty()) {
        (false, false) => format!("{}\n{}", stdout.trim(), stderr.trim()),
        (false, true) => stdout.trim().to_string(),
        (true, false) => stderr.trim().to_string(),
        (true, true) => String::new(),
    }
}

fn display_path(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        "<empty>".to_string()
    } else {
        path.display().to_string()
    }
}

fn display_attempts(attempted: &[PathBuf]) -> String {
    if attempted.is_empty() {
        return "no candidates".to_string();
    }
    format!(
        "tried {}",
        attempted
            .iter()
            .map(|path| format!("`{}`", display_path(path)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn display_exit_code(exit_code: Option<i32>) -> String {
    exit_code.map_or_else(String::new, |code| format!(" (exit {code})"))
}

fn display_details(details: &str) -> &str {
    if details.trim().is_empty() {
        "no diagnostic output"
    } else {
        details.trim()
    }
}

struct SystemLlvmVerifierHost;

#[cfg(windows)]
struct ProcessTree {
    job: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl ProcessTree {
    fn attach(child: &Child) -> Result<Self, LlvmVerifierProcessError> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
        };
        use windows_sys::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            SetInformationJobObject, TerminateJobObject,
        };
        use windows_sys::Win32::System::Threading::{
            OpenThread, ResumeThread, THREAD_SUSPEND_RESUME,
        };

        // SAFETY: both optional CreateJobObjectW pointers are null, requesting an
        // unnamed job with default security attributes.
        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            return Err(LlvmVerifierProcessError::Launch(format!(
                "failed to create verifier process job: {}",
                io::Error::last_os_error()
            )));
        }

        // SAFETY: the Windows structure is plain C data and zero is the documented
        // baseline before selecting only the KILL_ON_JOB_CLOSE limit flag.
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: `job` is a live job handle and `limits` points to a correctly sized
        // JOBOBJECT_EXTENDED_LIMIT_INFORMATION for the requested information class.
        let configured = unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                std::ptr::addr_of!(limits).cast(),
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if configured == 0 {
            let error = io::Error::last_os_error();
            // SAFETY: `job` was created above and has not been closed.
            unsafe { CloseHandle(job) };
            return Err(LlvmVerifierProcessError::Launch(format!(
                "failed to configure verifier process job: {error}"
            )));
        }

        // SAFETY: both handles are live. Job membership is inherited by descendants,
        // which makes the production timeout a process-tree deadline.
        let assigned = unsafe { AssignProcessToJobObject(job, child.as_raw_handle()) };
        if assigned == 0 {
            let error = io::Error::last_os_error();
            // SAFETY: `job` was created above and has not been closed.
            unsafe { CloseHandle(job) };
            return Err(LlvmVerifierProcessError::Launch(format!(
                "failed to contain verifier process tree: {error}"
            )));
        }

        // The process was created suspended, so its primary thread cannot run or
        // create an escaping descendant before job assignment. std::process does not
        // expose that thread handle; enumerate the still-single-threaded child and
        // resume it only after containment succeeds.
        // SAFETY: ToolHelp returns an owned snapshot handle or INVALID_HANDLE_VALUE.
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
        if snapshot == INVALID_HANDLE_VALUE {
            let error = io::Error::last_os_error();
            // SAFETY: the child is a member of `job`; termination prevents a suspended
            // verifier leak, and both handles are live.
            unsafe {
                TerminateJobObject(job, 1);
                CloseHandle(job);
            }
            return Err(LlvmVerifierProcessError::Launch(format!(
                "failed to enumerate suspended verifier threads: {error}"
            )));
        }

        // SAFETY: THREADENTRY32 is plain C data and dwSize is required by ToolHelp.
        let mut entry: THREADENTRY32 = unsafe { std::mem::zeroed() };
        entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
        let mut resumed = false;
        // SAFETY: `snapshot` and `entry` are valid for the duration of enumeration.
        let mut has_entry = unsafe { Thread32First(snapshot, std::ptr::addr_of_mut!(entry)) } != 0;
        while has_entry {
            if entry.th32OwnerProcessID == child.id() {
                // SAFETY: the enumerated thread belongs to the newly created child.
                let thread = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID) };
                if !thread.is_null() {
                    // SAFETY: `thread` was opened with THREAD_SUSPEND_RESUME.
                    let previous_suspend_count = unsafe { ResumeThread(thread) };
                    // SAFETY: `thread` is an owned handle returned by OpenThread.
                    unsafe { CloseHandle(thread) };
                    if previous_suspend_count != u32::MAX {
                        resumed = true;
                        break;
                    }
                }
            }
            // SAFETY: `snapshot` and `entry` remain valid.
            has_entry = unsafe { Thread32Next(snapshot, std::ptr::addr_of_mut!(entry)) } != 0;
        }
        // SAFETY: `snapshot` is an owned handle returned by ToolHelp.
        unsafe { CloseHandle(snapshot) };
        if !resumed {
            let error = io::Error::last_os_error();
            // SAFETY: the suspended child belongs to `job`; terminate before closing.
            unsafe {
                TerminateJobObject(job, 1);
                CloseHandle(job);
            }
            return Err(LlvmVerifierProcessError::Launch(format!(
                "failed to resume contained verifier process: {error}"
            )));
        }

        Ok(Self { job })
    }

    fn terminate(&self) {
        // SAFETY: `self.job` remains live for this guard's lifetime. Terminating the
        // job closes every verifier descendant before the adapter returns timeout.
        unsafe {
            windows_sys::Win32::System::JobObjects::TerminateJobObject(self.job, 1);
        }
    }
}

#[cfg(windows)]
impl Drop for ProcessTree {
    fn drop(&mut self) {
        // SAFETY: this guard uniquely owns the job handle.
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.job);
        }
    }
}

#[cfg(unix)]
struct ProcessTree {
    process_group: i32,
}

#[cfg(unix)]
impl ProcessTree {
    fn attach(child: &Child) -> Result<Self, LlvmVerifierProcessError> {
        Ok(Self {
            process_group: child.id() as i32,
        })
    }

    fn terminate(&self) {
        // SAFETY: the child was spawned as the leader of a new process group. A
        // negative PID targets that group and cannot target the compiler's group.
        unsafe {
            libc::kill(-self.process_group, libc::SIGKILL);
        }
    }
}

#[cfg(not(any(unix, windows)))]
struct ProcessTree;

#[cfg(not(any(unix, windows)))]
impl ProcessTree {
    fn attach(_child: &Child) -> Result<Self, LlvmVerifierProcessError> {
        Ok(Self)
    }

    fn terminate(&self) {}
}

impl LlvmVerifierHost for SystemLlvmVerifierHost {
    fn environment_override(&self, name: &'static str) -> Option<OsString> {
        env::var_os(name)
    }

    fn run_process(
        &self,
        request: LlvmVerifierProcessRequest,
    ) -> Result<LlvmVerifierProcessOutput, LlvmVerifierProcessError> {
        run_system_process(request)
    }
}

fn run_system_process(
    request: LlvmVerifierProcessRequest,
) -> Result<LlvmVerifierProcessOutput, LlvmVerifierProcessError> {
    let mut command = Command::new(&request.program);
    command
        .args(&request.arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if request.stdin.is_some() {
        command.stdin(Stdio::piped());
    } else {
        command.stdin(Stdio::null());
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(windows_sys::Win32::System::Threading::CREATE_SUSPENDED);
    }

    let mut child = command.spawn().map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => LlvmVerifierProcessError::NotFound,
        _ => LlvmVerifierProcessError::Launch(error.to_string()),
    })?;
    let process_tree = match ProcessTree::attach(&child) {
        Ok(process_tree) => process_tree,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
    };

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| LlvmVerifierProcessError::Io("stdout pipe unavailable".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| LlvmVerifierProcessError::Io("stderr pipe unavailable".to_string()))?;
    let stdout_reader = thread::spawn(move || read_pipe(stdout));
    let stderr_reader = thread::spawn(move || read_pipe(stderr));

    let stdin_writer = match request.stdin {
        Some(input) => {
            let mut stdin = child.stdin.take().ok_or_else(|| {
                LlvmVerifierProcessError::Stdin("stdin pipe unavailable".to_string())
            })?;
            Some(thread::spawn(move || {
                stdin.write_all(&input)?;
                stdin.flush()
            }))
        }
        None => None,
    };

    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= request.timeout => {
                process_tree.terminate();
                let _ = child.kill();
                let _ = child.wait();
                abandon_io_threads(stdin_writer, stdout_reader, stderr_reader);
                return Err(LlvmVerifierProcessError::TimedOut);
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(error) => {
                process_tree.terminate();
                let _ = child.kill();
                let _ = child.wait();
                abandon_io_threads(stdin_writer, stdout_reader, stderr_reader);
                return Err(LlvmVerifierProcessError::Io(format!(
                    "failed while waiting for verifier: {error}"
                )));
            }
        }
    };

    // A wrapper can exit successfully after spawning a descendant that inherits
    // its stdio handles. Bound pipe completion by the same operation deadline so
    // joining reader/writer threads cannot outlive verifier verification.
    while !stdin_writer
        .as_ref()
        .is_none_or(thread::JoinHandle::is_finished)
        || !stdout_reader.is_finished()
        || !stderr_reader.is_finished()
    {
        if started.elapsed() >= request.timeout {
            process_tree.terminate();
            abandon_io_threads(stdin_writer, stdout_reader, stderr_reader);
            return Err(LlvmVerifierProcessError::TimedOut);
        }
        thread::sleep(Duration::from_millis(10));
    }

    let stdin_result = join_stdin(stdin_writer);
    let stdout = join_reader(stdout_reader, "stdout")?;
    let stderr = join_reader(stderr_reader, "stderr")?;

    if status.success()
        && let Err(details) = stdin_result
    {
        return Err(LlvmVerifierProcessError::Stdin(details));
    }

    Ok(LlvmVerifierProcessOutput {
        success: status.success(),
        exit_code: status.code(),
        stdout,
        stderr,
    })
}

fn read_pipe(mut pipe: impl Read) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    pipe.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn join_stdin(writer: Option<thread::JoinHandle<io::Result<()>>>) -> Result<(), String> {
    let Some(writer) = writer else {
        return Ok(());
    };
    writer
        .join()
        .map_err(|_| "stdin writer thread panicked".to_string())?
        .map_err(|error| error.to_string())
}

fn join_reader(
    reader: thread::JoinHandle<io::Result<Vec<u8>>>,
    stream: &str,
) -> Result<Vec<u8>, LlvmVerifierProcessError> {
    reader
        .join()
        .map_err(|_| LlvmVerifierProcessError::Io(format!("{stream} reader thread panicked")))?
        .map_err(|error| {
            LlvmVerifierProcessError::Io(format!("failed to read verifier {stream}: {error}"))
        })
}

fn abandon_io_threads(
    stdin: Option<thread::JoinHandle<io::Result<()>>>,
    stdout: thread::JoinHandle<io::Result<Vec<u8>>>,
    stderr: thread::JoinHandle<io::Result<Vec<u8>>>,
) {
    // A script wrapper can leave a descendant holding inherited pipe handles after
    // the wrapper itself is killed. Joining here would let that descendant defeat
    // the adapter deadline. Dropping JoinHandles detaches the cleanup readers; the
    // operating system closes their handles when the descendant exits (or when the
    // caller process exits). The selected verifier process itself has been killed
    // and reaped before this function is called.
    drop(stdin);
    drop(stdout);
    drop(stderr);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::{HashMap, VecDeque};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Default)]
    struct FakeHost {
        environment: HashMap<&'static str, OsString>,
        responses: RefCell<VecDeque<Result<LlvmVerifierProcessOutput, LlvmVerifierProcessError>>>,
        requests: RefCell<Vec<LlvmVerifierProcessRequest>>,
    }

    impl FakeHost {
        fn with_responses(
            responses: impl IntoIterator<
                Item = Result<LlvmVerifierProcessOutput, LlvmVerifierProcessError>,
            >,
        ) -> Self {
            Self {
                responses: RefCell::new(responses.into_iter().collect()),
                ..Self::default()
            }
        }

        fn set_override(&mut self, name: &'static str, value: impl Into<OsString>) {
            self.environment.insert(name, value.into());
        }

        fn requests(&self) -> Vec<LlvmVerifierProcessRequest> {
            self.requests.borrow().clone()
        }
    }

    impl LlvmVerifierHost for FakeHost {
        fn environment_override(&self, name: &'static str) -> Option<OsString> {
            self.environment.get(name).cloned()
        }

        fn run_process(
            &self,
            request: LlvmVerifierProcessRequest,
        ) -> Result<LlvmVerifierProcessOutput, LlvmVerifierProcessError> {
            self.requests.borrow_mut().push(request);
            self.responses
                .borrow_mut()
                .pop_front()
                .expect("missing fake verifier response")
        }
    }

    fn success(stdout: &str) -> Result<LlvmVerifierProcessOutput, LlvmVerifierProcessError> {
        Ok(LlvmVerifierProcessOutput {
            success: true,
            exit_code: Some(0),
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
        })
    }

    fn rejection(stderr: &str) -> Result<LlvmVerifierProcessOutput, LlvmVerifierProcessError> {
        Ok(LlvmVerifierProcessOutput {
            success: false,
            exit_code: Some(42),
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        })
    }

    #[test]
    fn completed_wrapper_with_inherited_pipes_still_obeys_the_process_deadline() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "aero-verifier-inherited-pipes-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&root).expect("create inherited-pipe fixture directory");
        let escaped_marker = root.join("escaped-descendant.marker");

        #[cfg(windows)]
        let program = root.join("wrapper.cmd");
        #[cfg(windows)]
        let script = r#"@echo off
if "%~1"=="child" (
  "%SystemRoot%\System32\ping.exe" 127.0.0.1 -n 2 >nul
  >"%~2" echo escaped
  exit /b 0
)
start "" /b "%ComSpec%" /d /c ""%~f0" child "%~1""
exit /b 0
"#;

        #[cfg(unix)]
        let program = root.join("wrapper");
        #[cfg(unix)]
        let script = "#!/bin/sh\nmarker=$1\n( /bin/sleep 1; printf '%s\\n' escaped > \"$marker\" ) &\nexit 0\n";

        std::fs::write(&program, script).expect("write inherited-pipe wrapper");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755))
                .expect("make inherited-pipe wrapper executable");
        }

        let started = Instant::now();
        let result = run_system_process(LlvmVerifierProcessRequest {
            program,
            arguments: vec![escaped_marker.as_os_str().to_os_string()],
            stdin: None,
            timeout: Duration::from_millis(100),
        });
        assert_eq!(result, Err(LlvmVerifierProcessError::TimedOut));
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "inherited verifier pipes outlived the configured deadline"
        );

        thread::sleep(Duration::from_millis(1_100));
        let descendant_escaped = escaped_marker.exists();
        let _ = std::fs::remove_dir_all(root);
        assert!(
            !descendant_escaped,
            "an immediate verifier descendant escaped process-tree containment"
        );
    }

    #[test]
    fn parses_supported_llvm_version_forms() {
        assert_eq!(
            parse_llvm_version("Debian LLVM version 22.1.0\n  Optimized build."),
            Some((22, "22.1.0".to_string()))
        );
        assert_eq!(
            parse_llvm_version("Ubuntu LLVM version 22.0.0git"),
            Some((22, "22.0.0".to_string()))
        );
        assert_eq!(parse_llvm_version("clang version 22.0.0"), None);
    }

    #[test]
    fn opt_override_receives_exact_arguments_and_unchanged_nonempty_stdin() {
        let mut host = FakeHost::with_responses([success("LLVM version 22.1.0"), success("")]);
        host.set_override("AERO_LLVM_OPT", "override-opt");
        host.set_override("AERO_LLVM_AS", "must-not-run-as");
        let llvm = "define i32 @main() {\n  ret i32 0\n}\n";

        let status = verify_llvm_module_with_host(
            llvm,
            LlvmVerificationMode::Required,
            &host,
            Duration::from_secs(1),
        )
        .expect("compatible override should verify");
        let identity = status.external_verifier().expect("external identity");
        assert_eq!(identity.kind, LlvmVerifierKind::Opt);
        assert_eq!(identity.path, PathBuf::from("override-opt"));
        assert_eq!(identity.version, "22.1.0");

        let requests = host.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].program, PathBuf::from("override-opt"));
        assert_eq!(requests[0].arguments, [OsString::from("--version")]);
        assert_eq!(requests[0].stdin, None);
        assert_eq!(
            requests[1].arguments,
            [
                OsString::from("-passes=verify"),
                OsString::from("-disable-output"),
                OsString::from("-")
            ]
        );
        assert_eq!(requests[1].stdin.as_deref(), Some(llvm.as_bytes()));
    }

    #[test]
    fn llvm_as_override_precedes_all_path_discovery_and_uses_exact_arguments() {
        let mut host = FakeHost::with_responses([success("LLVM version 22.0.1"), success("")]);
        host.set_override("AERO_LLVM_AS", "override-as");

        verify_llvm_module_with_host(
            "module",
            LlvmVerificationMode::Required,
            &host,
            Duration::from_secs(1),
        )
        .expect("llvm-as override should verify");

        let requests = host.requests();
        assert_eq!(requests.len(), 2);
        assert!(
            requests
                .iter()
                .all(|request| request.program == Path::new("override-as"))
        );
        assert_eq!(
            requests[1].arguments,
            [
                OsString::from("-o"),
                OsString::from("-"),
                OsString::from("-")
            ]
        );
    }

    #[test]
    fn discovery_tries_versioned_opt_then_versioned_llvm_as() {
        let host = FakeHost::with_responses([
            Err(LlvmVerifierProcessError::NotFound),
            success("LLVM version 22.0.0"),
            success(""),
        ]);

        let status = verify_llvm_module_with_host(
            "module",
            LlvmVerificationMode::Required,
            &host,
            Duration::from_secs(1),
        )
        .expect("versioned llvm-as fallback should verify");

        assert_eq!(
            status.external_verifier().map(|identity| identity.kind),
            Some(LlvmVerifierKind::LlvmAs)
        );
        let programs = host
            .requests()
            .into_iter()
            .map(|request| request.program)
            .collect::<Vec<_>>();
        assert_eq!(
            programs,
            [
                PathBuf::from("opt-22"),
                PathBuf::from("llvm-as-22"),
                PathBuf::from("llvm-as-22")
            ]
        );
    }

    #[test]
    fn wrong_version_opt_is_fatal_without_llvm_as_fallback() {
        let host = FakeHost::with_responses([success("LLVM version 21.0.0")]);

        let error = verify_llvm_module_with_host(
            "module",
            LlvmVerificationMode::Required,
            &host,
            Duration::from_secs(1),
        )
        .expect_err("wrong-version opt must be fatal");

        assert!(matches!(
            error,
            LlvmVerificationError::WrongVersion {
                required_major: 22,
                ..
            }
        ));
        assert_eq!(host.requests().len(), 1);
    }

    #[test]
    fn rejecting_opt_is_fatal_without_llvm_as_fallback() {
        let host =
            FakeHost::with_responses([success("LLVM version 22.1.0"), rejection("invalid module")]);

        let error = verify_llvm_module_with_host(
            "module",
            LlvmVerificationMode::Required,
            &host,
            Duration::from_secs(1),
        )
        .expect_err("rejecting opt must be fatal");

        assert!(matches!(
            error,
            LlvmVerificationError::Rejected {
                exit_code: Some(42),
                ..
            }
        ));
        assert_eq!(host.requests().len(), 2);
    }

    #[test]
    fn genuine_absence_is_internal_only_or_required_failure_by_mode() {
        let missing = || {
            [
                Err(LlvmVerifierProcessError::NotFound),
                Err(LlvmVerifierProcessError::NotFound),
                Err(LlvmVerifierProcessError::NotFound),
                Err(LlvmVerifierProcessError::NotFound),
            ]
        };
        let prefer_host = FakeHost::with_responses(missing());
        let status = verify_llvm_module_with_host(
            "module",
            LlvmVerificationMode::PreferExternal,
            &prefer_host,
            Duration::from_secs(1),
        )
        .expect("PreferExternal should permit genuine absence");
        assert!(matches!(
            status,
            LlvmVerificationStatus::InternalOnly { .. }
        ));

        let required_host = FakeHost::with_responses(missing());
        let error = verify_llvm_module_with_host(
            "module",
            LlvmVerificationMode::Required,
            &required_host,
            Duration::from_secs(1),
        )
        .expect_err("Required must reject genuine absence");
        assert!(matches!(
            error,
            LlvmVerificationError::NoCompatibleVerifier { .. }
        ));
    }

    #[test]
    fn authoritative_missing_override_never_falls_through() {
        let mut host = FakeHost::with_responses([Err(LlvmVerifierProcessError::NotFound)]);
        host.set_override("AERO_LLVM_OPT", "missing-explicit-opt");

        let error = verify_llvm_module_with_host(
            "module",
            LlvmVerificationMode::PreferExternal,
            &host,
            Duration::from_secs(1),
        )
        .expect_err("missing explicit override must be fatal");

        assert!(matches!(
            error,
            LlvmVerificationError::ToolNotFound {
                override_variable: Some("AERO_LLVM_OPT"),
                ..
            }
        ));
        assert_eq!(host.requests().len(), 1);
    }

    #[test]
    fn timeout_and_empty_input_keep_stable_phase_identity() {
        let host = FakeHost::with_responses([Err(LlvmVerifierProcessError::TimedOut)]);
        let error = verify_llvm_module_with_host(
            "module",
            LlvmVerificationMode::Required,
            &host,
            Duration::from_millis(25),
        )
        .expect_err("version timeout must fail");
        assert!(matches!(
            error,
            LlvmVerificationError::TimedOut {
                operation: LlvmVerifierOperation::VersionProbe,
                ..
            }
        ));
        assert!(error.to_string().starts_with("LLVM Verification Error:"));

        let unused_host = FakeHost::default();
        let empty = verify_llvm_module_with_host(
            "",
            LlvmVerificationMode::PreferExternal,
            &unused_host,
            Duration::from_secs(1),
        )
        .expect_err("empty input must fail before discovery");
        assert_eq!(empty, LlvmVerificationError::EmptyModule);
        assert!(unused_host.requests().is_empty());
    }
}
