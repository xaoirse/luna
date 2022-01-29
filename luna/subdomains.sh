# ./tools/subfinder/amass enum -active  -d $$ -config tools/assetsfinder/amass.config.ini -ip -o amass.results -dir amass
./tools/subfinder/subfinder -d $$ -silent
# ./tools/subfinder/gobuster dns -d  $$ -r ns1.$$ -w luna/wl.txt -qi
