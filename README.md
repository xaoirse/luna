## Hooray, it works :)
[XAoirse](https://github.com/xaoirse)
# Luna 
(beta version, Under heavy development and tests)
### **Recon** tool Powered by **Rust** 🖤  


```
   __  __  ___  _____ 
  / / / / / / |/ / _ |  v0.4.0
 / /_/ /_/ /    / __ |        
/____|____/_/|_/_/ |_|  SA    

```
 
Luna can run any bunch of bash scripts in **Parallel**, collect results, save them as JSON format and find them with **Regex**.


# Installation   
1. Install cargo
2. Compile code with `cargo build --release`   

### Linux portable binary:
For building statically linked rust binary [read this link](https://blog.davidvassallo.me/2021/06/10/lessons-learned-building-statically-linked-rust-binaries-openssl/).



# Usage

```
luna 0.4.0

USAGE:
    luna [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      
    -V, --version    Prints version information

OPTIONS:
    -j, --json <json>     [default: luna.json]

SUBCOMMANDS:
    check     
    find      
    help      Prints this message or the help of the given
              subcommand(s)
    insert    
    report    
    script    
    server    
    test        
```
## Features
### Available keywords:

- ${program}
- ${scope}
- ${sub}
- ${url}
- ${ip}
- ${port}
- ${keyword}

### Available regex names:
- program
    - program_platform
    - program_handle
    - program_type
    - program_url
    - program_icon
    - program_bounty
    - program_state  

- scope
    - scope_type
    - scope_bounty
    - scop_severity

- sub
    - sub_type

- ip

- port
    - service_name
    - service_protocol
    - service_banner

- url
    - title
    - status_code
    - response

- tech
    - tech_version



# Simple Using
1. Create script file like `script.sh`:  
( pattern for parsing each line of results )  
[ commands ]
```bash
pattern = (?P<sub>)
subfinder -d ${scope} # sub.sample.com -> sub
ammass -d ${scope} # sub.sample.com -> sub

pattern = (?P<url>)-(?P<status_code>)
urlfinder  ${sub} # https://sub.sample.com/login 200 -> url, status_code
```
2. Insert some scopes (see helps):  
`luna insert scope --name google`
3. Run script:  
`luna script script.sh`  
4. Find subs (regex) :  
`luna find sub --scope ^google.com`


# Built With
- [**StructOpt**](https://github.com/TeXitoi/structopt)
: Parse command line arguments by defining a struct. It combines clap with custom derive.
- [**Rayon**](https://github.com/rayon-rs/rayon): A data-parallelism library for Rust.
- [**Regex**](https://github.com/rust-lang/regex
): A library for parsing, compiling, and executing regular expressions.
- ...

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

