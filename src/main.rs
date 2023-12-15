use std::process::Stdio;

use anyhow::Result;
use clap::Parser;
use tokio::process::Command;

#[derive(Debug, Parser)]
struct Args {
    /// `--cd` option passed to WSL.
    #[clap(long = "cd")]
    cd: Option<String>,

    /// `--distribution` option passed to WSL.
    #[clap(short = 'd', long = "distribution")]
    distribution: Option<String>,

    /// `--user` option passed to WSL.
    #[clap(short = 'u', long = "user")]
    user: Option<String>,

    /// `--system` option passed to WSL.
    #[clap(long = "system")]
    system: bool,

    #[clap(subcommand)]
    cmd: Cmd,
}

impl Args {
    fn wsl_args(&self) -> impl '_ + IntoIterator<Item = &str> {
        let mut args = Vec::new();

        if let Some(cd) = &self.cd {
            args.extend(["--cd", cd]);
        }

        if let Some(distribution) = &self.distribution {
            args.extend(["--distribution", distribution]);
        }

        if let Some(user) = &self.user {
            args.extend(["--user", user]);
        }

        if self.system {
            args.push("--system");
        }

        args
    }
}

#[derive(Debug, Parser)]
enum Cmd {
    Get,
    Store,
    Erase,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    let credential_subcmd = match args.cmd {
        Cmd::Get => "fill",
        Cmd::Store => "approve",
        Cmd::Erase => "reject",
    };

    let mut process = Command::new("wsl.exe")
        .args(args.wsl_args())
        .args(["-d", "Ubuntu", "--", "git", "credential", credential_subcmd])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdin = process.stdin.take();
    let stdout = process.stdout.take();

    let stdin_future = {
        async move {
            if let Some(mut stdin) = stdin {
                Ok(Some({
                    tokio::io::copy(&mut tokio::io::stdin(), &mut stdin).await?
                }))
            } else {
                Ok::<_, anyhow::Error>(None)
            }
        }
    };

    let stdout_future = async move {
        if let Some(mut stdout) = stdout {
            Ok(Some({
                tokio::io::copy(&mut stdout, &mut tokio::io::stdout()).await?
            }))
        } else {
            Ok::<_, anyhow::Error>(None)
        }
    };

    futures::future::try_join(stdin_future, stdout_future).await?;

    Ok(())
}
