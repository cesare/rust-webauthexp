use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "webauthexp")]
struct AppArgs {
    #[structopt(short = "b", long = "bind", default_value = "127.0.0.1")]
    bind: String,

    #[structopt(short = "p", long = "port", default_value = "8000")]
    port: u32,
}

pub struct AppConfig {
    args: AppArgs,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            args: AppArgs::from_args(),
        }
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.args.bind, self.args.port)
    }
}
