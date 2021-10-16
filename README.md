
Luna runs scripts  
- luna can run scripts and parse results (for now luna can only runs assetfinders scripts and collect subdomains and ips)

## Script

script.sh file:
```bash
amass enum -active  -d $domain -config config.ini -ip -o amass.results -dir amass
subfinder -d $domain -silent
gobuster dns -d  $domain -r ns1.$domain -w wl.txt -qi

```
Not completed:  
`luna domain --all `-s script.sh  
Extracts all subdomains and inserts to Database(file, sqlit, postgres and mysql)

## Installation
- Customize `.env` file
- Compile code with `cargo build --release`  

## USAGE

```
USAGE:
    luna [OPTIONS] [url] [SUBCOMMAND]

ARGS:
    <url>    url for scan

FLAGS:
    -h, --help       Print help information
    -V, --version    Print version information

OPTIONS:
    -c, --config <FILE>
            Sets a custom config file

    -d, --db-url <DATABASE-URL>
            Sets database url Example:
             postgres://postgres@localhost/test
             sqlite://a.sqlite
             sqlite::memory:
             sqlite:data.db
             sqlite://data.db
             sqlite:///data.db
             sqlite://data.db?mode=ro

    -s, --script <domain>
            Runs script against domain

SUBCOMMANDS:
    domain
            Controls domains
    help
            Print this message or the help of the
            given subcommand(s)
    subdomain
            Controls subdomains
    word
            Controls wordlist

```
### Examples
`luna -d postgres://postgres@localhost/test -a example.com `  
`luna subdomain -a sub.example.com`  
`luna word -a add keyword`



## TODOs
- [ ] Complete cli.yaml
- [ ] Tests
- [ ] Nuclei for fuzz
- [ ] Complete flags