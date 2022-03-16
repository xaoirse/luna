# Luna 
(preparing for stable version)
### **Reconnaissance** tool, Powered by **Rust**, built with 💖  

```
   __  __  ___  _____ 
  / / / / / / |/ / _ |  v0.8.0
 / /_/ /_/ /    / __ |        
/____|____/_/|_/_/ |_|  SA    

```
 
Luna can run any bunch of bash scripts in **Parallel**, collect results, save them as JSON format and find them with **Regex**.


# Installation   
1. [Install Rust](https://www.rust-lang.org/tools/install)
2. Compile code with `cargo build --release`   

### Linux portable binary:
For building statically linked rust binary [read this link](https://blog.davidvassallo.me/2021/06/10/lessons-learned-building-statically-linked-rust-binaries-openssl/) and [here](https://doc.rust-lang.org/reference/linkage.html#static-and-dynamic-c-runtimes)

minimum size:  
`RUSTFLAGS='-C target-feature=+crt-static -C link-arg=-s -C panic=abort -C codegen-units=1' cargo build --target x86_64-unknown-linux-musl --release` + UPX

# Usage

```
luna 0.8.0
SA
The moon rider has arived.

USAGE:
    luna [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help         Prints help information
        --no-backup    Save without backup
    -q, --quiet        Quiet mode
    -V, --version      Prints version information

OPTIONS:
    -i, --input <input>        Json file's path [default: luna.json]
    -o, --output <output>      Default output is input!
    -t, --threads <threads>    Number of threads

SUBCOMMANDS:
    check     
    find      
    help      Prints this message or the help of the given subcommand(s)
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
- ${program}
- ${domain}
- ${cidr}
- ${sub}
- ${url}

Example:
- `curl -I ${url}`
- `subfinder -d ${domain}`

### <a name="regex-names"> </a>Available regex names:
- asset
    - title
    - sc
    - response
- tag
    - severity
    - value

Example:
- `(?P<asset>.+)`  

[Test your regex](https://rustexp.lpil.uk/)

# Simple Using
1. Create script file like `script.sh`:  
( regex for parsing results [see this](#regex-names) )  
[ commands [see this](#available-keywords)]
(Be carfull when using tools with colorful output)
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
4. Find subs (regex):  
`luna find sub --program google`
5. log levels: debug, error, info  
`RUST_LOG=error luna insert asset blah`


# Built With
- [**Clap**](https://github.com/clap-rs/clap)
: Command Line Argument Parser for Rust.
- [**Rayon**](https://github.com/rayon-rs/rayon): A data-parallelism library for Rust.
- [**Regex**](https://github.com/rust-lang/regex
): A library for parsing, compiling, and executing regular expressions.
- ...

# FAQ
### Is it works?
Yes, I am using it. but it may has bugs so I try to fix them.
### Why Rust?
I want a standalone binary release from my code,


# Contribute
- Use
- Share your idea
- File Issue
- Pull Request
- ...

## In Progress
- Enhanece Performance
- Tests
- Comments


## TODO
- [ ] **WebServer**  
- [ ] Report system  
- [ ] Script validateor  
- [ ] Cache system  
- [x] Update and delete mechanism  
- [x] Limit for parallel requests to prevent rate limit  
- [ ] Worldlist
- [x] Custom inputs for script
- [ ] Save bash file scripts in json or each field? jom model
- [ ] Regex test tool (subcommand)
- [ ] Reduce release size
- [x] Filter by date
- [x] Find -vvv flags
- [ ] Remove all clones  
- [x] No-backup flag
- [x] Label or tag for vulnerabilities   
- [x] Import file
- [x] Number of urls, subs and ... for each program stringify
- [ ] Update_at updates every time!
- [ ] Global search
- [ ] Insert from file
- [x] Graceful shutdown
- [ ] Pause and Resume (OMG!)
- [x] Progress bar
- [x] Remove tech
- [ ] Update dependencies
- [ ] Benchmarks
- [ ] Tests
- [ ] Job