use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Ok};
use clap::Parser;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tabled::settings::Style;
use tokio::time::sleep;

mod args;
mod sadl;

fn get_terminal_input<T>() -> T
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    let read: T = line.trim().parse::<T>().unwrap();
    read
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = args::Args::parse();

    let resp = sadl::slavart_search(&args.query.clone()).await?;

    let mut builder = tabled::builder::Builder::default();

    builder.set_header(["No", "Id", "Title", "isrc", "Performer"]);

    for each in resp.tracks.items.iter().enumerate() {
        builder.push_record([
            (each.0 + 1).to_string(),
            each.1.id.to_string(),
            each.1.title.to_string(),
            each.1.isrc.to_string(),
            each.1.performer.to_string(),
        ]);
    }

    let table = builder.build().with(Style::psql()).to_string();
    println!("{}", table);

    while args.sel < 1 || args.sel > resp.tracks.items.len() {
        args.sel = get_terminal_input();
    }

    if args.sel > 0 && args.sel <= resp.tracks.items.len().try_into()? {
        let targ_path = PathBuf::from(".");

        let file_name = {
            let mut file_name: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .map(char::from)
                .take(30)
                .collect();
            file_name.push_str("_");
            file_name.push_str(resp.tracks.items[args.sel - 1].id.to_string().as_str());
            file_name.push_str("_");
            file_name.push_str(resp.tracks.items[args.sel - 1].isrc.as_str());
            file_name.push_str("_");
            file_name.push_str(resp.tracks.items[args.sel - 1].title.as_str());
            file_name.push_str("_");
            file_name.push_str(
                resp.tracks.items[args.sel - 1]
                    .performer
                    .to_string()
                    .as_str(),
            );
            file_name.push_str(".flac");
            file_name
        };

        let targ_path = targ_path.join(file_name);

        let mut retries = args.retries;
        while retries > 0 {
            if let Err(e) =
                sadl::slavart_fetch_track(resp.tracks.items[args.sel - 1].id, &targ_path).await
            {
                retries -= 1;
                sleep(Duration::from_secs(5)).await;
                println!("Retrying \n {:?}", e);
            } else {
                retries = 0;
            }
        }
    }

    Ok(())
}
