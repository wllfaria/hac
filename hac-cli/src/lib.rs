use std::path::Path;

use clap::Parser;

pub enum RuntimeBehavior {
    PrintConfigPath,
    PrintDataPath,
    DumpDefaultConfig,
    Run,
}

#[derive(Parser)]
pub struct Cli {
    /// prints the directory in which the config file is being loaded from
    #[arg(long)]
    config_dir: bool,
    /// dumps the default configuration to stdout.
    #[arg(long)]
    config_dump: bool,
    /// prints the directory in which the collections are being stored
    #[arg(long)]
    data_dir: bool,
}

impl Cli {
    pub fn parse_args() -> RuntimeBehavior {
        let args = Cli::parse();

        if args.config_dir {
            return RuntimeBehavior::PrintConfigPath;
        }
        if args.data_dir {
            return RuntimeBehavior::PrintDataPath;
        }
        if args.config_dump {
            return RuntimeBehavior::DumpDefaultConfig;
        }

        RuntimeBehavior::Run
    }

    pub fn print_data_path<P>(data_path: P)
    where
        P: AsRef<Path>,
    {
        println!(
            "collections are being stored at: {}",
            data_path.as_ref().to_string_lossy()
        );
        println!("you can change this on the configuration file by specifying `collections_dir`");
    }

    pub fn print_config_path<P>(maybe_path: Option<P>, usual_path: P)
    where
        P: AsRef<Path>,
    {
        match maybe_path {
            Some(config_dir) => {
                println!(
                    "config is being loaded from: {}",
                    config_dir.as_ref().to_string_lossy()
                );
            }
            None => {
                println!("no config file was found, the default one is being used");
                println!("the usual path for the configuration file is at:\n");
                println!("{}", usual_path.as_ref().to_string_lossy());
            }
        }
    }

    pub fn print_default_config(config_as_str: &str) {
        println!("{}", config_as_str)
    }
}
