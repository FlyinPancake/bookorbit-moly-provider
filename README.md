# BookOrbit Moly provider

A [BookOrbit](https://github.com/bookorbit/bookorbit) metadata provider for [moly.hu](https://moly.hu), the Hungarian
book community site. It scrapes moly.hu's search and book pages and answers in the
[book-metadata-provider](https://github.com/FlyinPancake/book-metadata-provider) format.

## What it fills

| Field                        | Source on moly.hu                                                                                                     |
| ---------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Title, subtitle, authors     | The book page. A language marker such as "Dune (angol)" is cut from the title                                         |
| Language                     | The "angol nyelvű" style tag, or the title's language marker without one. A book with several language tags gets none |
| Description                  | The book page, without the spoiler warning                                                                            |
| Series name and index        | The series link next to the title. Omnibus ranges such as "1–3." get no index                                         |
| Genres                       | The book's tags, minus the language tags                                                                              |
| Community rating             | moly's percentage divided by 20, and the number of ratings                                                            |
| Publisher, year, date, pages | The chosen edition                                                                                                    |
| ISBN-13 or ISBN-10           | The chosen edition, without hyphens                                                                                   |
| Cover                        | The chosen edition, in the largest size moly.hu serves                                                                |
| Narrators                    | The "Felolvasta" line of an audiobook edition                                                                         |

A moly.hu book page lists every edition of the book. The provider picks one per answer:

1. The edition whose ISBN BookOrbit searched for, if there is one.
2. For an audiobook search, the newest edition marked as an audiobook (it has a narrator or the "hangoskönyv" tag).
3. Otherwise the newest print edition with a cover, then the newest print edition.

An audiobook edition gets the id `<slug>#audiobook`, so a later lookup by id returns the same edition. The plain
`<slug>` always means the print edition.

## Run it

```yaml
services:
  moly-provider:
    image: ghcr.io/flyinpancake/bookorbit-moly-provider:0.0.3
    restart: unless-stopped
    environment:
      AUTH_TOKEN: change-me
```

Put it in the same Compose project as BookOrbit, then add it under
**Settings > Metadata > Providers > External providers** with the address `http://moly-provider:8080` and the
authorization header `Bearer change-me`. Turn on **Allow private address**, because the provider runs on your own
network.

To build the image yourself:

```bash
docker build -t bookorbit-moly-provider .
```

The image is a statically linked binary in an empty `scratch` image, running as user `65534`.

## Configuration

| Variable               | Default | Meaning                                                                                                   |
| ---------------------- | ------- | --------------------------------------------------------------------------------------------------------- |
| `PORT`                 | `8080`  | Port to listen on                                                                                         |
| `AUTH_TOKEN`           | unset   | When set, every API request needs `Authorization: Bearer <token>` (or the bare token). `/docs` stays open |
| `MOLY_MIN_INTERVAL_MS` | `500`   | Minimum gap between two requests to moly.hu, shared by all callers                                        |
| `RUST_LOG`             | `info`  | Log filter, such as `debug` or `bookorbit_moly_provider=debug`                                            |

Without `AUTH_TOKEN` anyone who can reach the port can use it. Set one if you expose it beyond your own network.

## API

- `GET /provider/info`: name, key (`moly`), version and capabilities.
- `GET /search?query=&author=&isbn=&limit=&mediaType=`: matches in moly.hu's order. `limit` defaults to 5 and is capped
  at 10, because every match costs one page fetch.
- `GET /book/{id}`: one book by the `providerId` a search returned, or `404`.
- `GET /docs`: the OpenAPI reference. With `AUTH_TOKEN` set it declares a bearer scheme, so you can enter the token
  there and try requests.

When moly.hu throttles, the provider answers `429` with `Retry-After`. When moly.hu fails or changes its page layout, it
answers `502`. If only some matches of a search fail to load, the answer has the rest and `"partial": true`.

## What it sends to moly.hu

For each search, the title and author, or the ISBN, that BookOrbit asks about, as a moly.hu search. Then one request per
match for its book page. Nothing else, and it keeps no data. It does not log queries.

## Known gaps

- moly.hu does not list audiobook length or abridgement, so the provider never fills them.
- Language tags map to ISO 639-1 codes through a fixed list of about 35 languages. A tag outside the list gives no
  language.
- Cover sizes are not known, so `coverWidth` and `coverHeight` are always empty.
- It depends on moly.hu's HTML. A layout change breaks parsing until the provider is updated.

## Development

```bash
cargo test
mise run dev   # rebuilds and restarts on change
```

The scraper tests run offline against saved pages in `tests/fixtures/`.
