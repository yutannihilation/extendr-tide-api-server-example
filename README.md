⚠WIP⚠ An example using extendr inside a simple API server ⚠WIP⚠
====================================================

## Usage

### Server

Disclaimer: I don't find how to stop this server...

```sh
cargo run
```

### Client

Plot points,

```sh
curl localhost:8080/plot/point -d '{ "x": 0.1, "y": 0.3, "radius": 2 }'
```

and view the result on browser on <http://localhost:8080/plot/result>.
Note that currently you can view it only once. The server doesn't accept the second view.