use std::{fs, io, ptr, str::FromStr};

use libc::{c_int, id_t, pid_t};
use task_manager_types::unix_process::{CSpawnArgs, Process};

use helpers::cvt;

pub fn posix_spawn(CSpawnArgs { program, args }: CSpawnArgs) -> io::Result<pid_t> {
    use helpers::cvt_nz;
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    let mut argv: Vec<_> = args.into_iter().map(|s| s.into_raw()).collect();
    argv.push(ptr::null_mut());

    let mut pid = 0;

    struct PosixSpawnFileActions<'a>(&'a mut MaybeUninit<libc::posix_spawn_file_actions_t>);

    impl Drop for PosixSpawnFileActions<'_> {
        fn drop(&mut self) {
            unsafe {
                libc::posix_spawn_file_actions_destroy(self.0.as_mut_ptr());
            }
        }
    }

    struct PosixSpawnattr<'a>(&'a mut MaybeUninit<libc::posix_spawnattr_t>);

    impl Drop for PosixSpawnattr<'_> {
        fn drop(&mut self) {
            unsafe {
                libc::posix_spawnattr_destroy(self.0.as_mut_ptr());
            }
        }
    }

    unsafe {
        let mut attrs = MaybeUninit::uninit();
        cvt_nz(libc::posix_spawnattr_init(attrs.as_mut_ptr()))?;
        let attrs = PosixSpawnattr(&mut attrs);

        let mut file_actions = MaybeUninit::uninit();
        cvt_nz(libc::posix_spawn_file_actions_init(
            file_actions.as_mut_ptr(),
        ))?;
        let file_actions = PosixSpawnFileActions(&mut file_actions);

        let envp = [ptr::null_mut()];
        cvt_nz(libc::posix_spawnp(
            &mut pid,
            program.as_ptr(),
            file_actions.0.as_ptr(),
            attrs.0.as_ptr(),
            argv.as_ptr(),
            envp.as_ptr(),
        ))?;
        argv.pop();
        argv.iter().for_each(|s| drop(CString::from_raw(*s)));

        Ok(pid)
    }
}

pub fn process_from_pid(pid: pid_t) -> Option<Process> {
    try {
        let stat = fs::read_to_string(format!("/proc/{}/stat", pid)).ok()?;
        Process::from_str(&stat).expect("process parsing error")
    }
}

pub fn kill(pid: pid_t) -> io::Result<()> {
    cvt(unsafe { libc::kill(pid, libc::SIGKILL) }).map(drop)
}

pub fn suspend(pid: pid_t) -> io::Result<()> {
    cvt(unsafe { libc::kill(pid, libc::SIGSTOP) }).map(drop)
}

pub fn continue_(pid: pid_t) -> io::Result<()> {
    cvt(unsafe { libc::kill(pid, libc::SIGCONT) }).map(drop)
}

pub fn set_priority(pid: pid_t, prio: c_int) -> io::Result<()> {
    cvt(unsafe { libc::setpriority(libc::PRIO_PROCESS, pid as id_t, prio) }).map(drop)
}

pub fn list_processes() -> Vec<Process> {
    let my_pid = unsafe { libc::getpid() };
    fs::read_dir("/proc")
        .expect("unable to find `/proc`")
        .filter_map(|entry| try {
            let entry = entry.ok()?;
            entry.file_type().ok()?.is_dir().then(|| ())?;
            let pid = entry.file_name().to_str()?.parse().ok()?;
            let p = process_from_pid(pid)?;
            if p.ppid == my_pid && p.state == 'Z' {
                let mut status: c_int = 0;
                unsafe { libc::waitpid(p.pid, &mut status, 0) };
                return None;
            };
            p
        })
        .collect()
}

mod helpers {
    use std::io;

    pub trait IsMinusOne {
        fn is_minus_one(&self) -> bool;
    }

    macro_rules! impl_is_minus_one {
        ($($t:ident)*) => ($(impl IsMinusOne for $t {
            fn is_minus_one(&self) -> bool {
                *self == -1
            }
        })*)
    }

    impl_is_minus_one! { i8 i16 i32 i64 isize }

    pub fn cvt<T: IsMinusOne>(t: T) -> io::Result<T> {
        if t.is_minus_one() {
            Err(io::Error::last_os_error())
        } else {
            Ok(t)
        }
    }

    pub fn cvt_nz(error: libc::c_int) -> io::Result<()> {
        if error == 0 {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(error))
        }
    }
}
