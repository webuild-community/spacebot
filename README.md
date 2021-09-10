# tokyo-rs

hey.

## Docker build

```
docker build -t ledongthuc/tokyo-rs:latest .;
```

## Docker start

New file `.env`

```
docker run -it -p 8091:8080 -e RUST_BACKTRACE=1 ledongthuc/tokyo-rs:latest
```

## Client guide

[Detail API for client](/blob/master/GUIDE.md)
