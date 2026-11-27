# Standard library: `std.cloud.s3`

Chapter key: `std`

> **Status:** Drafted. Written in AIL (`std/cloud/s3.ail`, ADR TBD).

A client for S3-compatible object storage (AWS S3, Cloudflare R2, MinIO, Ceph) over `std.net`, with AWS Signature
Version 4. Import it with `use std.cloud.s3`; it is used as `s3`. Every call that talks to the service has the effects
of `net.request` (`net`, `io`), and the calls that read the clock (`presign`, and every request, for its signing time)
are denied by `--deterministic` (`eff.det.1`). Its tests are the signing vectors of AWS and a loopback server.

Not here: bucket management, ACLs, encryption headers, user metadata, streaming bodies and retries.

## The module

**std.cloud.s3.1** — `client(endpoint, region, access_key, secret_key) Client` makes a client for the service at
`endpoint` (`http://HOST[:PORT]` or `https://HOST[:PORT]`, a trailing `/` dropped), using path-style URLs
(`/BUCKET/KEY`). `aws(region, access_key, secret_key)` uses `https://s3.REGION.amazonaws.com` with virtual-host URLs
(`BUCKET.HOST`), and `r2(account_id, access_key, secret_key)` uses `https://ACCOUNT.r2.cloudflarestorage.com`, region
`auto`. A `Client` has the public fields `endpoint`, `region`, `access_key`, `secret_key`, `session_token` (sent as
`x-amz-security-token` when not empty) and `virtual_host`. *(ADR TBD)*

```ail tool=run,FILE expect=true,eu-west-1 rules=std.cloud.s3.1
use std.cloud.s3

fn main() !
    mut c := s3.client("http://127.0.0.1:9000/", "eu-west-1", "key", "secret")
    c.virtual_host = true
    print("{c.virtual_host},{c.region}")
```

**std.cloud.s3.2** — Signing is AWS Signature Version 4. `sign_request(secret_key, access_key, region, service,
amz_date, method, path, query, headers, payload_hash) Signed` returns the `signature` (hex), the `authorization` header
value, the `signed_headers` list and the `canonical_request`. `amz_date` is `YYYYMMDDTHHMMSSZ`; `path` is the URI path as
sent, already percent-encoded and signed once, unnormalised (S3's rule); `query` is a `List[(str, str)]` of unencoded
pairs; every entry of `headers` is signed, with names in lower case, sorted, and values trimmed with inner runs of
white space made one space. `signing_key(secret_key, day, region, service) bytes` is the four chained HMAC-SHA256 steps
from `AWS4` and the secret. The result for the get-vanilla vector of the AWS test suite is
`5fa00fa31553b73ebf1942676e86291e8372ff2a2260956d9b8aae1d763fbf31`. *(ADR TBD)*

```ail tool=run,FILE expect=5fa00fa31553b73ebf1942676e86291e8372ff2a2260956d9b8aae1d763fbf31 rules=std.cloud.s3.2
use std.cloud.s3

fn main() !
    mut h := Map[str, str]()
    h["Host"] = "example.amazonaws.com"
    h["X-Amz-Date"] = "20150830T123600Z"
    v := s3.sign_request("wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY", "AKIDEXAMPLE", "us-east-1", "service", "20150830T123600Z", "GET", "/", List[(str, str)](), h, hash.sha256(""))
    print(v.signature)
```

**std.cloud.s3.3** — Addressing. `encode_key(key)` percent-encodes each `/`-separated segment, keeping only `A-Z a-z 0-9 - . _ ~`
(`std.url.7`). Path style addresses an object as `/BUCKET/KEY` and the bucket as `/BUCKET`; virtual host as
`BUCKET.HOST` with `/KEY`. `canonical_query(pairs)` encodes names and values the same way, sorts by name then value, and
joins `name=value` with `&`. An empty bucket fails with kind `BadRequest` before anything is sent. *(ADR TBD)*

```ail tool=run,FILE expect=a%20b/c/%C3%A9 rules=std.cloud.s3.3
use std.cloud.s3

fn main() !
    print(s3.encode_key("a b/c/é"))
    print(s3.canonical_query([("b", "2"), ("a", "x y"), ("a", "1")]))
```

**std.cloud.s3.4** — Objects. `c.put(bucket, key, data bytes, content_type) str ! S3Error` stores `data` and returns the
ETag without quotes; `content_type` is sent and signed when not empty. `c.get(bucket, key) bytes ! S3Error` returns the
bytes; `c.get_range(bucket, key, first, last)` returns the bytes `first` to `last` inclusive (a `Range` request;
`last` below `first`, or a negative `first`, is `BadRequest`). `c.delete(bucket, key) ! S3Error` removes an object, and
succeeds for a key that is not there. *(ADR TBD)*

```ail norun rules=std.cloud.s3.4
use std.cloud.s3

fn main() !
    c := s3.client("http://127.0.0.1:9000", "us-east-1", "key", "secret")
    etag := c.put("bucket", "dir/a.txt", "hello".to_bytes(), "text/plain")?
    print(etag)
    print(c.get("bucket", "dir/a.txt")?.text())
    print(c.get_range("bucket", "dir/a.txt", 1, 3)?.text())
    c.delete("bucket", "dir/a.txt")?
```

**std.cloud.s3.5** — `c.head(bucket, key) ObjectInfo ! S3Error` is a `HEAD` request: `ObjectInfo` has `size`, `etag` (no
quotes), `content_type` and `modified`, read by `object_info(headers)` from the response headers (names in lower case;
a missing header gives 0 or `""`). *(ADR TBD)*

```ail tool=run,FILE expect=1234,abc rules=std.cloud.s3.5
use std.cloud.s3

fn main() !
    mut h := Map[str, str]()
    h["content-length"] = "1234"
    h["etag"] = "\"abc\""
    i := s3.object_info(h)
    print("{i.size},{i.etag}")
```

**std.cloud.s3.6** — `c.list_page(bucket, prefix, delimiter, token, max_keys) Listing ! S3Error` is one ListObjectsV2
request (`list-type=2`); `""` and 0 leave a parameter out. A `Listing` has `objects` (`Object` with `key`, `size`, `etag`,
`modified`, `storage_class`), `prefixes` (the folded "directories" when a delimiter was given), `truncated` and
`next_token`. `parse_listing(xml)` reads the answer; text that is not XML fails with kind `Parse`. *(ADR TBD)*

```ail tool=run,FILE expect=true,tok,a.txt,1 rules=std.cloud.s3.6
use std.cloud.s3

fn main() !
    l := s3.parse_listing("<ListBucketResult><IsTruncated>true</IsTruncated><NextContinuationToken>tok</NextContinuationToken><Contents><Key>a.txt</Key><Size>1</Size></Contents></ListBucketResult>")?
    print("{l.truncated},{l.next_token},{l.objects[0].key},{l.objects[0].size}")
```

**std.cloud.s3.7** — `c.list(bucket, prefix) List[Object] ! S3Error` returns every object whose key starts with `prefix`,
following `next_token` until a page is not truncated. *(ADR TBD)*

```ail norun rules=std.cloud.s3.7
use std.cloud.s3

fn main() !
    c := s3.aws("us-east-1", "key", "secret")
    for o in c.list("bucket", "logs/2026/")?
        print("{o.key} {o.size}")
```

**std.cloud.s3.8** — Multipart upload by hand. `c.start_upload(bucket, key, content_type) str ! S3Error` returns the upload id;
`c.upload_part(bucket, key, id, number, data) Part ! S3Error` sends part `number` (1 to 10000, else `BadRequest`); every
part but the last must be at least 5 MiB (the service's rule). `c.finish_upload(bucket, key, id, parts) str ! S3Error`
puts the parts together in number order and returns the ETag; a `200` answer that holds an `<Error>` fails with kind
`Status`. `c.abort_upload(bucket, key, id) ! S3Error` drops the upload. `complete_body(parts)` is the request body.
*(ADR TBD)*

```ail tool=run,FILE expect=PartNumber rules=std.cloud.s3.8
use std.cloud.s3

fn main() !
    print(s3.complete_body([s3.Part(number: 2, etag: "b"), s3.Part(number: 1, etag: "a")]))
```

**std.cloud.s3.9** — `c.put_large(bucket, key, data, content_type, part_size) str ! S3Error` stores `data` in one request
when it is at most `part_size` bytes, otherwise as a multipart upload of parts of `part_size` bytes. A `part_size` under
5 MiB is `BadRequest`. A failed part or completion aborts the upload. It returns the ETag. *(ADR TBD)*

```ail norun rules=std.cloud.s3.9
use std.cloud.s3

fn main() !
    c := s3.r2("account", "key", "secret")
    data := fs.read_bytes("backup.tar")?
    print(c.put_large("bucket", "backup.tar", data, "application/x-tar", 8388608)?)
```

**std.cloud.s3.10** — `c.presign(method, bucket, key, expires) str ! S3Error` returns a URL that allows `method` on the
object for `expires` seconds (1 to 604800, else `BadRequest`) without the credentials: the SigV4 parameters are in the
query, only `host` is signed, and the payload is `UNSIGNED-PAYLOAD`. A session token is added as
`X-Amz-Security-Token`. `c.presign_at(method, bucket, key, expires, stamp)` signs at an explicit
`YYYYMMDDTHHMMSSZ`. The URL of the presigned-URL example of the Amazon S3 documentation is reproduced exactly. *(ADR TBD)*

```ail tool=run,FILE expect=X-Amz-Signature=aeeed9bbccd4d02ee5c0109b86d86835f995330da4c265957d157751f604d404 rules=std.cloud.s3.10
use std.cloud.s3

fn main() !
    mut c := s3.client("https://s3.amazonaws.com", "us-east-1", "AKIAIOSFODNN7EXAMPLE", "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY")
    c.virtual_host = true
    print(c.presign_at("GET", "examplebucket", "test.txt", 86400, "20130524T000000Z")?)
```

**std.cloud.s3.11** — Every request is signed with the headers `host`, `x-amz-date`, `x-amz-content-sha256` (the hex
SHA-256 of the body, `e3b0...b855` for none), `x-amz-security-token` when there is a session token, and `content-type` or
`range` when the call sets them; the `Authorization` header carries the result. *(ADR TBD)*

```ail tool=run,FILE expect=host;x-amz-date rules=std.cloud.s3.11
use std.cloud.s3

fn main() !
    mut h := Map[str, str]()
    h["host"] = "example.amazonaws.com"
    h["x-amz-date"] = "20150830T123600Z"
    print(s3.sign_request("k", "a", "us-east-1", "s3", "20150830T123600Z", "GET", "/", List[(str, str)](), h, hash.sha256("")).signed_headers)
```

**std.cloud.s3.12** — A failure is `error struct S3Error { kind S3Kind, status int, code str, message str }`. `S3Kind` is
`BadRequest` (an argument refused before sending), `Auth` (401 or 403), `NotFound` (404, or the code `NoSuchKey` or
`NoSuchBucket`), `Status` (any other status outside 200 to 299), `Parse` (an answer that is not the expected XML) and `Net`
(the request could not be sent or read; `message` names the `net.NetKind`). `status` is the HTTP status (0 for
`BadRequest` and `Net`) and `code` the `<Code>` of the service's error body (`""` when there is none). *(ADR TBD)*

```ail tool=run,FILE expect=Parse rules=std.cloud.s3.12
use std.cloud.s3

fn main() !
    _ := s3.parse_listing("not xml") else err
        print(err.kind.name())
        s3.Listing(objects: [], prefixes: [], truncated: false, next_token: "")
```

**std.cloud.s3.13** — Services that differ: R2 wants region `auto`; MinIO and Ceph take any region and path style; AWS
wants the bucket's region and accepts virtual-host URLs. The `Client` makes no region redirects; a bucket in another
region fails with kind `Status` and the service's code. *(ADR TBD)*

```ail norun rules=std.cloud.s3.13
use std.cloud.s3

fn main() !
    minio := s3.client("http://localhost:9000", "us-east-1", "minioadmin", "minioadmin")
    r2 := s3.r2("account", "key", "secret")
    aws := s3.aws("eu-central-1", "key", "secret")
    print("{minio.virtual_host} {r2.region} {aws.virtual_host}")
```

**std.cloud.s3.14** — Limits. An object is read or written whole; the HTTP client's limits apply (`std.net.7`, `std.net.19`).
Header values are signed as given with white space collapsed; non-ASCII header values are not supported. The
signing code uses the in-house HMAC-SHA256 of `std.hash`, which is unaudited: treat it as such. *(ADR TBD)*

```ail tool=run,FILE expect=c4afb1cc5771d871763a393e44b703571b55cc28424d1a5e86da6ed3c154a4b9 rules=std.cloud.s3.14
use std.cloud.s3

fn main() !
    print(s3.signing_key("wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY", "20150830", "us-east-1", "iam").hex())
```
