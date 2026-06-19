# Actix Practice

Daily practice repository while learning the **Actix Web** framework in Rust.

I'm going through the official [Actix Web book/tutorials](https://actix.rs/docs/) section by section — implementing concepts hands-on from basic routing all the way to middleware, error handling, static files, templating, and more.

## Progress

- **Parts 1–9 completed** — Core fundamentals: routing, extractors, middleware, error handling, responses, etc.
- **Parts 10–12** — Middleware deep dive, CORS, static files + templating (Askama).
- Currently working on advanced topics daily (testing, databases, auth, etc.).

**Total commits**: 37+ (as of June 2026)

## Project Structure

```bash
.
├── src/                    # Root workspace binary (currently minimal)
├── actix_practices/
│   ├── part1/              # Basic server & routing
│   ├── part2/              # ...
│   ├── ...
│   ├── part12/             # Static files + HTML templating
├── Cargo.toml              # Workspace root
└── README.md
