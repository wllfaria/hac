use std::path::Path;

use clap::Parser;

/// How the runtime should behave. Dictated by the flags provided to  `Cli`
#[derive(Debug, PartialEq)]
pub enum RuntimeBehavior {
    /// will print all directories `HAC` is looking for a configuration file
    /// that means. Will print wether or not HAC_CONFIG is set, and if so where
    /// it points to, will print `$XDG_CONFIG_HOME`, and also `$HOME/.config`
    PrintConfigPath,
    /// will print all directories `HAC` is looking for collections, this will
    /// also print the path specified on the configuration file, if any.
    PrintDataPath,
    /// will dump the default configuration to stdout instead of running the
    /// application.
    DumpDefaultConfig,
    /// will run the application with all disk-synchronization disabled. That
    /// means `HAC` wont't save any files or changes to collection to disk.
    DryRun,
    /// the default running behavior of the application, this is the default
    /// behavior for `HAC`.
    Run,
}

#[derive(Parser, Debug)]
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
    /// wether or not we should sync changes to the disk, when --dry-run is
    /// specified, no collection, request, or anything will be saved to disk.
    #[arg(long)]
    dry_run: bool,
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
        if args.dry_run {
            return RuntimeBehavior::DryRun;
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
