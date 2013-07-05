# Cog

Cog is an example program written in rust of using the Jack Audio Connection Kit to play sound.  It relies on the crate rusty-jack(https://github.com/psantacl/rusty-jack) for its interop with Jack.

## Building
1. [Install](http://jackaudio.org/download) the Jack Audio Connection Kit server
2. Clone the [rusty-jack](https://github.com/psantacl/rusty-jack) create and build with `make`.
3. Copy the crate's build artifact (librusty_jack*dylib*) into the cog's libs directory
4. `make && ./bin/cog` 

## Fun
Cog will connect to a runner Jack server.  You can use the JackPilot which comes with the Jack to connect the Cog application to whatever output you wish: Speakers, Logic Audio etc.

Cog also creates a named fifo under /tmp/cog-in.  Any data you pipe through that fifo will be interpretted as audio data and played: `cat snd-files/the-sound-of-silence-f32.wav > /tmp/cog-in`

  
## Playback Algorithms
Cog can optionally manipulate the data it receives prior to sending to Jack.  Descriptions of some of the algorithm are below:
  * Clean: Interpret all data as 32 bit floats.  Any samples outside of the audio range(-1.0 ... 1.0) will be brought into the accept range.
  * Bit Reduce:  Interpret all data as 32 bit floats and then bit reduce them by XOR the mantissa.  The result is distortion.
  * Stutter: Emulate CD skipping by probabilistically stuttering the output.
