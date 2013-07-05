# Cog

Cog is an example program written in rust of using the Jack Audio Connection Kit to play sound.  It relies on the crate rusty-jack(https://github.com/psantacl/rusty-jack) for its interop with Jack.

## Building
1. Install the Jack Audio Connection Kit server: http://jackaudio.org/download
2. Clone the rusty-jack crate( https://github.com/psantacl/rusty-jack) and build with make.
3. Copy the crate's build artifact (librusty_jack*dylib*) into the cog's libs directory
4. make && ./bin/cog 

## Fun
Cog will connect to a runner Jack server.  You can use the JackPilot which comes with the Jack to connect the Cog application to whatever output you wish: Speakers, Logic Audio etc.
Cog also creates a named fifo under /tmp/cog-in.  Any data you pipe through that fifo will be interpretted as audio data and played.
