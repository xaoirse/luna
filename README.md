# Luna 

### **Reconnaissance** tool, Powered by **Rust**, Built with 💖  

```
   __  __  ___  _____ 
  / / / / / / |/ / _ |  v1.0.0
 / /_/ /_/ /    / __ |        
/____|____/_/|_/_/ |_|  SA    

```
 
Luna can run any batch of bash scripts in **Parallel**, collect results, store them in JSON format and find them with **Regex**.


# Installation   
1. [Install Rust](https://www.rust-lang.org/tools/install)
2. Compile code with `cargo build --release`   

### Linux portable binary:
For building statically linked rust binary [read this link](https://blog.davidvassallo.me/2021/06/10/lessons-learned-building-statically-linked-rust-binaries-openssl/) and [here](https://doc.rust-lang.org/reference/linkage.html#static-and-dynamic-c-runtimes)

minimum size:  
`RUSTFLAGS='-C target-feature=+crt-static -C link-arg=-s -C panic=abort -C codegen-units=1' cargo build --target x86_64-unknown-linux-musl --release` + UPX

# Usage

```
Luna 0.9.0
SAoirse <https://github.com/xaoirse>
A Reconnaissance Tool

USAGE:
    luna [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help                 Print help information
    -i, --input <INPUT>        Json file's path [default: luna.json]
        --no-backup            Save without backup!
    -o, --output <OUTPUT>      Default output is input!
    -q, --quiet                Quiet mode
    -t, --threads <THREADS>    Number of threads
    -V, --version              Print version information

SUBCOMMANDS:
    check     
    dnsgen    
    find      
    help      Print this message or the help of the given subcommand(s)
    import    
    insert    
    remove    
    report    
    script    
    server    
    stat
```
## Features
### <a name="available-keywords"> </a>Available keywords:
- `${program}`
- `${domain}`
- `${cidr}`
- `${sub}`
- `${url}`

Example:
- `curl -I ${url}`
- `subfinder -d ${domain}`

### <a name="regex-names"> </a>Available regex names:
- `asset`
    - `title`
    - `sc`
    - `resp`
- `tag`
    - `severity`
    - `value`

Example:
- `(?P<asset>.+)`  

[Test your regex](https://rustexp.lpil.uk/)

# Simple Using
1. Create a script file like `script.sh`:  
( regex for parsing results [see this](#regex-names) )  
[ commands [see this](#available-keywords)]
(Be cautious when using tools that produce colorful output)
```bash
regex = (?P<asset>.+)
subfinder -d ${sub} # sub1.sample.com -> sub

regex = (?P<asset>(?:\w+)://\S+) \[(?P<sc>\d*)\] \[(?P<title>[^\]]*)\] \[(?P<tag>[^\]]*)\]
echo ${sub} | ./httpx -nc -silent -sc -title -ip -td 
```
2. Insert some scopes (see helps):  
`luna insert asset google.com -p google`
3. Run script:  
`luna script script.sh`  
4. Find subs with regex (use `(?-a)` for case sensitive):  
`luna find sub --program google`
5. log levels: debug, error, info  
`RUST_LOG=error luna insert asset blah`


# Built With
- [**Clap**](https://github.com/clap-rs/clap)
: Command Line Argument Parser for Rust.
- [**Rayon**](https://github.com/rayon-rs/rayon): A data-parallelism library for Rust.
- [**Regex**](https://github.com/rust-lang/regex
): A library for parsing, compiling and executing regular expressions.
- ...

# FAQ
### Does it work?
Yes, I use it. There may be bugs, so I try to fix them.
### Why Rust?
I want a standalone binary release of my code.


# Contribute
- Use
- Share your idea
- File Issue
- Pull Request
- ...

## In Progress
- Enhance Performance
- Tests
- Docs


## TODO
- [ ] **WebServer**  
- [ ] Report system  
- [ ] Script validateor  
- [ ] Cache system  
- [x] Update and delete mechanism  
- [x] Limit for parallel requests to prevent rate limit  
- [ ] Worldlist
- [x] Custom inputs for script
- [ ] Save bash file scripts in json or each field? job model
- [ ] Regex test tool (subcommand)
- [ ] Reduce release size
- [x] Filter by date
- [x] Find -vvv flags
- [ ] Remove all clones  
- [x] No-backup flag
- [x] Label or tag for vulnerabilities   
- [x] Import file
- [x] Number of urls, subs and ... for each program stringify
- [x] Update_at updates every time!
- [ ] Global search
- [ ] Insert from file
- [x] Graceful shutdown
- [ ] Pause and Resume (OMG!)
- [x] Progress bar
- [x] Remove tech
- [x] Update dependencies
- [ ] Benchmarks
- [ ] Tests
- [ ] Job
- [ ] [High] Concurrent access? Lock luna.json and then import?
- [ ] Aggregating Cidrs should aggregate tags or separate cidrs from assets
- [ ] Time-based auto-saving
- [ ] Bring regexes to luna?
- [ ] assets from redirect
- [ ] Refactor