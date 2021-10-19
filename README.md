
# Luna 
Standalone Binary, Async, and Muti Database Support 

```
   __  __  ___  _____ 
  / / / / / / |/ / _ |  v0.1.1
 / /_/ /_/ /    / __ |        
/____|____/_/|_/_/ |_|  SA    

```
- Luna can run scripts and collect results (now luna can only run asset finders scripts and collect subdomains and ips) and push them to Database, File, and Discord channel (optional)
- Luna Supports PostgreSQL, MySQL, SQLite, and MSSQL.
- Luna creates a wordlist from the results of every Run.

## Simple Using

create this files:

script.sh:
```bash
amass enum -active  -d $domain -config config.ini -ip -o amass.results -dir amass
subfinder -d $domain -silent
gobuster dns -d  $domain -r ns1.$domain -w wl.txt -qi

```

luna.ini:
```ini
DATABASE = mysql://example.com/test
PATH = .
DISCORD = https://discord.com/api/webhooks/***
```
`luna domain -a target1.com`  
`luna domain -a target2.com`  
`luna domain -s script.sh`  



## Installation   

Compile code with `cargo build --release`   

### Linux portable binary:
For building statically linked rust binary [read this link](https://blog.davidvassallo.me/2021/06/10/lessons-learned-building-statically-linked-rust-binaries-openssl/).



## Usage

```
USAGE:
    luna [OPTIONS] [url] [SUBCOMMAND]

FLAGS:
    -h, --help
            Print help information

    -V, --version
            Print version
            information

OPTIONS:
    
    -d, --db-url <DATABASE-URL>
            Sets database url
            Example:
            
            postgres://postgres@localhost/test
             sqlite://a.sqlite
             sqlite::memory:
             sqlite:data.db
             sqlite://data.db
             sqlite:///data.db
            
            sqlite://data.db?mode=ro

SUBCOMMANDS:
    domain
            Controls domains
    help
            Print this message or
            the help of the given
            subcommand(s)
    subdomain
            Controls subdomains
    word
            Controls wordlist
```
   

## Built With
- **Tokio**: A powerfull runtime for writing reliable, **asynchronous**, and slim applications with the Rust programming language
- **SQLx**: An async, pure Rust SQL crate
- **orm**: A self made library for easy developing using macros in Rust
- ...


## Roadmap
- [ ] cli.yaml
- [ ] Tests
- [ ] Further stages (Fuzz, Scan, Attack)
- [ ] More customize
- [ ] Setup check and show status
- [ ] Single target mode
- [ ] More push notifications (Telegram, ...)
- [ ] NoSQL support

