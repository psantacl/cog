extern mod std;

use core::libc::{c_void, c_int, c_char, c_ulong, c_uint, c_float, c_double, uint32_t, size_t, memset, mkfifo, S_IRUSR, S_IWUSR, O_RDONLY, open, read, malloc, free, close};
use core::str::raw::{from_c_str};
use core::libc::funcs::posix88::unistd::{sleep};
use core::libc::funcs::c95::string::{memcpy};
use core::libc::{rand, RAND_MAX};

use core::hashmap::linear;
use core::rand;
use core::pipes::{stream, Port, Chan};

type JackClient               = c_void;
type JackPort                 = c_void;
type c_str                    = *c_char;
type c_str_array              = *c_str;
type JackNFrames              = uint32_t;
type JackProcessCallback      = *u8;
type JackDefaultAudioSample   = c_float;  


struct BoxedJackStatus {
  pub val   : c_int,
  pub errors: ~[JackStatus]
}

struct JackRingBuffer {
  buf       : *c_char,
  write_ptr : size_t, 
  read_ptr  : size_t,
  size      : size_t,
  size_mask : size_t,
  mlocked   : c_int
}

struct ProcessArgs {
  pub out_port_ptr: *JackPort,
  pub rb_ptr: *JackRingBuffer,
  pub chan : Chan<~str>
}

fn write_cstr(c: *c_char) -> () {
  unsafe { 
    use core::libc::{puts};
    puts(c);
  }
}


//TODO:
//Callbacks:
//void jack_on_shutdown ( jack_client_t ∗ client, JackShutdownCallback function, void ∗ arg )
//int jack_set_buffer_size_callback ( jack_client_t ∗ client, JackBufferSizeCallback bufsize_callback, void ∗ arg )
//int jack_set_port_connect_callback ( jack_client_t ∗ , JackPortConnectCallback connect_callback, void ∗ arg )
//float jack_cpu_load ( jack_client_t ∗ client )
//jack_nframes_t jack_get_buffer_size ( jack_client_t ∗ )
//jack_nframes_t jack_get_sample_rate ( jack_client_t ∗ )

//Ports:
//int jack_connect ( jack_client_t ∗ , const char ∗ source_port, const char ∗ destination_port )
//int jack_disconnect ( jack_client_t ∗ , const char ∗ source_port, const char ∗ destination_port )


impl BoxedJackStatus {
  fn parse_jack_status(& mut self) -> () {
    let mut all_statuses = linear::LinearMap::new();
    let mut remaining = self.val;

    all_statuses.insert(0x01, JackFailure);
    all_statuses.insert(0x02, JackInvalidOption);
    all_statuses.insert(0x04, JackNameNotUnique);
    all_statuses.insert(0x08, JackServerStarted);
    all_statuses.insert(0x10, JackServerFailed);
    all_statuses.insert(0x20, JackServerError); 
    all_statuses.insert(0x40, JackNoSuchClient); 
    all_statuses.insert(0x80, JackLoadFailure); 
    all_statuses.insert(0x100, JackInitFailure); 
    all_statuses.insert(0x200, JackShmFailure); 
    all_statuses.insert(0x400, JackVersionError); 
    all_statuses.insert(0x800, JackBackendError); 
    all_statuses.insert(0x1000, JackClientZombie); 

    if (remaining == 0) {
      return; 
    }

    for uint::range_rev(6,0) |i| {
      let bit_val = float::pow_with_uint(2, i) as int;
      if remaining as int >= bit_val {
        self.errors.push(*all_statuses.get(&bit_val));
        remaining = remaining - bit_val as i32;
      }
      if (remaining == 0) {
        break;  
      }
    }
    return;
  }
}


enum JackStatus {
  JackFailure       = 0x01,
  JackInvalidOption = 0x02,
  JackNameNotUnique = 0x04,
  JackServerStarted = 0x08,
  JackServerFailed  = 0x10,
  JackServerError   = 0x20,
  JackNoSuchClient  = 0x40,
  JackLoadFailure   = 0x80,
  JackInitFailure   = 0x100,
  JackShmFailure    = 0x200,
  JackVersionError  = 0x400,
  JackBackendError  = 0x800,
  JackClientZombie  = 0x1000
}


enum JackPortFlags {
  JackPortIsInput    = 0x1,
  JackPortIsOutput   = 0x2,
  JackPortIsPhysical = 0x4,
  JackPortCanMonitor = 0x8,
  JackPortIsTerminal = 0x10
}


#[link_args = "-ljack"]
extern {
  fn jack_client_open( client_name : c_str,  options : c_int, status: *c_int) -> *JackClient;
  fn jack_client_close ( c: *JackClient ) -> c_int;

  fn jack_activate( client : *JackClient) -> c_int;
  fn jack_deactivate( client: *JackClient ) -> c_int;

  fn jack_port_register(client      : *JackClient,    port_name : c_str, 
      port_type   : c_str,          flags     : c_ulong, 
      buffer_size : c_ulong)     -> *JackPort;

  fn jack_get_ports(client            : *JackClient, port_name_pattern : c_str, 
      type_name_pattern : c_str,      flags              : c_ulong) -> c_str_array;

  fn jack_set_process_callback (client : *JackClient,  process_callback : JackProcessCallback, 
      arg    : *c_void)   -> c_int;

  fn jack_port_get_buffer (port : *JackPort, frames : JackNFrames) -> *JackDefaultAudioSample;

  fn jack_ringbuffer_create ( sz : size_t ) -> *JackRingBuffer;
  fn jack_ringbuffer_free   ( rb : *JackRingBuffer ) -> ();
  fn jack_ringbuffer_mlock  ( rb : *JackRingBuffer ) -> c_int;
  fn jack_ringbuffer_read_space ( rb :  *JackRingBuffer ) -> size_t;
  fn jack_ringbuffer_read   ( rb: *JackRingBuffer, dest : *c_char, cnt: size_t ) -> size_t;
  fn jack_ringbuffer_write_space ( rb: *JackRingBuffer ) -> size_t;
  fn jack_ringbuffer_write  ( rb: *JackRingBuffer, src : *char, cnt : size_t ) -> size_t;


}


unsafe fn from_c_str_array(str_array: c_str_array, results : & mut ~[~str]) -> () {
  let mut curr_ptr = str_array;

  while (!ptr::is_null(*curr_ptr)) {
    let next = str::raw::from_c_str(*curr_ptr);
    results.push(next);
    curr_ptr = ptr::offset(curr_ptr, 1); 
  }
}


fn register_output_port(client : * JackPort) -> (*JackPort) {
  unsafe { 
    do str::as_c_str(~"32 bit float mono audio") |default_audio| {
      do str::as_c_str(~"out") |port_name| {
        let port_type : JackPortFlags = JackPortIsOutput;
        jack_port_register(client, port_name, default_audio, port_type as c_ulong, 0 as c_ulong)
      }
    }
  }
}

fn list_ports(client : *JackClient) -> () {
  unsafe { 
    do str::as_c_str(~"") |empty_string| {
      let ports : c_str_array = jack_get_ports(client, empty_string ,empty_string, 0 as c_ulong);
      let mut port_names :  ~[~str] = ~[];
      from_c_str_array(ports, & mut port_names);
      for port_names.each |port| {
        io::println(*port);
      }
    }
  }
}

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
    let mut frames_to_be_read = 0;

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
    let fifo = ~"/tmp/rusty-jack-in";
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
    
      let mut curr_ptr : *mut c_void  = read_buffer as *mut c_void;

      let (processed_bytes, processed_bytes_size) = cb(read_buffer, bytes_available_from_pipe);

      bytes_written = bytes_written + processed_bytes_size;

      let write_result = jack_ringbuffer_write(rb, processed_bytes as *char, processed_bytes_size);
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
    let mut stutter_sample : f32 = 0.0;

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
          let mut next_sample : f32 =  0.0;

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

