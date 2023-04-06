#![allow(unused)]

use std::{collections::HashMap, fs, path::PathBuf};

use bytes::Bytes;
use chrono::{Utc, TimeZone};
use colored::{Colorize, ColoredString};
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
fn help(program_name: &ColoredString) {
    println!("{} ${{alias}} [${{option}}[${{option}}..]]", program_name);
    println!(
        "{} {} ${{ntfs_file}} ${{ntfs_function}} ${{option}}",
        program_name,
        "ntfs".bright_red()
    );
    println!(
        "{} {} ${{ext4_file}} ${{ext4_function}} ${{option}}",
        program_name,
        "ext4".bright_red()
    );
    println!("{} {}", program_name, "alias".bright_red());
    println!("\t: Show aliases");
    println!("Examples:");
    println!(
        "\t{} ntfs \\\\.\\C: search_disk encoding=regex,to_search=http://.*/",
        program_name
    );
    println!("\t{} nsdr \\\\.\\C: http://.*/", program_name);
    println!("\t{} alias", program_name);
    return;
}
fn main() {
    sigpipe::reset();
    let mut args = std::env::args().collect::<Vec<String>>();
    let program_path = PathBuf::from(&args[0]); // 获取全路径
    let program_name = program_path.file_name().unwrap().to_str().unwrap().green();
    if args.len() == 1 {
        help(&program_name);
        return;
    }

    if args[1].eq("help") {
        help(&program_name);
        return;
    }
    let mut _f_args = HashMap::new();
    let mut function = String::new();
    if args[1].eq("nsds") {
        if args.len() < 4 {
            println!(
                "nsds alias -> [ntfs ${{img}} search_disk encode=string,to_search=${{str_value}}]"
            );
            println!("nsds ${{img}} ${{string}}");
            return;
        }
        args[1] = "ntfs".to_string();
        function = "search_disk".to_string();
        _f_args.insert("encode".to_string(), "string".to_string());
        _f_args.insert("to_search".to_string(), args[3].to_string());
    } else if args[1].eq("nsdr") {
        if args.len() < 4 {
            println!("nsdr alias -> [ntfs ${{img}} search_disk encode=regex,to_search=${{regex_pattern}}]");
            println!("{} nsdr ${{img}} ${{regex_pattern}}", program_name);
            return;
        }
        args[1] = "ntfs".to_string();
        function = "search_disk".to_string();
        _f_args.insert("encode".to_string(), "regex".to_string());
        _f_args.insert("to_search".to_string(), args[3].to_string());
    } else if args[1].eq("nsdh") {
        if args.len() < 4 {
            println!(
                "{} alias -> [ntfs ${{img}} search_disk encode=hex,to_search=${{hex_string}}]",
                "nsdh".bright_red()
            );
            println!("{} nsdh ${{img}} ${{hex_string}}", program_name);
            return;
        }
        args[1] = "ntfs".to_string();
        function = "search_disk".to_string();
        _f_args.insert("encode".to_string(), "hex".to_string());
        _f_args.insert("to_search".to_string(), args[3].to_string());
    } else if args[1].eq("nsdb") {
        if args.len() < 4 {
            println!("nsdb alias -> [ntfs ${{img}} search_disk encode=hex,to_search=${{base64}}]");
            println!("{} nsdb ${{img}} ${{base64}}", program_name);
            return;
        }
        args[1] = "ntfs".to_string();
        function = "search_disk".to_string();
        _f_args.insert("encode".to_string(), "base64".to_string());
        _f_args.insert("to_search".to_string(), args[3].to_string());
    } else if args[1].eq("e4ld") {
        if args.len() < 4 {
            println!(
                "e4ld alias -> [ext4 ${{ext4_file}} list_deleted_files path=${{target_dir}}]"
            );
            println!("{} e4ld ${{img}} ${{path_dir}}", program_name);
            return;
        }
        args[1] = "ext4".to_string();
        function = "list_deleted_files".to_string();
        _f_args.insert("path".to_string(), args[3].to_string());
    } else if args[1].eq("e4lr") {
        if args.len() < 4 {
            println!("e4lr alias -> [ext4 ${{ext4_file}} list_recoverable path=${{target_dir}}]");
            println!("{} e4lr ${{img}} ${{path_dir}}", program_name);
            return;
        }
        args[1] = "ext4".to_string();
        function = "list_recoverable".to_string();
        _f_args.insert("path".to_string(), args[3].to_string());
    } else if args[1].eq("e4jr") {
        if args.len() < 5 {
            println!("e4jr alias -> [ext4 ${{ext4_file}} journal_recover_file inode=${{inode_id}},out_file=${{out_file}}]");
            println!("{} e4jr ${{img}} ${{inode}} ${{outfile}}", program_name);
            return;
        }
        args[1] = "ext4".to_string();
        function = "journal_recover_file".to_string();
        _f_args.insert("inode".to_string(), args[3].to_string());
        _f_args.insert("out_file".to_string(), args[4].to_string());
    } else if args[1].eq("e4cat") {
        if args.len() < 4 {
            println!("e4cat alias -> [ext4 ${{ext4_file}} list_recoverable path=${{target_dir}}]");
            println!("{} e4cat ${{img}} ${{path_dir}}", program_name);
            return;
        }
        args[1] = "ext4".to_string();
        function = "read_file".to_string();
        _f_args.insert("path".to_string(), args[3].to_string());
    } else if args[1].eq("e4sdf") {
        if args.len() < 4 {
            println!("e4sdf alias -> [ext4 ${{ext4_file}} list_recoverable path=${{target_dir}}]");
            println!("{} e4sdf ${{img}} ${{path_dir}}", program_name);
            return;
        }
        args[1] = "ext4".to_string();
        function = "search_deleted_files".to_string();
        _f_args.insert("path".to_string(), args[3].to_string());
    } else if args[1].eq("alias") {
        println!("{} {} \n\t-> [ntfs ${{ntfs_file}} search_disk encode=regex,to_search=${{regex_pattern}}]",program_name, "nsdr".bright_red());
        println!(
            "{} {} \n\t-> [ntfs ${{ntfs_file}} search_disk encode=hex,to_search=${{hex_string}}]",
            program_name,
            "nsdh".bright_red()
        );
        println!(
            "{} {} \n\t-> [ntfs ${{ntfs_file}} search_disk encode=hex,to_search=${{base64}}]",
            program_name,
            "nsdb".bright_red()
        );
        println!(
            "{} {} \n\t-> [ext4 ${{ext4_file}} list_deleted_files path=${{target_dir}}]",
            program_name,
            "e4ld".bright_red()
        );
        println!(
            "{} {} \n\t-> [ext4 ${{ext4_file}} list_recoverable path=${{target_dir}}]",
            program_name,
            "e4lr".bright_red()
        );
        println!("{} {} \n\t-> [ext4 ${{ext4_file}} journal_recover_file inode=${{inode_id}},out_file=${{out_file}}]", program_name, "e4jr".bright_red());
        println!(
            "{} {} \n\t-> [ext4 ${{ext4_file}} list_recoverable path=${{target_dir}}]",
            program_name,
            "e4cat".bright_red()
        );
        println!(
            "{} {} \n\t-> [ext4 ${{ext4_file}} list_recoverable path=${{target_dir}}]",
            program_name,
            "e4sdf".bright_red()
        );
    } else if args[1].eq("test") {

    } else {
        if args.len() < 4 {
            return ;
        }
        function = (&args[3]).to_string();
        if args.len() >= 5 {
            let options = args[4].split(",");
            for option in options {
                let kv = option.split("=");
                let kv = kv.collect::<Vec<&str>>();
                _f_args.insert(kv[0].trim().to_string(), kv[1].trim().to_string());
            }
        }
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
        } else if function.eq("search_disk") {
            module.search_disk(_f_args);
        }
    } else if args[1].eq("ntfs") {
        if args.len() <= 3 {
            println!("support function:");
            println!("\t${{ntfs_file}} stat path=${{target_dir}}");
            println!("\t${{ntfs_file}} deleted_files path=${{target_dir}}");
            println!("\t${{ntfs_file}} search_disk encode=${{default:hex,base64,file,string,regex,regex_bytes,regex_utf16}},to_search=${{value}}");
            println!("\t${{ntfs_file}} dump_usn 'path=${{path}}'");
            return;
        }
        let img = &args[2];
        let mut module = NtfsModule::new(img).unwrap();

        if function.eq("stat") {
            module.stat(_f_args).unwrap();
        } else if function.eq("deleted_files") {
            module.deleted_files(_f_args).unwrap();
        } else if function.eq("search_disk") {
            if let Err(e) = module.search_disk(_f_args) {
                println!("[Error]:{}", e);
            }
        } else if function.eq("dump_usn") {
            module.dump_usn(_f_args).unwrap();
        }
    } else if args[1].eq("test") {
        let mut ntfs = Ntfs::open("\\\\.\\C:").unwrap();
        //let b = ntfs.get_mft_entry_by_index(516778);
        let t = 1680078256;

        let datetime_utc = match Utc.timestamp_opt(t as i64, 0) {
            chrono::LocalResult::None => {return ;},
            chrono::LocalResult::Single(s) => s,
            chrono::LocalResult::Ambiguous(_, _) => {return ;},
        };

        println!("{}", datetime_utc.to_string());
    } else if args[1].eq("alias") {
    } else {
        println!(
            "Not support {}, try {} command",
            args[1],
            "help".bright_red()
        );
    }
    return;
}
