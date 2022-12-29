mod backend;
mod cmd;
mod model;

use structopt::StructOpt;

use model::result::Result;

#[derive(StructOpt, Debug)]
#[structopt(name = "virtool")]
struct Cli {
    // Uname of the wallet to retrieve
    #[structopt(short, long)]
    uname: String,

    #[structopt(subcommand)]
    cmd: CliCmd,
}

#[derive(StructOpt, Debug)]
enum CliCmd {
    Balance,
    Transfer {
        #[structopt(short, long)]
        dest: String,
        #[structopt(short, long)]
        value: u128,
    },
}

#[async_std::main]
async fn main() {
    match run().await {
        Ok(_) => {}
        Err(err) => {
            log::error!("{}", err);
            std::process::exit(1);
        }
    }
}

async fn run() -> Result<()> {
    let cli = Cli::from_args();
    let uname = cli.uname;

    match cli.cmd {
        CliCmd::Balance => cmd::balance::balance(uname).await,
        CliCmd::Transfer { dest, value } => cmd::transfer::transfer(uname, dest, value).await,
    }
}
