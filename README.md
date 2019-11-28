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

# Example Output
```
File with a reall…essively long name.mp4  |  h264   2.846 mb/s  720p   |  mp4a-40-2         317.3 kb/s  VBR   5.1   
Some Other File.mkv                       |  h264   9.857 mb/s  1080p  |  TrueHD + Atmos    8.910 mb/s  VBR   7.1   
Another Example File.mkv                  |  HEVC   9.917 mb/s  1080   |  TrueHD + Atmos    4.929 mb/s  VBR   7.1   
My Hot Mixtape.mkv                        |  h264   7.392 mb/s  808p   |  Dolby Digital     448.0 kb/s  CBR   5.1   
Embarassing Home Video.mkv                |  h264   4.377 mb/s  1040p  |  Dolby Digital     640.0 kb/s  CBR   5.1   
Cat.mp4                                   |  h264   986.0 kb/s  720p   |  mp4a-40-2         93.80 kb/s  VBR   Stereo
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
