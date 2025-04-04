# Data Race Web Server Demo

A Rust-based server which features a potential data race.  
***NOTE: This is a demo application modified from [The Rust Programming Language book](https://doc.rust-lang.org/book/ch21-00-final-project-a-web-server.html) to showcase how a data race may occur in a multithreaded web server. DO NOT use this code in production.***

## Routes

The following routes are defined:
- `GET` Method Routes:
  - `/`: Index page.
  - `/incr`: Incrementer page. Updates counter variable value.
- All other routes return a `404 Not Found` response.

## Data Race

Both the index (`/`) and incrementer (`/incr`) routes feature a counter which
displays the current value of a location in memory referenced with a shared
pointer among threads. Being that the incrementer route modifies this value,
there does exist the potential for a data race wherein if we suppose two or
more clients attempt to `GET /incr` simultaneously there may not be a
correctly-updated value (there could only be one increment, no increment, etc).
Additionally, reads of this value through `GET /` could potentially not show
the current value of this variable if there are updates (writes) occurring on
it while attempting to read the value.

## Running

The recommended method of running this program is via `cargo`:
```sh
cargo run
```