// use std::io::BufReader;
use ureq;
use fastobo::ast;
use redis::Client;
use redisgraph::{Graph, RedisGraphResult};
use fastobo_graphs::{self, IntoGraph, };
use serde::{Serialize, Deserialize};
use serde_json;
use std::fs::File;
use std::path::Path;
use std::io::Write;

//static HPO_URL: &str = "http://purl.obolibrary.org/obo/hp.obo";
static HPO_FILE_PATH: &str = "../../data/hpo/hp.obo";

fn main() {
    // let resp = ureq::get(HPO_URL).call();
    // let mut reader = BufReader::new(resp.into_reader()) ;
    // let doc = fastobo::from_reader(&mut reader).expect("error reading obo file to OboDoc");
    let doc = fastobo::from_file(HPO_FILE_PATH).unwrap();
    let header = doc.header();
    println!("format: {}", header.format_version().unwrap());
    println!("data {}", header.data_version().unwrap());
    
    let terms: Vec<&ast::TermFrame> = doc
        .entities()
        .iter()
        .flat_map(ast::EntityFrame::as_term_frame)
        .collect();
    println!("terms: {}", terms.len());
    
    /*
    // there are no typedefs and instances in hpo
    let typedefs: Vec<&ast::TypedefFrame> = doc
    .entities()
    .iter()
    .flat_map(ast::EntityFrame::as_typedef_frame)
    .collect();
    println!("typedefs: {}", typedefs.len());

    let instances: Vec<&ast::InstanceFrame> = doc
    .entities()
    .iter()
    .flat_map(ast::EntityFrame::as_instance_frame)
    .collect();
    println!("instances: {}", instances.len());
    */
    let res = terms
        .into_iter()
        .find(|&x| x.id().to_string().trim() == "HP:0000003").expect("didn't find the id :/");
    println!("{}", res);

    let clauses = res.clauses();
    for clause in clauses.into_iter() {
        if let ast::TermClause::Name(_unquoted_string) = clause.as_inner() {
            println!("{}", _unquoted_string);
        }
        
        let inner: Option<&ast::TermClause> = match clause.as_inner() {
            ast::TermClause::Name(_unquoted_string) => Some(clause),
            ast::TermClause::AltId(_ident) => Some(clause),
            ast::TermClause::Xref(_xref) => Some(clause),
            _ => None,
        };
        if let Some(inner) = inner {
            println!("{}", inner);
        }
    }

    // Here we create a GraphDocument from the OboDoc. This sshould allow ous to export to JSON etc.
    println!();
    println!("HPO OBO into GraphDocument:");
    println!();

    let doc_graph = doc.into_graph().unwrap();
    // let graphs_len = doc_graph.graphs.len();
    // println!("number of graphs: {}", graphs_len);
    let nodes_count = doc_graph.graphs[0].nodes.len();
    println!("number of nodes: {}", nodes_count);

    let prefix: &str = "http://purl.obolibrary.org/obo/";
    let node = &doc_graph.graphs[0].nodes
        .iter()
        .find(|&x| x.id.trim() == format!("{}HP_0000003", prefix)).expect("could not find the node");
    let serialized = serde_json::to_string(&node);
    let path = "../../data/hpo/hp_0000003.json";
    let file = File::create(path).unwrap();
    serde_json::to_writer(file, &node).unwrap();
    fastobo_graphs::to_file("../../data/hpo/hpo.json", &doc_graph).expect("erropr writing GraphDocument to file");


    redis_test().unwrap();
}

pub fn redis_test() -> RedisGraphResult<()> {
    // Lets test out the Redis and RedisGraph clients! 
    println!("&& --- &&");
    println!("Let's test RedisGraph!");
    println!("$$ --- $$");

    let client = Client::open("redis://127.0.0.1")?;
    let mut connection = client.get_connection()?;

    let mut graph = Graph::open(&mut connection, "MotoGP")?;

    // Create six nodes (three riders, three teams) and three relationships between them.
    graph.mutate("CREATE (:Rider {name: 'Valentino Rossi', birth_year: 1979})-[:rides]->(:Team {name: 'Yamaha'}), \
        (:Rider {name:'Dani Pedrosa', birth_year: 1985, height: 1.58})-[:rides]->(:Team {name: 'Honda'}), \
        (:Rider {name:'Andrea Dovizioso', birth_year: 1986, height: 1.67})-[:rides]->(:Team {name: 'Ducati'})")?;

    // Get the names and birth years of all riders in team Yamaha.
    let results: Vec<(String, u32)> = graph.query("MATCH (r:Rider)-[:rides]->(t:Team) WHERE t.name = 'Yamaha' RETURN r.name, r.birth_year")?;
    println!("{:?}", results);
    println!();
    // Since we know just one rider in our graph rides for team Yamaha,
    // we can also write this and only get the first record:
    let (name, birth_year): (String, u32) = graph.query("MATCH (r:Rider)-[:rides]->(t:Team) WHERE t.name = 'Yamaha' RETURN r.name, r.birth_year")?;
    // Let's now get all the data about the riders we have.
    // Be aware of that we only know the height of some riders, and therefore we use an `Option`:
    let results: Vec<(String, u32, Option<f32>)> = graph.query("MATCH (r:Rider) RETURN r.name, r.birth_year, r.height")?;
    println!("{:?}", results);

    // That was just a demo; we don't need this graph anymore. Let's delete it from the database:
    graph.delete()?;
    Ok(())
 
}
