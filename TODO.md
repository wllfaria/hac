# Thoughts

Terminology and ideas for the inner workings of HAC, main focus is for me to
organize my thoughts

## Terminology

### Index

HAC-Index is a file with metadada about previously opened collections. This is
the source of truth for the collections that will appear on the initial
greeting screen. HAC will verify if the path pointed by any indexed collection
is valid, and notify the user if there is a broken link. Allowing for ease of
removal

#### Details

* On startup, HAC will read or create the index file if one is not present
  already;
* HAC will verify that every collection pointed by the index exist, and mark for
  deletion the ones that are broken;
* When opening a collection, HAC will read its data from the directory specified
  by the index.

## Collection

Represents documentation for a single API. Containing requests, cookies, headers
and everything else that's necessary for executing requests against certain API.

This is stored in-disk, in a directory with a url-friendly version of the
collection's name.

*(not sure yet which format to store files)*

The directory will consist of: 
* meta.{json|toml}
    * Will contain metadata about the given collectioin, including environment,
      authentication presets and many other

