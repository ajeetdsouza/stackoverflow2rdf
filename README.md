# stackoverflow2rdf

Converts [StackExchange 2020 data dumps](https://archive.org/download/stackexchange) into RDF. Intended for loading into [Dgraph](https://github.com/dgraph-io/dgraph).

To run:

```shell
# Installs the stackoverflow2rdf binary to your system.
cargo install --path=.

# Extract the StackOverflow dataset and put all the XML files in a directory.
stackoverflow2rdf <xml_directory> <output.rdf.gz>
```

The schema has not been properly documented, but there is an unofficial version [here](https://meta.stackexchange.com/a/2678).

More context about the tables can be found [here](https://data.stackexchange.com/stackoverflow/query/472607/information-schema-for-a-table?table=posts#resultSets).
