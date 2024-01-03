mod cli;
mod diagram;
mod field;
mod lagrangian;
use rand::seq::SliceRandom;
use rand::thread_rng;

fn main() {
    let (lagrangian, n) = cli::parse();

    println!("Lagrangian is {:#?}", lagrangian);
    println!("Lagrangian is {}", lagrangian);
    println!(
        "Number of kinds of particles is {}",
        lagrangian.fields().len()
    );
    println!("Maximum order is {}", n);

    let diag = diagram::Diagram::new(vec![
        diagram::Vertex::external(lagrangian.fields()[1].clone()),
        diagram::Vertex::external(lagrangian.fields()[2].clone()),
        diagram::Vertex::external(lagrangian.fields()[1].clone()),
        diagram::Vertex::external(lagrangian.fields()[2].clone()),
        diagram::Vertex::internal(lagrangian.interactions()[0].clone()),
        diagram::Vertex::internal(lagrangian.interactions()[0].clone()),
        diagram::Vertex::internal(lagrangian.interactions()[0].clone()),
        diagram::Vertex::internal(lagrangian.interactions()[0].clone()),
        diagram::Vertex::internal(lagrangian.interactions()[0].clone()),
        diagram::Vertex::internal(lagrangian.interactions()[0].clone()),
        diagram::Vertex::internal(lagrangian.interactions()[1].clone()),
        diagram::Vertex::internal(lagrangian.interactions()[1].clone()),
        diagram::Vertex::internal(lagrangian.interactions()[1].clone()),
        diagram::Vertex::internal(lagrangian.interactions()[1].clone()),
        diagram::Vertex::internal(lagrangian.interactions()[1].clone()),
        diagram::Vertex::internal(lagrangian.interactions()[1].clone()),
        diagram::Vertex::internal(lagrangian.interactions()[2].clone()),
        diagram::Vertex::internal(lagrangian.interactions()[2].clone()),
    ]);

    println!("Diagram is {:#?}", diag);

    let mut diag = diag.draw();
    diag.shuffle(&mut thread_rng());

    for x in &diag {
        println!("Diagram is {}", x);
    }
}
