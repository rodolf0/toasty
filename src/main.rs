#![deny(warnings)]

use shunting::{ShuntingParser, MathContext};

use dbus::tree::{MTFn, MethodInfo, Interface, MethodResult, MethodErr};
use dbus::arg::Variant;

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, Once};

fn connect() -> Result<dbus::Connection, dbus::Error> {
    const BUS_NAME: &str = "rodolf0.toasty.SearchProvider";
    let conn = dbus::Connection::get_private(dbus::BusType::Session)?;
    conn.register_name(BUS_NAME, dbus::NameFlag::ReplaceExisting as u32)?;
    Ok(conn)
}

fn ctx() -> &'static Mutex<HashSet<String>> {
    // map from result-id to context
    static INIT: Once = Once::new();
    static mut CTX: Option<Mutex<HashSet<String>>> = None;
    unsafe {
        INIT.call_once(|| { CTX = Some(Mutex::new(HashSet::new())); });
        CTX.as_ref().unwrap()
    }
}


fn get_initial_resultset(minfo: &MethodInfo<MTFn<()>, ()>) -> MethodResult {
    let terms: Vec<String> = minfo.msg.read1()?;
    eprintln!("GetInitialResultSet terms={:?}", terms);

    // Just use the merged terms as result-id
    let expr = terms.join(" ");
    ctx().lock().unwrap().insert(expr.clone());
    let result_ids = vec!(expr);

    let mret = minfo.msg.method_return().append1(result_ids);
    Ok(vec!(mret))
}

fn get_subsearch_resultset(minfo: &MethodInfo<MTFn<()>, ()>) -> MethodResult {
    let (prev, terms): (Vec<String>, Vec<String>) = minfo.msg.read2()?;
    eprintln!("GetSubsearchResultSet prev={:?} terms={:?}", prev, terms);

    // Called to refine search within initial results, nothing we wanna do here
    let expr = terms.join(" ");
    ctx().lock().unwrap().insert(expr.clone());
    let result_ids = vec!(expr);

    let mret = minfo.msg.method_return().append1(result_ids);
    Ok(vec!(mret))
}

// A map from string to variant
type MetasMap = HashMap<String, Variant<String>>;

fn get_result_metas(minfo: &MethodInfo<MTFn<()>, ()>) -> MethodResult {
    let ids: Vec<String> = minfo.msg.read1()?;
    eprintln!("GetResultMetas ids={:?}", ids);

    // We're storing the
    let mut metas = Vec::new();
    for input in ids {
        let expr = ShuntingParser::parse_str(&input)
            .map_err(|_| MethodErr::invalid_arg(&"Can't parse input"))?;
        let result = MathContext::new().eval(&expr)
            .map_err(|_| MethodErr::invalid_arg(&"Can't eval input"))?;

        let mut meta = MetasMap::new();
        meta.insert(format!("name"), Variant(result.to_string()));
        meta.insert(format!("id"), Variant(input.clone()));
        meta.insert(format!("description"), Variant(input));
        metas.push(meta);
    }

    let mret = minfo.msg.method_return().append1(metas);
    Ok(vec!(mret))
}

fn activate_result(minfo: &MethodInfo<MTFn<()>, ()>) -> MethodResult {
    let (id, terms, ts): (String, Vec<String>, u32) = minfo.msg.read3()?;
    eprintln!("ActivateResult id={} terms={:?} ts={}", id, terms, ts);
    Ok(vec!())
}

fn launch_search(minfo: &MethodInfo<MTFn<()>, ()>) -> MethodResult {
    let terms: Vec<String> = minfo.msg.read1()?;
    eprintln!("LaunchSearch terms={:?}", terms);
    Ok(vec!())
}

fn search_iface() -> Interface<MTFn<()>, ()> {
    let f = dbus::tree::Factory::new_fn();
    // DOCS: org.gnome.ShellSearchProvider2.xml
    f.interface("org.gnome.Shell.SearchProvider2", ())

        .add_m(f.method("GetInitialResultSet", (), get_initial_resultset)
               .inarg::<Vec<String>,_>("terms")
               .outarg::<Vec<String>,_>("results"))

        .add_m(f.method("GetSubsearchResultSet", (), get_subsearch_resultset)
               .inarg::<Vec<String>,_>("previous_results")
               .inarg::<Vec<String>,_>("terms")
               .outarg::<Vec<String>,_>("results"))

        .add_m(f.method("GetResultMetas", (), get_result_metas)
               .inarg::<String,_>("identifiers")
               .outarg::<Vec<MetasMap>,_>("metas"))

        .add_m(f.method("ActivateResult", (), activate_result)
               .inarg::<String,_>("identifier")
               .inarg::<Vec<String>,_>("terms")
               .inarg::<u32,_>("timestamp"))

        .add_m(f.method("LaunchSearch", (), launch_search)
               .inarg::<Vec<String>,_>("terms")
               .inarg::<u32,_>("timestamp"))
}


fn main() {
    // connect to session dbus and register our bus
    let conn = connect().unwrap();
    // factory is used for creating object-paths, methods, interfaces, etc.
    let f = dbus::tree::Factory::new_fn();

    let search_interface = search_iface();

    // Create a tree with single object-path for our search
    const SEARCH_PATH: &str = "/rodolf0/toasty/SearchProvider";
    let tree = f.tree(()).add(
        f.object_path(SEARCH_PATH, ()).introspectable().add(search_interface));

    // Register all object paths in the tree.
    tree.set_registered(&conn, true).unwrap();

    // We add the tree to the connection to handle incoming method calls
    conn.add_handler(tree);

    // Serve other peers forever.
    loop { conn.incoming(1000).next(); }
}
