# fact-graph-clustering

Prototype clustering application for my Masters capstone project at University of Washington: Bothell.


## Usage

- Run `make workdir` to set up directory for experiments.
- Unpack dataset to workdir. `workdir/raw_input` should contain all the dataset's files with file names in the format `<cluster_identifier>-<whatever_else>`
- Modify `src/config.rs` as needed.
- Run `make preprocess` to preprocess the dataset.
- Run `make generate` to generate graphs from the preprocessed files
- Run `make cluster` to cluster the graphs.

`workdir` will then contain three files:

- `names` contains the filenames.
- `pred` contains the predicted clusters
- `true` contains the expected clustering
