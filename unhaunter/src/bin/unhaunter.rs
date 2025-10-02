use clap::Parser;
use uncore::resources::cli_options::CliOptions;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, action)]
    draft_maps: bool,
}

fn main() {
    // Parse command line arguments for game configuration
    // This ensures proper CLI argument handling for the game
    let args = Args::parse();

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Err(e) = unhaunter::assetidx_updater::update_assetidx_files() {
            eprintln!("Failed to update assetidx files: {}", e);
        }
    }
    unhaunter::app_run(CliOptions {
        include_draft_maps: args.draft_maps,
    });
}
