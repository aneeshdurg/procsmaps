use lazy_static::lazy_static;
use libc::pid_t;
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Default, PartialEq)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub shared: bool,
    pub private: bool,
}

impl Permissions {
    pub fn read(&mut self, v: bool) {
        self.read = v;
    }
    pub fn write(&mut self, v: bool) {
        self.write = v;
    }
    pub fn execute(&mut self, v: bool) {
        self.execute = v;
    }
    pub fn shared(&mut self, v: bool) {
        self.shared = v;
    }
    pub fn private(&mut self, v: bool) {
        self.private = v;
    }

    // This isn't public because it assumes that the string passed in is a valid permission set
    fn from_str(s: &str) -> Permissions {
        let mut perms: Permissions = Default::default();
        for c in s.chars() {
            match c {
                'r' => perms.read(true),
                'w' => perms.write(true),
                'x' => perms.execute(true),
                's' => perms.shared(true),
                'p' => perms.private(true),
                _ => {
                    // ignore any other character
                }
            }
        }
        perms
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct VmFlags {
    pub rd: bool,
    pub wr: bool,
    pub ex: bool,
    pub sh: bool,
    pub mr: bool,
    pub mw: bool,
    pub me: bool,
    pub ms: bool,
    pub gd: bool,
    pub pf: bool,
    pub dw: bool,
    pub lo: bool,
    pub io: bool,
    pub sr: bool,
    pub rr: bool,
    pub dc: bool,
    pub de: bool,
    pub ac: bool,
    pub nr: bool,
    pub ht: bool,
    pub sf: bool,
    pub nl: bool,
    pub ar: bool,
    pub wf: bool,
    pub dd: bool,
    pub sd: bool,
    pub mm: bool,
    pub hg: bool,
    pub nh: bool,
    pub mg: bool,
    pub um: bool,
    pub uw: bool,
}

impl VmFlags {
    fn from_str(s: &str) -> VmFlags {
        let mut flags: VmFlags = Default::default();
        for flag in s.split(" ") {
            match flag {
                "rd" => flags.rd = true,
                "wr" => flags.wr = true,
                "ex" => flags.ex = true,
                "sh" => flags.sh = true,
                "mr" => flags.mr = true,
                "mw" => flags.mw = true,
                "me" => flags.me = true,
                "ms" => flags.ms = true,
                "gd" => flags.gd = true,
                "pf" => flags.pf = true,
                "dw" => flags.dw = true,
                "lo" => flags.lo = true,
                "io" => flags.io = true,
                "sr" => flags.sr = true,
                "rr" => flags.rr = true,
                "dc" => flags.dc = true,
                "de" => flags.de = true,
                "ac" => flags.ac = true,
                "nr" => flags.nr = true,
                "ht" => flags.ht = true,
                "sf" => flags.sf = true,
                "nl" => flags.nl = true,
                "ar" => flags.ar = true,
                "wf" => flags.wf = true,
                "dd" => flags.dd = true,
                "sd" => flags.sd = true,
                "mm" => flags.mm = true,
                "hg" => flags.hg = true,
                "nh" => flags.nh = true,
                "mg" => flags.mg = true,
                "um" => flags.um = true,
                "uw" => flags.uw = true,
                _ => {
                    // Ignore unknown flags so that if future versions of linux add additional flags
                    // the parsing won't break
                }
            }
        }
        flags
    }
}

#[test]
fn test_vmflags_from_str() {
    // No flags enabled should parse correctly
    assert_eq!(VmFlags::from_str(""), Default::default());

    // Unknown flags should be ignored
    assert_eq!(VmFlags::from_str("a b c d"), Default::default());

    // Check enabling some subset of flags
    let mut flags: VmFlags = Default::default();
    flags.rd = true;
    flags.de = true;
    flags.uw = true;
    assert_eq!(VmFlags::from_str("rd de uw"), flags);

    // Check that the order of flags doesn't matter
    let mut flags: VmFlags = Default::default();
    flags.rd = true;
    flags.de = true;
    flags.uw = true;
    assert_eq!(VmFlags::from_str("uw rd de"), flags);
}

#[derive(Debug, Default, PartialEq)]
pub struct Device {
    pub major: u64,
    pub minor: u64,
}

#[derive(Debug, Default, PartialEq)]
pub struct Mapping {
    pub start: u64,
    pub end: u64,
    pub perms: Permissions,
    pub offset: usize,
    pub device: Device,
    pub inode: u64,
    pub pathname: Option<String>,
}

lazy_static! {
    static ref RE: Regex = {
        let hex = "[0-9a-f]+";
        let address = hex;
        let permissions = "[rwsxp-]+";
        let dev_maj = hex;
        let dev_min = hex;
        let offset = hex;
        let inode = r"\d+";
        let pathname = ".*";
        let pattern = format!(
            r"^({})-({})\s+({})\s+({})\s+({}):({})\s+({})(?:\s+({}))?$",
            address, address, permissions, offset, dev_maj, dev_min, inode, pathname
        );
        Regex::new(&pattern).unwrap()
    };
}

impl Mapping {
    pub fn from_str(s: &str) -> Option<Mapping> {
        let caps = RE.captures(s.trim())?;
        let start = u64::from_str_radix(&caps[1], 16).ok()?;
        let end = u64::from_str_radix(&caps[2], 16).ok()?;
        let perms = Permissions::from_str(&caps[3]);
        let offset = usize::from_str_radix(&caps[4], 16).ok()?;

        let dev_major = u64::from_str_radix(&caps[5], 16).ok()?;
        let dev_minor = u64::from_str_radix(&caps[6], 16).ok()?;
        let device = Device {
            major: dev_major,
            minor: dev_minor,
        };
        let inode: u64 = caps[7].parse().ok()?;
        let pathname = caps.get(8).map(|m| m.as_str().to_string());
        Some(Mapping {
            start,
            end,
            perms,
            offset,
            device,
            inode,
            pathname,
        })
    }
}

#[test]
fn test_mapping_from_str() {
    assert_eq!(Mapping::from_str(""), None);
    assert_eq!(Mapping::from_str("    \n   "), None);
    let mut perms: Permissions = Default::default();
    perms.read(true);
    perms.write(true);
    perms.private(true);
    assert_eq!(
        Mapping::from_str("00e24000-011f7000 rw-p 00000000 00:00 0           [heap]"),
        Some(Mapping {
            start: 0x00e24000,
            end: 0x011f7000,
            perms,
            offset: 0,
            device: Device { major: 0, minor: 0 },
            inode: 0,
            pathname: Some("[heap]".to_string())
        })
    );

    let mut perms: Permissions = Default::default();
    perms.read(true);
    perms.write(true);
    perms.private(true);

    assert_eq!(
        Mapping::from_str("35b1a21000-35b1a22000 rw-p abcd ff:10 0"),
        Some(Mapping {
            start: 0x35b1a21000,
            end: 0x35b1a22000,
            perms,
            offset: 0xabcd,
            device: Device {
                major: 0xff,
                minor: 0x10
            },
            inode: 0,
            pathname: None
        })
    );
}

#[derive(Debug, Default, PartialEq)]
pub struct SMap {
    pub mapping: Mapping,
    pub size: u64,
    pub kernel_page_size: u64,
    pub mmu_page_size: u64,
    pub rss: u64,
    pub pss: u64,
    pub pss_dirty: u64,
    pub shared_clean: u64,
    pub shared_dirty: u64,
    pub private_clean: u64,
    pub private_dirty: u64,
    pub referenced: u64,
    pub anonymous: u64,
    pub ksm: u64,
    pub lazy_free: u64,
    pub anon_huge_pages: u64,
    pub shmem_huge_pages: u64,
    pub shmem_pmd_mapped: u64,
    pub file_pmd_mapped: u64,
    pub shared_hugetlb: u64,
    pub private_hugetlb: u64,
    pub swap: u64,
    pub swap_pss: u64,
    pub locked: u64,
    pub thp_eligible: u64,
    pub protection_key: u64,
    pub vm_flags: VmFlags,
}

impl SMap {
    pub fn from_lines(mapping: Mapping, lines: Vec<&str>) -> Option<SMap> {
        let mut res: SMap = Default::default();
        res.mapping = mapping;
        for line in lines {
            let key_value: Vec<&str> = line.split(":").collect();
            if key_value.len() != 2 {
                println!("??? {}", line);
                continue;
            }

            let key = key_value[0];
            let mut value = key_value[1].trim();
            if key == "VmFlags" {
                res.vm_flags = VmFlags::from_str(&line["VmFlags:".len()..]);
            } else {
                let mut multiplier = 1;
                if value.ends_with("kB") {
                    multiplier *= 1024;
                    value = value[..value.len() - 2].trim();
                }

                let last = value.chars().last()?;
                if !last.is_digit(10) {
                    return None;
                }

                let numeric_value = value.parse::<u64>().ok()? * multiplier;
                match key {
                    "AnonHugePages" => res.anon_huge_pages = numeric_value,
                    "Anonymous" => res.anonymous = numeric_value,
                    "FilePmdMapped" => res.file_pmd_mapped = numeric_value,
                    "KSM" => res.ksm = numeric_value,
                    "KernelPageSize" => res.kernel_page_size = numeric_value,
                    "LazyFree" => res.lazy_free = numeric_value,
                    "Locked" => res.locked = numeric_value,
                    "MMUPageSize" => res.mmu_page_size = numeric_value,
                    "Private_Clean" => res.private_clean = numeric_value,
                    "Private_Dirty" => res.private_dirty = numeric_value,
                    "Private_Hugetlb" => res.private_hugetlb = numeric_value,
                    "ProtectionKey" => res.protection_key = numeric_value,
                    "Pss" => res.pss = numeric_value,
                    "Pss_Dirty" => res.pss_dirty = numeric_value,
                    "Referenced" => res.referenced = numeric_value,
                    "Rss" => res.rss = numeric_value,
                    "Shared_Clean" => res.shared_clean = numeric_value,
                    "Shared_Dirty" => res.shared_dirty = numeric_value,
                    "Shared_Hugetlb" => res.shared_hugetlb = numeric_value,
                    "ShmemHugePages" => res.shmem_huge_pages = numeric_value,
                    "ShmemPmdMapped" => res.shmem_pmd_mapped = numeric_value,
                    "Size" => res.size = numeric_value,
                    "Swap" => res.swap = numeric_value,
                    "SwapPss" => res.swap_pss = numeric_value,
                    "THPeligible" => res.thp_eligible = numeric_value,
                    _ => {
                        // Ignore unknown keys. There's no authoritative list of all possible keys
                        // in any of the linux documentation and it may change from version to
                        // version.
                    }
                }
            }
        }
        Some(res)
    }
}

/// Returns mappings for a given pid
pub fn from_pid(pid: pid_t) -> Option<Vec<SMap>> {
    let path = format!("/proc/{}/smaps", pid);
    let mut file = File::open(path).ok()?;
    let mut input = String::new();
    file.read_to_string(&mut input).ok()?;
    from_str(&input)
}

pub fn from_path(path: &mut PathBuf) -> Option<Vec<SMap>> {
    path.push("maps");
    let mut file = File::open(path).ok()?;
    let mut input = String::new();
    file.read_to_string(&mut input).ok()?;
    from_str(&input)
}

pub fn from_str(raw: &str) -> Option<Vec<SMap>> {
    let input = String::from(raw);
    let mut res: Vec<SMap> = Vec::new();
    let lines: Vec<&str> = input.split("\n").collect();
    let mut i = 0;
    while i < lines.len() {
        let mut smap_lines: Vec<&str> = Vec::new();
        if let Some(map) = Mapping::from_str(lines[i]) {
            i += 1;
            while i < lines.len() && Mapping::from_str(lines[i]).is_none() {
                smap_lines.push(lines[i]);
                i += 1;
            }
            res.push(SMap::from_lines(map, smap_lines)?);
        } else {
            return None;
        }
    }
    Some(res)
}

#[test]
fn test_smaps_from_str() {
    let txt = "\
6036e81d0000-6036e84bd000 rw-p 00000000 00:00 0                          [heap]
Size:               2996 kB
KernelPageSize:        4 kB
MMUPageSize:           4 kB
Rss:                2796 kB
Pss:                2796 kB
Pss_Dirty:          2796 kB
Shared_Clean:          0 kB
Shared_Dirty:          0 kB
Private_Clean:         0 kB
Private_Dirty:      2796 kB
Referenced:         2796 kB
Anonymous:          2796 kB
KSM:                   0 kB
LazyFree:              0 kB
AnonHugePages:         0 kB
ShmemPmdMapped:        0 kB
FilePmdMapped:         0 kB
Shared_Hugetlb:        0 kB
Private_Hugetlb:       0 kB
Swap:                  0 kB
SwapPss:               0 kB
Locked:                0 kB
THPeligible:           1
ProtectionKey:         0
VmFlags: rd wr mr mw me ac sd 
76be15f03000-76be160ed000 rw-p 00000000 00:00 0 
Rss:                1960 kB
Swap:                  0 kB
Locked:                0 kB
FilePmdMapped:         0 kB
Referenced:         1960 kB
Private_Dirty:      1960 kB
KernelPageSize:        4 kB
Private_Clean:         0 kB
ProtectionKey:         0
ShmemPmdMapped:        0 kB
Anonymous:          1960 kB
Size:               1960 kB
Shared_Dirty:          0 kB
Shared_Clean:          0 kB
Pss:                1960 kB
MMUPageSize:           4 kB
VmFlags: 
";

    let mut perms: Permissions = Default::default();
    perms.read = true;
    perms.write = true;
    perms.private = true;

    assert_eq!(
        from_str(txt),
        Some(vec![
            SMap {
                mapping: Mapping {
                    start: 0x6036e81d0000,
                    end: 0x6036e84bd000,
                    perms: Permissions { ..perms },
                    offset: 0,
                    inode: 0,
                    device: Default::default(),
                    pathname: Some("[heap]".to_string())
                },
                size: 2996 * 1024,
                kernel_page_size: 4 * 1024,
                mmu_page_size: 4 * 1024,
                rss: 2796 * 1024,
                pss: 2796 * 1024,
                pss_dirty: 2796 * 1024,
                private_dirty: 2796 * 1024,
                referenced: 2796 * 1024,
                anonymous: 2796 * 1024,
                thp_eligible: 1,
                vm_flags: VmFlags {
                    rd: true,
                    wr: true,
                    mr: true,
                    mw: true,
                    me: true,
                    ac: true,
                    sd: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            SMap {
                mapping: Mapping {
                    start: 0x76be15f03000,
                    end: 0x76be160ed000,
                    perms: Permissions { ..perms },
                    offset: 0,
                    inode: 0,
                    device: Default::default(),
                    pathname: None
                },
                rss: 1960 * 1024,
                referenced: 1960 * 1024,
                private_dirty: 1960 * 1024,
                kernel_page_size: 4 * 1024,
                anonymous: 1960 * 1024,
                size: 1960 * 1024,
                pss: 1960 * 1024,
                mmu_page_size: 4 * 1024,
                ..Default::default()
            },
        ])
    );
}
