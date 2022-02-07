## Usable but still beta (Under heavy development and tests)
# Luna 
### **Recon** tool Powered by **Rust** 🖤

```
   __  __  ___  _____ 
  / / / / / / |/ / _ |  v0.4.0
 / /_/ /_/ /    / __ |        
/____|____/_/|_/_/ |_|  SA    

```
 
Luna can run any bunch of bash scripts in **Parallel** and collect results and save them as JSON format then find them with **Regex**.

  


## Simple Using
1. Create script file like `script.sh`:  
set pattern for parsing each line of results
```bash
pattern = (?P<sub>)
subfinder -d ${scope} # sub.sample.com -> sub
ammass -d ${scope} # sub.sample.com -> sub

pattern = (?P<url>)-(?P<status_code>)
x  ${sub} # https://sub.sample.com/login 200 -> url, status_code
```
2. Insert some scopes:  
`luna insert scope --name google`
3. Run script:  
`luna script script.sh`  
4. Find subs:  
`luna find sub --scope google.com`


## Installation   
1. Install cargo
2. Compile code with `cargo build --release`   

### Linux portable binary:
For building statically linked rust binary [read this link](https://blog.davidvassallo.me/2021/06/10/lessons-learned-building-statically-linked-rust-binaries-openssl/).



## Usage

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

## Built With
- **StructOpt**: Parse **command line arguments** by defining a struct. It combines clap with custom derive.
- **Rayon**: A data-parallelism library for Rust.
- **Regex**: A library for parsing, compiling, and executing regular expressions.
- ...


## In Progress
- Tests
- Comments
- Regex test
- Reduce release size
- Report -vvv
- Filter by date
- Default luna parameters
- Regex names in readme


## TODO
- [ ] **WebServer**  
- [ ] Report system  
- [ ] Script validateor  
- [ ] Cache system  
- [ ] Update and delete mechanism  
- [ ] Limit for parallel requests to prevent rate limit  
- [ ] Worldlist
- [ ] Custom inputs for script
- [ ] Request and Response for Url