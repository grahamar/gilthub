gilthub
====

Rust command line application to archive/restore a git repository to/from an AWS S3 bucket.

# Dependencies
- git
- aws cli

# Archiving

### NOTE: Please make sure to create the S3 bucket first.

`gilthub archive git@github.com:gilt/scala-1-day.git s3://github-repo-archive`


# Restoring:

### NOTE: Please make sure to create the empty remote git repository first.

`gilthub restore s3://github-repo-archive/scala-1-day.tar.gz git@github.com:grahamar/scala-1-day.git`

# Building

`cargo build --release`