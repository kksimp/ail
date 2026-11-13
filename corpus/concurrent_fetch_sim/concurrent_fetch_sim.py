def fake_fetch(r):
    rid, url, attempt = r
    if "missing" in url:
        status = 404
    elif "busy" in url and attempt < 2:
        status = 503
    else:
        status = 200
    size = len(url.encode()) * 37 % 900 + 100 if status == 200 else 0
    return (rid, url, status, size)


def run_pool(reqs):
    return sorted((fake_fetch(r) for r in reqs), key=lambda x: x[0])


def fetch_all(urls):
    first = run_pool([(i, urls[i], 1) for i in range(len(urls))])
    retry = [(r[0], r[1], 2) for r in first if r[2] == 503]
    done = list(first)
    if retry:
        print(f"retrying {len(retry)} busy pages")
        for again in run_pool(retry):
            done[again[0]] = again
    return done


def main():
    urls = [
        "http://site.test/index",
        "http://site.test/busy/report",
        "http://site.test/missing/page",
        "http://site.test/about",
        "http://site.test/busy/search",
        "http://site.test/docs/intro",
        "http://site.test/docs/missing",
    ]
    done = fetch_all(urls)
    by_status = {}
    total = 0
    for rid, url, status, size in done:
        print(f"{rid} {status} {size:>5} {url}")
        by_status[status] = by_status.get(status, 0) + 1
        total += size
    for code in sorted(by_status):
        print(f"status {code}: {by_status[code]}")
    print(f"{len(done)} pages, {total} bytes fetched")


main()
