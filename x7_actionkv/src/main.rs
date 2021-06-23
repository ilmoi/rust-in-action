use libactionkv::ActionKV;

// conditional compilation - below only compiles on windows, while the next block only on non-windows
#[cfg(target_os = "windows")]
const USAGE: &str = "
Usage:
    cargo.exe run -- FILE get KEY
    cargo.exe run -- FILE delete KEY
    cargo.exe run -- FILE insert KEY VALUE
    cargo.exe run -- FILE update KEY VALUE
";

#[cfg(not(target_os = "windows"))]
const USAGE: &str = "
Usage:
    cargo run -- FILE get KEY
    cargo run -- FILE delete KEY
    cargo run -- FILE insert KEY VALUE
    cargo run -- FILE update KEY VALUE
";

fn main() {
    //collect and unpack args
    let args: Vec<String> = std::env::args().collect();
    let fname = args.get(1).expect(&USAGE);
    // the reason we need .as_ref() here is because we're treating the next 2 values as slices later on - ie they have to be refs to original
    let action = args.get(2).expect(&USAGE).as_ref();
    let key = args.get(3).expect(&USAGE).as_ref();
    let maybe_value = args.get(4);
    println!("Passed in: {:?} {:?} {:?} {:?}", fname, action, key, maybe_value);

    //Pathbuf is akin to String
    //Path is akin to str (slice)
    //one vs the other: https://stackoverflow.com/questions/32730714/what-is-the-right-way-to-store-an-immutable-path-in-a-struct
    // - Store a PathBuf if you want the struct to own it. If you don't know what you want, start here.
    // - Store a &Path if you just want a reference to a path. Depending on what you're doing, this may be what you want, but if you don't know, it's probably not correct.
    let path = std::path::Path::new(&fname);

    // create an instance of the store = 2 steps:
    // 1 open store = opens the file + creates an empty index
    let mut store = ActionKV::open(path).expect("failed to open file");
    // 2 load store = populates the index with all KV pairs
    store.load().expect("failed to load data");

    match action {
        "get" => {
            match store.get(key).unwrap() {
                None => eprintln!("{:?} not found", key),
                //todo in theory shouldn't be converting to String, as string is only valid utf-8, and we intentionally accept any bytes
                Some(v) => println!("{:?}", String::from_utf8(v)),
            }
        },
        "delete" => {
            store.delete(key).unwrap();
            println!("deleted!")
        },
        "insert" => {
            //delay unwraping the maybe_value till here, till we know we need it
            let v = maybe_value.expect("&USAGE").as_ref();
            store.insert(key, v).unwrap();
            println!("inserted!")
        },
        "update" => {
            let v = maybe_value.expect("&USAGE").as_ref();
            store.update(key, v).unwrap();
            println!("updated!")
        },
        _ => eprintln!("{}", &USAGE),
    }

}

