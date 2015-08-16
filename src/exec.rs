use std::io::Write;
use std::fmt;
use errno::{Errno, errno};
use std::fs::File;
use std::ffi::CString;
use libc;

pub struct Error {
    pub code: Option<Errno>,
    pub msg: String
}

impl Error {
    fn new(msg: &str) -> Error {
        let err = errno();
        let code = if err.0 == libc::EACCES {
            None
        } else {
            Some(err)
        };
        Error{code: code, msg: msg.to_string()}
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(code) = self.code {
            write!(f, "Error<{}: {}>", code, self.msg)
        } else {
            write!(f, "Error<{}>", self.msg)
        }
    }
}

#[derive(Default)]
pub struct Executor {
    cmd: Vec<String>,
    name: Option<String>,
    group: Option<String>,
    log_path: Option<String>,
    pid_path: Option<String>,
    pid: u32
}

impl Executor {
    pub fn new(cmd: &Vec<String>, shell: bool) -> Executor {
        Executor{cmd: if shell {
                          vec!("sh".to_string(), "-c".to_string(), cmd.connect(" "))
                      } else {
                          cmd.clone()
                      }, ..Default::default()}
    }

    pub fn with_name_group(mut self, name: Option<String>, group: Option<String>) -> Executor {
        self.name = name;
        self.group = group;
        self
    }

    pub fn with_log(mut self, log: Option<String>) -> Executor {
        self.log_path = log;
        self
    }

    pub fn with_pid(mut self, pid: Option<String>) -> Executor {
        self.pid_path = pid;
        self
    }

    #[cfg(target_os = "linux")]
    pub fn run(&mut self) -> Result<u32, Error> {
        // converting CString to ensure 0-terminated
        let mut tmp_argv: Vec<CString> = Vec::with_capacity(self.cmd.len());
        tmp_argv.extend(self.cmd.iter().map(|c| CString::new(c.as_bytes()).ok().unwrap()));
        let mut argv: Vec<*const libc::c_char> = Vec::with_capacity(self.cmd.len()+1);
        argv.extend(tmp_argv.iter().map(|c| c.as_ptr()));
        argv.push(0 as *const libc::c_char);

        let pid = unsafe{libc::fork()};
        if pid < 0 {
            return Err(Error::new("fork fail"));
        } else if pid == 0 {
            unsafe {
                if libc::setsid() < 0 {
                    panic!("setsid fail: {}", errno());
                }
                if libc::close(0) < 0 {
                    panic!("close stdin fail: {}", errno());
                }
                let log_fd = match self.log_path {
                    Some(ref path) => {
                        let p = CString::new(path.as_bytes()).ok().unwrap();
                        let fd = libc::open(p.as_ptr(),
                                            libc::O_APPEND|libc::O_CREAT|libc::O_WRONLY, 0o644);
                        if fd < 0 {
                            panic!("open log file fail: {}", errno());
                        }
                        if libc::dup2(fd, 1) < 0 {
                            panic!("dup stdout fail: {}", errno());
                        }
                        if libc::dup2(fd, 2) < 0 {
                            panic!("dup stderr fail: {}", errno());
                        }
                        fd
                    },
                    None => 0
                };

                if libc::execvp(argv[0], argv.as_mut_ptr()) < 0 {
                    let err = errno();
                    if log_fd > 0 {
                        libc::close(log_fd);
                    }
                    panic!("execvp fail: {}", err);
                }
            }
        }
        if let Some(ref pid_path) = self.pid_path {
            match File::create(pid_path) {
                Ok(ref mut file) => {
                    if let Err(e) = file.write_fmt(format_args!("{}", pid)) {
                        panic!("write pid fail: {}", e);
                    }
                },
                Err(e) => {
                    panic!("open pid file fail: {}", e);
                }
            }
        }
        self.pid = pid as u32;
        Ok(pid as u32)
    }
}
