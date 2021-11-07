
# Luna 
Automatic script runner

```
   __  __  ___  _____ 
  / / / / / / |/ / _ |  v0.2.0
 / /_/ /_/ /    / __ |        
/____|____/_/|_/_/ |_|  SA    

```
- Luna can run any bunch of bash scripts and collect results and save them to Database and send the new one's to a Discord channel (optional) [in this version Luna only captures domain-IP and saves them to Database, You can look at data structures in source code ]
- Luna Supports PostgreSQL, MySQL, SQLite, MSSQL and Mongodb (For now mongodb is preferred).
- (Not now) Luna creates a wordlist from the results of every Run.

- لونا در حال حاظر میتونه برای مجموعه ای از دامنه هایی که تو دیتابیسش هست یه سری اسکریپت رو اجرا کنه و مهمتر از اون نتایجش رو با ریجکس استخراج کنه و توی دیتابیس ذخیره کنه (البته هنوز خیلی چیزای دیگه قراره بهش اضافه بشه. اگه ایده ای تو پیاده سازیش داشتید خوشحال میشم در میون بذارید


## Simple Using

1. create this files:  

script.sh:
```bash
amass enum -active  -d $domain -config config.ini -ip -o amass.results -dir amass
subfinder -d $domain -silent
gobuster dns -d  $domain -r ns1.$domain -w wl.txt -qi
...
```
luna.ini:
```ini
DATABASE = mongodb://example.com/test
PATH = .
DISCORD = https://discord.com/api/webhooks/***
```
2. insert scopes:  
`luna insert scope -a target1.com`  
3. run script:  
`luna script -s script.sh --all-scopes `


## Installation   

Compile code with `cargo build --release`   

### Linux portable binary:
For building statically linked rust binary [read this link](https://blog.davidvassallo.me/2021/06/10/lessons-learned-building-statically-linked-rust-binaries-openssl/).



## Usage

```
luna 0.2.0
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
- [ ] Custome regexes for captures all structs
- [ ] More Custome keywords in scripts
- [ ] More push notifications (Telegram, ...)
- [+] NoSQL support
- [ ] cli.yaml
- [ ] Tests
- [ ] More customize
- [ ] Setup check and show status
- [ ] Documents
