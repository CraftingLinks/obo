// use std::io::BufReader;
// use ureq;
use fastobo::ast;

// static HPO_URL: &str = "http://purl.obolibrary.org/obo/hp.obo";
static HPO_FILE_PATH: &str = "../../data/hpo/hp.obo";

fn main() {
    // let resp = ureq::get(HPO_URL).call();
    // let mut reader = BufReader::new(resp.into_reader()) ;
    // let doc = fastobo::from_reader(&mut reader).expect("error reading obo file");
    
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
        .find(|&x| x.id().to_string().trim() == "HP:0000006").expect("didn't find the id :/");
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
}
