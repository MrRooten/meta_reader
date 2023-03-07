#![allow(unused)]

use std::{collections::HashMap, fs};

use bytes::Bytes;
use meta_reader::{
    file_struct::{
        elf::{elf32::ELF32, elf64::ELF64},
        ext4::{Ext4, FileType, Inode},
        ntfs::Ntfs,
    },
    modules::{ext4::Ext4Module, ntfs::NtfsModule},
    utils::{file::filesize_to_human_string, funcs::i_to_m},
};

static mut PASSWD: Option<HashMap<u16, String>> = None;

pub fn get_username(uid: u16) -> String {
    unsafe {
        if PASSWD.is_none() {
            let content = match fs::read_to_string("/etc/passwd") {
                Ok(o) => o,
                Err(e) => {
                    return "".to_string();
                }
            };

            let iter = content.split("\n");
            let mut map: HashMap<u16, String> = HashMap::new();
            for i in iter {
                let vs = i.split(":").collect::<Vec<&str>>();
                if vs.len() < 3 {
                    continue;
                }
                map.insert(vs[2].parse::<u16>().unwrap(), vs[0].to_string());
            }
            PASSWD = Some(map);
        }
        let s = match &PASSWD {
            Some(o) => o,
            None => {
                return "".to_string();
            }
        };

        s.get(&uid).unwrap_or(&"Unknown".to_string()).to_string()
    }
}

fn main() {
    sigpipe::reset();
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() == 1 {
        println!("{} ${{file_format}} ${{function}} ${{options}}", args[0]);
        println!("example:");
        println!(
            "\t{} ext4 ./test.img find_deleted_files path=/test_dir/",
            args[0]
        );
        return;
    }

    if args[1].eq("ext4") {
        if args.len() <= 3 {
            println!("support function:");
            println!("\t${{ext4_file}} list_deleted_files path=${{target_dir}}");
            println!("\t${{ext4_file}} list_recoverable path=${{target_dir}}");
            println!(
                "\t${{ext4_file}} journal_recover_file inode=${{inode_id}},out_file=${{out_file}}"
            );
            println!("\t${{ext4_file}} list_files path=${{target_dir}}");
            println!("\t${{ext4_file}} read_file path=${{path}}");
            println!("\t${{ext4_file}} search_deleted_files path=${{path_dir}}");
            return;
        }
        let img = &args[2];
        let mut module = Ext4Module::new(img).unwrap();

        let function = &args[3];
        let mut _f_args = HashMap::new();
        if args.len() >= 5 {
            let options = args[4].split(",");
            for option in options {
                let kv = option.split("=");
                let kv = kv.collect::<Vec<&str>>();
                _f_args.insert(kv[0].trim().to_string(), kv[1].trim().to_string());
            }
        }

        if function.eq("list_deleted_files") {
            let dirs = module
                .list_deleted_files(_f_args, |id, inode, name1, name2, ext4| {
                    println!("{}", name1);
                    //println!("\tname2: {}", name2);
                    println!("\tinode id: {}", id);
                    if ext4.is_inode_taken(id) == false {
                        if let Some(inode) = inode {
                            println!("\tatime: {}", inode.get_atime().to_string());
                            println!("\tctime: {}", inode.get_ctime().to_string());
                            println!("\tmtime: {}", inode.get_mtime().to_string());
                            println!("\tdtime: {}", inode.get_dtime().to_string());
                            println!("\tbirth time: {}", inode.get_birth().to_string());
                            println!(
                                "\tfile size: {}",
                                filesize_to_human_string(inode.get_size() as usize)
                            );
                            println!("\tuid: {}", inode.get_uid());
                            println!(
                                "\tusername: {} \t-> #Base on /etc/passwd",
                                get_username(inode.get_uid())
                            );
                        }
                    }
                    println!();
                })
                .unwrap();
        } else if function.eq("journal_recover_file") {
            module.journal_recover_file(_f_args).unwrap();
        } else if function.eq("list_files") {
            let dirs = module.list_files(_f_args).unwrap();
            for dir in dirs {
                println!("{}", dir.get_name());
            }
        } else if function.eq("read_file") {
            module.read_file(_f_args).unwrap();
        } else if function.eq("list_recoverable") {
            let mut last: String = String::new();
            let inodes = module
                .list_recoverable_inodes(_f_args, |id, inode, name, ext4| {
                    let mut output = String::new();
                    output.push_str(&format!("{}\n", name));
                    //output.push_str(&format!("\tname2: {}\n", name2));
                    output.push_str(&format!("\tinode id: {}\n", id));
                    output.push_str(&format!("\tatime: {}\n", inode.get_atime().to_string()));
                    output.push_str(&format!("\tctime: {}\n", inode.get_ctime().to_string()));
                    output.push_str(&format!("\tmtime: {}\n", inode.get_mtime().to_string()));
                    output.push_str(&format!("\tdtime: {}\n", inode.get_dtime().to_string()));
                    output.push_str(&format!(
                        "\tbirth time: {}\n",
                        inode.get_birth().to_string()
                    ));
                    output.push_str(&format!(
                        "\tfile size: {}\n",
                        filesize_to_human_string(inode.get_size() as usize)
                    ));
                    output.push_str(&format!("\tuid: {}\n", inode.get_uid()));
                    output.push_str(&format!(
                        "\tusername: {} \t-> #Base on /etc/passwd\n",
                        get_username(inode.get_uid())
                    ));
                    if last.eq(&output) {
                        return;
                    } else {
                        println!("{}", output);
                        last = output;
                    }
                })
                .unwrap();
        } else if function.eq("search_deleted_files") {
            let files = module
                .search_deleted_files(_f_args, |id, inode, name, name2, ext4| {
                    println!("{}", name);
                    //println!("\tname2: {}", name2);
                    println!("\tinode id: {}", id);
                    if ext4.is_inode_taken(id) == false {
                        println!("\tatime: {}", inode.get_atime().to_string());
                        println!("\tctime: {}", inode.get_ctime().to_string());
                        println!("\tmtime: {}", inode.get_mtime().to_string());
                        println!("\tdtime: {}", inode.get_dtime().to_string());
                        println!("\tbirth time: {}", inode.get_birth().to_string());
                        println!(
                            "\tfile size: {}",
                            filesize_to_human_string(inode.get_size() as usize)
                        );
                        println!("\tuid: {}", inode.get_uid());
                        println!(
                            "\tusername: {} \t-> #Base on /etc/passwd",
                            get_username(inode.get_uid())
                        );
                    }
                })
                .unwrap();
            println!();
        } else if function.eq("search_recoverable_files") {
            let mut last: String = String::new();
            let files = module
                .search_recoverable_files(_f_args, |id, inode, name, name2, ext4| {
                    let mut output = String::new();
                    output.push_str(&format!("{}\n", name));
                    //output.push_str(&format!("\tname2: {}\n", name2));
                    output.push_str(&format!("\tinode id: {}\n", id));
                    output.push_str(&format!("\tatime: {}\n", inode.get_atime().to_string()));
                    output.push_str(&format!("\tctime: {}\n", inode.get_ctime().to_string()));
                    output.push_str(&format!("\tmtime: {}\n", inode.get_mtime().to_string()));
                    output.push_str(&format!("\tdtime: {}\n", inode.get_dtime().to_string()));
                    output.push_str(&format!(
                        "\tbirth time: {}\n",
                        inode.get_birth().to_string()
                    ));
                    output.push_str(&format!(
                        "\tfile size: {}\n",
                        filesize_to_human_string(inode.get_size() as usize)
                    ));
                    output.push_str(&format!("\tuid: {}\n", inode.get_uid()));
                    output.push_str(&format!(
                        "\tusername: {} \t-> #Base on /etc/passwd\n",
                        get_username(inode.get_uid())
                    ));
                    if last.eq(&output) {
                        return;
                    } else {
                        println!("{}", output);
                        last = output;
                    }
                })
                .unwrap();
            println!();
        }
    } else if args[1].eq("ntfs") {
        if args.len() <= 3 {
            println!("support function:");
            println!("\t${{ntfs_file}} stat path=${{target_dir}}");
            println!("\t${{ntfs_file}} deleted_files path=${{target_dir}}");
            println!("\t${{ntfs_file}} search_disk encode=${{default:hex,base64,file,string,regex,regex_bytes,regex_utf16}},to_search=${{value}}");
            return;
        }
        let img = &args[2];
        let mut module = NtfsModule::new(img).unwrap();

        let function = &args[3];
        let mut _f_args = HashMap::new();
        if args.len() >= 5 {
            let options = args[4].split(",");
            for option in options {
                let kv = option.split("=");
                let kv = kv.collect::<Vec<&str>>();
                _f_args.insert(kv[0].trim().to_string(), kv[1].trim().to_string());
            }
        }

        if function.eq("stat") {
            module.stat(_f_args).unwrap();
        } else if function.eq("deleted_files") {
            module.deleted_files(_f_args).unwrap();
        } else if function.eq("search_disk") {
            if let Err(e) = module.search_disk(_f_args) {
                println!("[Error]:{}", e);
            }
        }
    } else if args[1].eq("test") {
        let mut ntfs = Ntfs::open("\\\\.\\C:").unwrap();
        let file = ntfs.get_mft_by_path("\\$Extend").unwrap();
        let subs = file.get_sub_files().unwrap();
        for sub in subs {
            println!("{} {}", sub.get_index(), sub.get_name());
        }
    }
    return;
}
