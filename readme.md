

## Description
This is a simple Rust application that I made for creating custom webhooks for anywhere that I might need them. It is a simple web server that listens for incoming requests and then performs some per sepcified operations. The application is built using the Axum framework and uses the shuttle for hosting.

I also have a scheduler that runs cron-like jobs at specified intervals. I am using 

## Webhooks implemented
- **Notion**: This webhook is used to edit a pdf based on data from a Notion database. It listens for incoming requests from Notion and then updates the pdf with the data from the request and creating a copy. It them emails the pdf to the specified email address. The webhook is triggered by a button in Notion that sends a request to the server with the data from the database. 
	- TODO: add generalisation for other people to be able to use it.

- **Discord**: I haven't implemented this yet, but the idea is to create a middleman for discord webhooks. It recieves webhooks from different sources, formats them for discord and then sends them to the specified discord channel. This is useful for sending notifications from different sources to a single discord channel and using sources that aren't supported by discord yet, example: Square, PayU.


## Resources

### Notion

- [Notion SDK for JavaScript](https://github.com/makenotion/notion-sdk-js)
- [Database Query API Reference](https://developers.notion.com/reference/post-database-query)

### Shuttle

- [Shuttle Console](https://console.shuttle.dev/)
- [Shuttle Hello World Example](https://github.com/shuttle-hq/shuttle-examples/blob/main/axum/hello-world/Cargo.toml)

### Other Rust crates

- [pdf_form crate](https://crates.io/crates/pdf_form)
- [reqwest Response documentation](https://docs.rs/reqwest/latest/reqwest/struct.Response.html)
- [axum documentation](https://docs.rs/axum/latest/axum/index.html#handlers)
- [Parsing dynamic JSON in Rust](https://ahmadrosid.com/blog/rust-parsing-dynamic-json)

