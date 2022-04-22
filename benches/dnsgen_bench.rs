use dnsgen::dnsgen;

pub fn dnsgen_bench(n: i32) {
    let mut subdomains = vec![];
    for i in 0..n {
        subdomains.push(format!("www{}.test.com", i))
    }

    let mut wl = vec![];
    for i in 0..n {
        wl.push(format!("word{}", i))
    }

    dnsgen(subdomains, wl);
}
