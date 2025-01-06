# Http Receiver

A simple tool for receiving http requests and viewing their contents.

## Installation

1. Clone the repository: git clone https://github.com/colin99d/http-receiver
2. Install rust: https://www.rust-lang.org/tools/install
3. Build the project: cargo build --release
4. Copy the binary into your bin: cp target/release/http-receiver /usr/local/bin


## Usage

To begin using the server, you can simply run `http-receiver` in your terminal.
This will start the server on port 9000. For more advanced usage see our example
below.

### Example

Here is an example of returning custom json and selecting headers to highlight.

```bash
http-receiver -j '{"value1": "key1", "value2": 5 }' -H authorization,content-length
```

Here is an example of changing the port to 3030 and making the return status code 404.

```bash
http-receiver -s 404 -p 3030
```
