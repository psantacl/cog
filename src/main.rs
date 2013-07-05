extern mod std;
extern mod rusty_jack;

use rusty_jack::audio::*;
use core::libc::{c_void, c_int, c_char, c_double, size_t, memset, mkfifo, S_IRUSR, S_IWUSR, O_RDONLY, open, read, malloc, free, close};
use core::libc::funcs::c95::string::{memcpy};
use core::libc::{rand, RAND_MAX};

use core::rand;
use core::pipes::{stream, Port, Chan};

extern fn process_ring_buffer( frames: JackNFrames, args: *c_void ) -> c_int {
  use core::libc::{puts};
  unsafe {
    let out_port_ptr = (*(args as *ProcessArgs)).out_port_ptr; 
    let rb_ptr = (*(args as *ProcessArgs)).rb_ptr;
    let mut offset = 0;
    let buffer = jack_port_get_buffer( out_port_ptr, frames);
    let bytes_available : size_t = jack_ringbuffer_read_space(rb_ptr);
    let frames_available = bytes_available as uint/(sys::size_of::<JackDefaultAudioSample>() as uint);
    let total_bytes_requested = frames as u64 * (sys::size_of::<JackDefaultAudioSample>() as u64);
    let mut frames_to_be_read : u32;

    if (frames < frames_available as u32) {
      frames_to_be_read = frames; 
    } else {
      frames_to_be_read = frames_available as u32;
    }

    // do str::as_c_str(~"-------------------------------------------------------") |c_str| {
    //   puts( c_str as *c_char );
    // }

    // do str::as_c_str(fmt!("Bytes requested: %d", total_bytes_requested as int)) |c_str| {
    //   puts( c_str as *c_char );
    // }

    // do str::as_c_str(fmt!("Frames requested: %d", frames as int)) |c_str| {
    //   puts( c_str as *c_char );
    // }

    // do str::as_c_str(fmt!("Bytes Available: %d", bytes_available as int)) |c_str| {
    //  puts( c_str as *c_char );
    // }

    // do str::as_c_str(fmt!("frames Available: %d", frames_available as int)) |c_str| {
    //  puts( c_str as *c_char );
    // }

    // do str::as_c_str(fmt!("memcopying over %d frames", frames_to_be_read as int)) |c_str| {
    //  puts( c_str as *c_char );
    // }

    //copy over as much of the requested data as we have available in the ring buffer
    for uint::range(0, frames_to_be_read as uint) |i| {
      let buffer_ptr = buffer as uint + offset;
      
      //(*(args as *ProcessArgs)).chan.send( *(buffer_ptr as *f32) );
      let result : size_t = jack_ringbuffer_read(rb_ptr, buffer_ptr as *c_char, sys::size_of::<JackDefaultAudioSample>() as u64);

      if (result  != sys::size_of::<JackDefaultAudioSample>() as u64) {
        do str::as_c_str(fmt!("ERROR: bad read result: %d", result as int)) |c_str| {
          puts( c_str as *c_char );
        }
        return -1 as c_int;
      }

      offset = offset + sys::size_of::<JackDefaultAudioSample>();
    }


    if (bytes_available < total_bytes_requested) {
      let buffer_ptr = buffer as uint + offset;
      memset( buffer_ptr as *c_void, 0, (total_bytes_requested - bytes_available) );
    }
    (*(args as *ProcessArgs)).chan.send(~"more");
  }
  return 0 as c_int;
}

extern fn process_loud( frames : JackNFrames, args: *c_void) -> c_int {
  unsafe {
    let buffer = jack_port_get_buffer( (*(args as *ProcessArgs)).out_port_ptr, frames);
    for uint::range(0, frames as uint) |i| {
      let mut next_sample : JackDefaultAudioSample = 10000000000000.0 as JackDefaultAudioSample;
      if (i % 4 == 0) {
        next_sample = next_sample * -1.0;
      }
      let next_sample_ptr = core::ptr::addr_of(&next_sample);
      let buffer_ptr = buffer as uint + (sys::size_of::<JackDefaultAudioSample>() * i);
      memcpy(buffer_ptr as *c_void, (next_sample_ptr as *c_void), sys::size_of::<JackDefaultAudioSample>() as u64);
    }
    return 0 as c_int; 
  }
}

extern fn process_noise( frames : JackNFrames, args: *c_void ) -> c_int {
  unsafe {
    let buffer = jack_port_get_buffer( (*(args as *ProcessArgs)).out_port_ptr, frames);
    for uint::range(0, frames as uint) |i| {
      let mut next_sample : JackDefaultAudioSample = ((rand() as c_double/ (RAND_MAX as c_double) * 2.0) - 1.0) as JackDefaultAudioSample;
      let next_sample_ptr = core::ptr::addr_of(&next_sample);
      let buffer_ptr = buffer as uint + (sys::size_of::<JackDefaultAudioSample>() * i);
      memcpy(buffer_ptr as *c_void, (next_sample_ptr as *c_void), sys::size_of::<JackDefaultAudioSample>() as u64);
    }
    return 0 as c_int; 
  }
}

fn ensure_fifo_pipe() -> () {
  unsafe {
    let fifo = ~"/tmp/cog-in";
    if ( PosixPath(fifo).exists()) {
      return
    } 
    do str::as_c_str(fifo) |fifo_name| {
      if (mkfifo(fifo_name,  (S_IWUSR | S_IRUSR ) as u16) != 0) {
        fail!(fmt!("ERROR: creating named pipe"));  
      }
    }
  }
}

fn read_from_fifo_clean(rb : *JackRingBuffer, pipe: c_int, cb: &fn(*c_void, i64) -> (*c_void, u64)) -> () {
  unsafe {
    let mut bytes_written = 0;
    let mut rb_available_space = jack_ringbuffer_write_space(rb);

    while (rb_available_space > bytes_written) {
      rb_available_space = (rb_available_space / 4) * 4;
      let mut read_buffer = malloc( rb_available_space - bytes_written);

      let mut bytes_available_from_pipe = read( pipe, read_buffer as *mut c_void, rb_available_space - bytes_written); 

      //No data available in pipe
      if (bytes_available_from_pipe <= 0) {
        free(read_buffer);
        break;
      }

      let (processed_bytes, processed_bytes_size) = cb(read_buffer, bytes_available_from_pipe);

      bytes_written = bytes_written + processed_bytes_size;

      //NB> should examine result of jack_ringbuffer_write to make sure everything went smoothly
      jack_ringbuffer_write(rb, processed_bytes as *char, processed_bytes_size);
      free(read_buffer);
    }
  }
}



trait Playable {
  fn get_next_sample( &mut self, sample : f32 ) -> f32;
}

struct CleanCog;

impl Playable for CleanCog {
  fn get_next_sample(&mut self, sample : f32) -> f32 {
    sample
  }
}

struct DirtyCog;

impl Playable for DirtyCog {
  fn get_next_sample(&mut self, sample : f32) -> f32 {
    let int_sample_ptr : *u32 = (ptr::addr_of(&sample) as *u32);
    unsafe { 
      let int_sample = (*int_sample_ptr) & 0b1_01111100_11111111111111111111111;
      return *(ptr::addr_of(&int_sample) as *f32);
    }
  }
}

struct StutterCog {
  pub stutter_idx : int,
  pub data : ~[f32],
  pub in_stutter: bool,
  pub stutter_win_size: int
}


impl Playable for StutterCog {
  fn get_next_sample(&mut self, sample : f32) -> f32 {
    let mut stutter_sample : f32;

    //normal playback
    if (!self.in_stutter) {
      self.begin_stutter_pred();
      return sample;
    }
    
    self.end_stutter_pred(); 

    //in stutter...
    //window is full, begin to repeat
    if (self.data.len() == self.stutter_win_size as uint) {
      self.stutter_idx = self.stutter_idx % self.stutter_win_size;
      stutter_sample = self.data[self.stutter_idx];
      self.stutter_idx = self.stutter_idx + 1;
      //maybe wrap around
      return stutter_sample;
    }
    //fill up window
    self.data.push(sample); 
    self.stutter_idx = self.stutter_idx + 1;
    sample
  }
}

impl StutterCog {
  fn end_stutter_pred(&mut self) -> () {
    let p_of_end_stutter : c_double = 0.0001;
    if (!self.in_stutter) {
      fail!(~"error: not in a stutter");
    }
    unsafe { 
      if (rand() as c_double / (RAND_MAX as c_double) < p_of_end_stutter) {
        self.in_stutter = false;
        self.data.clear();
        self.stutter_idx = 0;
      }
    }
  }
  
  fn begin_stutter_pred(&mut self) -> () {
    let p_of_start_stutter : c_double = 0.00001;

    if (self.in_stutter) {
      fail!(~"error: already in a stutter");
    } 
    unsafe { 
      if (rand() as c_double / (RAND_MAX as c_double) < p_of_start_stutter) {
        let r = core::rand::Rng();
        //-0.5..0.5
        let stutter_win_delta =  core::rand::Rand::rand::<float>(r) - 0.5;
        self.stutter_win_size = self.stutter_win_size + ((self.stutter_win_size as float) * stutter_win_delta) as int;
        self.in_stutter = true;
        self.stutter_idx = 0;
      }
    }
  }
}

fn main() -> () {
  unsafe { 
    do str::as_c_str(~"Rusty Jack") |client_name| {
      let options = 0 as c_int;
      let mut status = BoxedJackStatus { val: 0, errors: ~[] };

      ensure_fifo_pipe();

      let client : *JackClient = jack_client_open(client_name, options, & status.val); 
      if (ptr::is_null(client)) {
        status.parse_jack_status();
        fail!(fmt!("ERROR: connecting to server: %?", status.errors)); 
      } 

      let out_port_ptr = register_output_port(client);
      let initial_size = (1024 * sys::size_of::<JackDefaultAudioSample>() as u64);
      let rb : *JackRingBuffer = jack_ringbuffer_create( initial_size );

      if (jack_ringbuffer_mlock(rb) != 0) {
        fail!(~"ERROR: unable mlock ring buffer");
      }


      let (fifo_cmd_port, fifo_cmd_chan): (Port<~str>, Chan<~str>) = stream();
      let process_args = ~ProcessArgs { out_port_ptr : out_port_ptr, 
        rb_ptr       : rb, 
        chan         : fifo_cmd_chan };

      if (jack_set_process_callback(client, process_ring_buffer, ptr::addr_of(process_args) as *c_void) != 0) {
        fail!(~"ERROR: unable to set process callback");
      }

      if (jack_activate(client) != 0) {
        fail!(~"could not activate client"); 
      }


      let (std_in_port, std_in_chan): (Port<~str>, Chan<~str>) = stream();

      //task for handling stdin
      do spawn {
        loop {
          io::println(~"q) quit 0)clean 1)bit reduce 2)stutter");
          let next_line : ~str = io::stdin().read_line();
          if (next_line == ~"q") {
            std_in_chan.send( next_line );
            break;
          }
          std_in_chan.send( next_line );
        }
      };

      //list_ports(client);

      do str::as_c_str(~"/tmp/rusty-jack-in") |pipe_path| {
        let pipe = open( pipe_path, (O_RDONLY | 0x0004) as i32, (S_IWUSR | S_IRUSR ) as i32);
        let mut current_cog : @Playable = @CleanCog as @Playable; 

        let cb = |bytes: *c_void, bytes_size: i64| {
          let mut curr_ptr : *mut c_void  = bytes as *mut c_void;
          let mut bytes_processed : u64 = 0;
          let mut next_sample : f32;

          while (bytes_processed < bytes_size as u64) {
            next_sample = *(curr_ptr as *f32);
            next_sample = current_cog.get_next_sample( next_sample );
            if (next_sample > 1.0) {
              next_sample = 1.0;
            } else if (next_sample < -1.0) {
              next_sample = -1.0;
            } 
            memcpy( curr_ptr as *c_void, (core::ptr::addr_of(&next_sample) as *c_void), sys::size_of::<JackDefaultAudioSample>() as u64);
            curr_ptr = (curr_ptr as uint + 4) as *mut c_void;
            bytes_processed = bytes_processed + 4;
          }
          (bytes, bytes_processed)
        };

        loop {
          read_from_fifo_clean(rb, pipe, cb );

          fifo_cmd_port.recv();
          if (std_in_port.peek()) {
            match std_in_port.recv() {
              ~"q"   =>  {
                jack_deactivate(client);
                jack_client_close(client);
                break;
              }

              ~"0" => { current_cog = @CleanCog as @Playable;}
              ~"1" => { current_cog = @DirtyCog as @Playable;}
              ~"2" => { current_cog = @StutterCog { stutter_idx: 0, data: ~[], in_stutter : false, stutter_win_size : 1500 } as @Playable;}
              _ => {}
            }
          }
        } //loop 
        close(pipe);
      } 
    }
  }
}

