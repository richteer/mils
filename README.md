# mils
An opinionated tool to quickly print out a table of video metadata written in garbage-tier Rust
Prints out a table of information useful for making quick quality comparisons of video files.

# Usage
```
USAGE:
    mils [FLAGS] [OPTIONS] [DIR]...

FLAGS:
    -h, --help         Prints help information
    -r, --recursive    Recurse into subdirectories (Default max depth of 10)
    -V, --version      Prints version information

OPTIONS:
    -a, --num-audio <audio_tracks>    Max number of audio tracks to display (Default 1)
    -d, --depth <DEPTH>               Max depth of recursion. Implies -r
    -t, --threads <THREADS>           Number of threads to use to collect mediainfo
    -v, --num-video <video_tracks>    Max number of video tracks to display (Default 1)

ARGS:
    <DIR>...    Directory to list mediainfo from. Root directory if recursive

```

# Design
## Opinions
This tool was intended for simple use, hence why it only outputs a limited set of information from the file.
It also translates certain outputs into slightly more readable, or compact forms as I have found them.
A long form comparator tool may come later, but likely as a separate tool entirely.

## Why not link directly against MediaInfo, ffprobe, etc?
That would make too much sense.

In interest of getting this tool actually up and running quickly, I skipped to using a more reliable form of parsing.
Rather than trying to get the rust bindings working for libmediainfo, or link against ffmpeg, it was much easier to parse JSON, which is (hopefully) less likely to have breaking changes.
This may change in the future, depending on how much more effort ends up going into this tool.
