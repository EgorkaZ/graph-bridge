use graph_bridge::graph;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short='g', long)]
    graph_backend: graph_bridge::graph::GraphBackend,

    #[arg(
        short='d',
        long,
    )]
    draw_backend: graph_bridge::gui::DrawBackend,
}

fn main() {
    let args = Args::parse();

    let mut graph = graph::with_dots_count(args.graph_backend, 10);
    graph.add_edge(0, 1);
    graph.add_edge(1, 2);
    graph.add_edge(2, 0);
    graph.add_edge(0, 4);

    graph.draw(args.draw_backend)
}
