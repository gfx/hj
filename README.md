# hj

**hj** is a command line tool to convert HTTP/1 style text into JSON.

## Synopsis

Simple usage with **curl(1)**:

```shell
# -sv is required to output HTTP response headers.
$ curl -sv https://httpbin.org/get 2>&1 | hj | jq .
```

The output may be:

```json
{
  "protocol": "HTTP/1.1",
  "status_code": 200,
  "headers": {
    "accept-ranges": "bytes",
    "age": "417384",
    "cache-control": "max-age=604800",
    "content-type": "text/html; charset=UTF-8",
    "date": "Tue, 25 Jan 2022 04:28:00 GMT",
    "etag": "\"3147526947\"",
    "expires": "Tue, 01 Feb 2022 04:28:00 GMT",
    "last-modified": "Thu, 17 Oct 2019 07:18:26 GMT",
    "server": "ECS (sab/56BC)",
    "vary": "Accept-Encoding",
    "x-cache": "HIT",
    "content-length": "1256"
  },
  "content": "<!doctype html>\n<html>\n<head>\n    <title>Example Domain</title>\n\n    <meta charset=\"utf-8\" />\n    <meta http-equiv=\"Content-type\" content=\"text/html; charset=utf-8\" />\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n    <style type=\"text/css\">\n    body {\n        background-color: #f0f0f2;\n        margin: 0;\n        padding: 0;\n        font-family: -apple-system, system-ui, BlinkMacSystemFont, \"Segoe UI\", \"Open Sans\", \"Helvetica Neue\", Helvetica, Arial, sans-serif;\n        \n    }\n    div {\n        width: 600px;\n        margin: 5em auto;\n        padding: 2em;\n        background-color: #fdfdff;\n        border-radius: 0.5em;\n        box-shadow: 2px 3px 7px 2px rgba(0,0,0,0.02);\n    }\n    a:link, a:visited {\n        color: #38488f;\n        text-decoration: none;\n    }\n    @media (max-width: 700px) {\n        div {\n            margin: 0 auto;\n            width: auto;\n        }\n    }\n    </style>    \n</head>\n\n<body>\n<div>\n    <h1>Example Domain</h1>\n    <p>This domain is for use in illustrative examples in documents. You may use this\n    domain in literature without prior coordination or asking for permission.</p>\n    <p><a href=\"https://www.iana.org/domains/example\">More information...</a></p>\n</div>\n</body>\n</html>\n"
}
```

If the response is JSON (`content-type: application/json`), its content is automatically parsed as JSON:

```shell
curl -sv https://jsonplaceholder.typicode.com/todos/1 2>&1 | hj | jq .
```

output:

```json
{
  "protocol": "HTTP/1.1",
  "status_code": 200,
  "headers": {
    "date": "Tue, 25 Jan 2022 04:57:52 GMT",
    "content-type": "application/json; charset=utf-8",
    "content-length": "83",
    "connection": "keep-alive",
    "x-powered-by": "Express",
    "x-ratelimit-limit": "1000",
    "x-ratelimit-remaining": "999",
    "x-ratelimit-reset": "1631494143",
    "vary": "Origin, Accept-Encoding",
    "access-control-allow-credentials": "true",
    "cache-control": "max-age=43200",
    "pragma": "no-cache",
    "expires": "-1",
    "x-content-type-options": "nosniff",
    "etag": "W/\"53-hfEnumeNh6YirfjyjaujcOPPT+s\"",
    "via": "1.1 vegur",
    "cf-cache-status": "HIT",
    "age": "13133",
    "accept-ranges": "bytes",
    "expect-ct": "max-age=604800, report-uri=\"https://report-uri.cloudflare.com/cdn-cgi/beacon/expect-ct\"",
    "report-to": "{\"endpoints\":[{\"url\":\"https:\\/\\/a.nel.cloudflare.com\\/report\\/v3?s=kfDw3zHT7KgD4%2FmRBWFv5gYnwzVVSqFH5N%2F9sTmdI425jb9mZyZggJNoeaYF2%2B%2FdEQ57JdVJggqDJZSRKt5YX%2BStzgoYRGmSwcsQ5M%2Bd1vD9rzT72hrwQfh62ZXPa01QSJxhEpyJUmf7y8BVuagT\"}],\"group\":\"cf-nel\",\"max_age\":604800}",
    "nel": "{\"success_fraction\":0,\"report_to\":\"cf-nel\",\"max_age\":604800}",
    "server": "cloudflare",
    "cf-ray": "6d2eec5508f10ac0-NRT",
    "alt-svc": "h3=\":443\"; ma=86400, h3-29=\":443\"; ma=86400"
  },
  "content": {
    "completed": false,
    "userId": 1,
    "title": "delectus aut autem",
    "id": 1
  }
}
```

## Install

cargo(1) and rustc(1) v1.58 or greater are required to install:

```
cargo install --path .
```

## Supported Commands

* curl - https://curl.se/
  * `-sv` options are required for `hj`
* h2o-httplicent - https://github.com/h2o/h2o/
* Any of HTTP/1 style text

## License

Copyright 2022 FUJI Goro.

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

