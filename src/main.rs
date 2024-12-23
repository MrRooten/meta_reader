#![allow(unused)]
use std::{collections::HashMap, fs, path::PathBuf, ptr::addr_of};

use clap::{Args, Parser, Subcommand};
use colored::{Colorize, ColoredString};
use meta_reader::{
    modules::{ext4::Ext4Module, ntfs::NtfsModule}, utils::file::filesize_to_human_string
};

#[derive(Subcommand, Debug)]
enum Commands {
    /// ntfs
    Ntfs(Ntfs),
    Ext4(Ext4),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Args)]
pub struct Ntfs {
    #[arg(short, long)]
    function: String,

    #[arg(short, long)]
    device: String,

    #[arg(short, long)]
    options: Option<String>
}

#[derive(Debug, Args)]
pub struct Ext4 {
    #[arg(short, long)]
    function: String,

    #[arg(short, long)]
    device: String,

    #[arg(short, long)]
    options: Option<String>
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
}
fn main() {
    sigpipe::reset();

    let mut _f_args = HashMap::new();

    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Ntfs(ntfs) => {
            let img = &ntfs.device;
            let options = ntfs.options.as_ref().map(|x| {
                let options = x.split(',');
                for option in options {
                    let kv = option.split('=');
                    let kv = kv.collect::<Vec<&str>>();
                    _f_args.insert(kv[0].trim().to_string(), kv[1].trim().to_string());
                }
            });
            
            let mut module = NtfsModule::new(img).unwrap();
            let function = &ntfs.function;

            if function.eq("stat") {
                module.stat(_f_args).unwrap();
            } else if function.eq("deleted_files") {
                module.deleted_files(_f_args).unwrap();
            } else if function.eq("search_disk") {
                if let Err(e) = module.search_disk(_f_args) {
                    println!("[Error]:{}", e);
                }
            } else if function.eq("search_usn") {
                module.search_usn(_f_args).unwrap();
            }
        },
        Commands::Ext4(ext4) => {
            let img = &ext4.device;
            if let Some(x) = ext4.options.as_ref() {
                let options = x.split(',');
                for option in options {
                    let kv = option.split('=');
                    let kv = kv.collect::<Vec<&str>>();
                    _f_args.insert(kv[0].trim().to_string(), kv[1].trim().to_string());
                }
            }
            
            let mut module = Ext4Module::new(img).unwrap();
            let function = &ext4.function;
            if function.eq("list_deleted_files") {
                let _dirs = module
                    .list_deleted_files(_f_args, |id, inode, name1, name2, ext4| {
                        println!("{}", name1);
                        //println!("\tname2: {}", name2);
                        println!("\tinode id: {}", id);
                        if !ext4.is_inode_taken(id) {
                            if let Some(inode) = inode {
                                println!("\tatime: {}", inode.get_atime());
                                println!("\tctime: {}", inode.get_ctime());
                                println!("\tmtime: {}", inode.get_mtime());
                                println!("\tdtime: {}", inode.get_dtime());
                                println!("\tbirth time: {}", inode.get_birth());
                                println!(
                                    "\tfile size: {}",
                                    filesize_to_human_string(inode.get_size() as usize)
                                );
                                println!("\tuid: {}", inode.get_uid());
                                println!(
                                    "\tuid: {} \t-> #Base on /etc/passwd",
                                    inode.get_uid()
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
                        output.push_str(&format!("\tatime: {}\n", inode.get_atime()));
                        output.push_str(&format!("\tctime: {}\n", inode.get_ctime()));
                        output.push_str(&format!("\tmtime: {}\n", inode.get_mtime()));
                        output.push_str(&format!("\tdtime: {}\n", inode.get_dtime()));
                        output.push_str(&format!(
                            "\tbirth time: {}\n",
                            inode.get_birth()
                        ));
                        output.push_str(&format!(
                            "\tfile size: {}\n",
                            filesize_to_human_string(inode.get_size() as usize)
                        ));
                        output.push_str(&format!("\tuid: {}\n", inode.get_uid()));
                        output.push_str(&format!(
                            "\tuid: {} \t-> #Base on /etc/passwd\n",
                            inode.get_uid()
                        ));
                        if last.eq(&output) {
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
                        if !ext4.is_inode_taken(id) {
                            println!("\tatime: {}", inode.get_atime());
                            println!("\tctime: {}", inode.get_ctime());
                            println!("\tmtime: {}", inode.get_mtime());
                            println!("\tdtime: {}", inode.get_dtime());
                            println!("\tbirth time: {}", inode.get_birth());
                            println!(
                                "\tfile size: {}",
                                filesize_to_human_string(inode.get_size() as usize)
                            );
                            println!("\tuid: {}", inode.get_uid());
                            println!(
                                "\tuid: {} \t-> #Base on /etc/passwd",
                                inode.get_uid()
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
                        output.push_str(&format!("\tatime: {}\n", inode.get_atime()));
                        output.push_str(&format!("\tctime: {}\n", inode.get_ctime()));
                        output.push_str(&format!("\tmtime: {}\n", inode.get_mtime()));
                        output.push_str(&format!("\tdtime: {}\n", inode.get_dtime()));
                        output.push_str(&format!(
                            "\tbirth time: {}\n",
                            inode.get_birth()
                        ));
                        output.push_str(&format!(
                            "\tfile size: {}\n",
                            filesize_to_human_string(inode.get_size() as usize)
                        ));
                        output.push_str(&format!("\tuid: {}\n", inode.get_uid()));
                        output.push_str(&format!(
                            "\tuid: {} \t-> #Base on /etc/passwd\n",
                            inode.get_uid()
                        ));
                        if last.eq(&output) {
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
        },
    }

}
