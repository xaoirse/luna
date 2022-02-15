## My automation tool
[XAoirse](https://github.com/xaoirse)
# Luna 
(beta version, Under heavy development and tests)
### **Reconnaissance** tool, Powered by **Rust**, built with 🖤  


```
   __  __  ___  _____ 
  / / / / / / |/ / _ |  v0.4.0
 / /_/ /_/ /    / __ |        
/____|____/_/|_/_/ |_|  SA    

```
 
Luna can run any bunch of bash scripts in **Parallel**, collect results, save them as JSON format and find them with **Regex**.


# Installation   
1. [Install Rust](https://www.rust-lang.org/tools/install)
2. Compile code with `cargo build --release`   

### Linux portable binary:
For building statically linked rust binary [read this link](https://blog.davidvassallo.me/2021/06/10/lessons-learned-building-statically-linked-rust-binaries-openssl/) and [here](https://doc.rust-lang.org/reference/linkage.html#static-and-dynamic-c-runtimes)

`RUSTFLAGS='-C target-feature=+crt-static -C link-arg=-s' cargo build --target x86_64-unknown-linux-musl --release`


`-s`, `--strip-all` Omit all symbol information from the output file.

See more keys and their definitions [here](https://doc.rust-lang.org/cargo/reference/manifest.html)


# Usage

```
luna 0.4.0
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
    -j, --json <json>    Json file's path [default: luna.json]

SUBCOMMANDS:
    check     
    find      
    help      Prints this message or the help of the given
              subcommand(s)
    import
    insert
    luna    
    report    
    script    
    server    
    test      
```
## Features
### <a name="available-keywords"> </a>Available keywords:
- ${program}
- ${domain}
- ${cidr}
- ${sub}
- ${url}
- ${ip}
- ${port}
- ${keyword}

Example:
- `curl -I ${url}`
- `subfinder -d ${domain}`

### <a name="regex-names"> </a>Available regex names:
- program
    - program-platform
    - program-handle
    - program-type
    - program-url
    - program-icon
    - program-bounty
    - program-state  

- scope
    - scope-bounty
    - scop-severity

- sub
    - sub-type

- ip

- port
    - service-name
    - service-protocol
    - service-banner

- url
    - title
    - status-code
    - response

- tech
    - tech-version

Example:
- `(?P<response>.+)`  

[Test your patterns](https://rustexp.lpil.uk/)

# Simple Using
1. Create script file like `script.sh`:  
( pattern for parsing each line of results [see this](#regex-names) )  
[ commands [see this](#available-keywords)]
```bash
pattern = (?P<sub>.+)
subfinder -d ${scope} # sub1.sample.com -> sub
findsuber -d ${scope} # sub2.sample.com -> sub

pattern = (?P<url>.+) (?P<status-code>\d+)
urlfinder  ${sub} # https://sub.sample.com/login 200 -> url status-code
```
2. Insert some scopes (see helps):  
`luna insert scope --asset google.com`
3. Run script:  
`luna script script.sh`  
4. Find subs (regex):  
`luna find sub --scope ^google.com`
5. log levels: debug, error, info  
`RUST_LOG=error luna insert program --name blah`


# Built With
- [**StructOpt**](https://github.com/TeXitoi/structopt)
: Parse command line arguments by defining a struct. It combines clap with custom derive.
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
- Wordlist
- Tests
- Comments


## TODO
- [ ] **WebServer**  
- [ ] Report system  
- [ ] Script validateor  
- [ ] Cache system  
- [ ] Update and delete mechanism  
- [ ] Limit for parallel requests to prevent rate limit  
- [ ] Worldlist
- [ ] Custom inputs for script
- [ ] Request body for Url
- [ ] Regex test tool (subcommand)
- [ ] Reduce release size
- [x] Filter by date
- [x] Find -vvv flags
- [ ] Remove all clones  
- [x] No-backup flag
- [ ] Label or tag for vulnerabilities   
(how to add and remove)(vec of Strings)  
How got this regex `"[a-z]+ (?P<tag>\[[a-z]+\]){1,3}"` for `"url [crit][a][b]"` to captures a and b individually?
- [ ] Rename to delete for delete a field
- [x] Merge two file
- [ ] Number of urls, subs and ... for program stringify