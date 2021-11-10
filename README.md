
# Luna 
Automatic script runner  
لونا فقط یه ابزار اوتومیت سازی برای اجرای اسکریپت ها و ذخیره ی نتایج اوناست
  
```
   __  __  ___  _____ 
  / / / / / / |/ / _ |  v0.2.2
 / /_/ /_/ /    / __ |        
/____|____/_/|_/_/ |_|  SA    

```
- Luna can run any bunch of bash scripts and collect results and save them to Database and send the new one's to a Discord channel (optional) *[in this version Luna can extract Hosts and Domains and URLs out of tools like **Subfinder**, **Amass**, etc and saves them to Database, You can look at data structures in source code]*
- Luna Supports PostgreSQL, MySQL, SQLite, MSSQL and mongodb (For now **mongodb** is preferred).
- (Not now) Luna creates a wordlist out of the results.



## Simple Using

1. Create this files:  

script.bash:
```bash
amass enum -active  -d $$ -config config.ini -ip -o amass.results -dir amass
subfinder -d $$ -silent
gobuster dns -d  $$ -r ns1.$$ -w wl.txt -qi
...
```
luna.ini:
```ini
DATABASE = mongodb://example.com/test
PATH = .
DISCORD = https://discord.com/api/webhooks/***
```
2. Insert scopes:   
`luna insert scope -a target1.com -p TestProgram`    
`luna insert scope -a target2.com -p WorkProgram`   
3. Run script for all scopes:  
`luna script -s script.bash --all-scopes `  
4. Find them with mongo query:  
`luna find sub "{'scope':'target1.com'}"`  
`luna find host "{'sub':'subdomain.target1.com'}" -n 10 --sort '{"update":1}' -f ip`

## Installation   

Compile code with `cargo build --release`   

### Linux portable binary:
For building statically linked rust binary [read this link](https://blog.davidvassallo.me/2021/06/10/lessons-learned-building-statically-linked-rust-binaries-openssl/).



## Usage

```
luna 0.2.2
The Moon Rider has arrived.
mongodb

USAGE:
    luna <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version
            Prints version information


SUBCOMMANDS:
    find      
    help      Prints this message or the
              help of the given
              subcommand(s)
    insert    
    script    
```
   

## Built With
- **Tokio**: A powerfull runtime for writing reliable, **asynchronous**, and slim applications with the Rust programming language
- **orm**: A self made library for easy developing using macros in Rust
- ...

## TODO
- [ ] Better regexes for captures all structs
- [ ] More Custome keywords in scripts
- [ ] More push notifications (Telegram, ...)
- [x] NoSQL support
- [ ] cli.yaml
- [ ] Tests
- [ ] More customize
- [ ] Setup check and show status
- [ ] Documents

## Need Idea
- How get more Keyowrds? from config file, commandline, or hardcode?
- Better Names for structs
- Better Database structure
- Regexes are very heavy and take a long time

## For Tomorrow
- Complete regexes for all types for tools such as FFUF,HTTPX,etc
- Make a MVP and deploy it
