use core::libc::{c_void, c_int, c_char, c_ulong, c_uint, c_float, c_double, uint32_t, size_t, memset};

use core::libc::funcs::posix88::unistd::{sleep};
use core::libc::funcs::c95::string::{memcpy};
use core::libc::{rand, RAND_MAX};

use core::hashmap::linear;
use core::rand;

type JackClient             = c_void;
type JackPort               = c_void;
type c_str                  = *c_char;
type c_str_array            = *c_str;
type JackNFrames            = uint32_t;
type JackProcessCallback    = *u8;
type JackDefaultAudioSample = c_float;  

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
  pub rb_ptr: *JackRingBuffer
}

fn write_cstr(c: *c_char) -> () {
  unsafe { 
    use core::libc::{puts};
    puts(c);
  }
  //use core::libc::{write, strlen};
  //write(1, p as *c_void , 1);
  //use core::vec;
  //use core::cast::{transmute};

  //unsafe { 
  //  let len = strlen(p);
  //  let p: *u8 = transmute(p);
  //  do vec::raw::buf_as_slice(p, len as uint) |s| {
  //    write(1, s as *c_void , len);
  //  }
  //}
}


//TODO:
//int jack_client_close ( jack_client_t ∗ client )
//int jack_deactivate ( jack_client_t ∗ client )

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
  fn jack_activate( client : *JackClient) -> c_int;

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
    let buffer = jack_port_get_buffer( out_port_ptr, frames);
    let available_read : size_t = jack_ringbuffer_read_space(rb_ptr);

    //do str::as_c_str(fmt!("Available Read: %d", available_read as int)) |c_str| {
    //  puts( c_str as *c_char );
    //}

    //do str::as_c_str(fmt!("NFrames: %d", frames as int)) |c_str| {
    //  puts( c_str as *c_char );
    //}

    if (available_read as JackNFrames > frames) {
      //read what you can(available_read), pad the rest with silence  
      for uint::range(0, available_read/4 as uint) |i| {
        let buffer_ptr = buffer as uint + (sys::size_of::<JackDefaultAudioSample>() * i);
        let result : size_t = jack_ringbuffer_read(rb_ptr, buffer_ptr as *c_char, sys::size_of::<JackDefaultAudioSample>() as u64);

        if (result  != sys::size_of::<JackDefaultAudioSample>() as u64) {
          do str::as_c_str(fmt!("Read Result: %d", result as int)) |c_str| {
            puts( c_str as *c_char );
          }
          return -1 as c_int;
        }
      }
      //let offset : uint = available_read as uint + 1;
      //let buffer_ptr = buffer as uint + (sys::size_of::<JackDefaultAudioSample>() * offset );
      //memset( buffer_ptr as *c_void, 0, (sys::size_of::<JackDefaultAudioSample>() * (frames as uint) - offset) as u64 );
    } else {
      //read frames worth from buffer
      //for uint::range(0, frames as uint) |i| {
      //  let buffer_ptr = buffer as uint + (sys::size_of::<JackDefaultAudioSample>() * i);
      //  let result = jack_ringbuffer_read( rb_ptr, buffer_ptr as *c_char, sys::size_of::<JackDefaultAudioSample>() as u64);
      //  if (result != frames as u64) {
      //    return -1;
      //  }
      //}
    }
  }
  return 0 as c_int;
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

fn main() -> () {
  do str::as_c_str(~"Rusty Jack") |client_name| {
    let options = 0 as c_int;
    let mut status = BoxedJackStatus { val: 0, errors: ~[] };
    unsafe {
      let client : *JackClient = jack_client_open(client_name, options, & status.val); 

      if (ptr::is_null(client)) {
        status.parse_jack_status();
        fail!(fmt!("ERROR: connecting to server: %?", status.errors)); 
      } 

      let out_port_ptr = register_output_port(client);
      let initial_size = (1024 * sys::size_of::<JackDefaultAudioSample>() as u64);
      let rb = jack_ringbuffer_create( initial_size );

      if (jack_ringbuffer_mlock(rb) != 0) {
        fail!(~"ERROR: unable mlock ring buffer");
      }

      let process_args = ~ProcessArgs { out_port_ptr : out_port_ptr, rb_ptr: rb };

      if (jack_set_process_callback(client, process_ring_buffer, ptr::addr_of(process_args) as *c_void) != 0) {
        fail!(~"ERROR: unable to set process callback");
      }

      if (jack_activate(client) != 0) {
        fail!(~"could not activate client"); 
      }

      //list_ports(client);

      loop {
        let mut available_write = jack_ringbuffer_write_space(rb);
        while (available_write > sys::size_of::<JackDefaultAudioSample>() as u64) {
          let mut next_sample : JackDefaultAudioSample = ((rand() as c_double/ (RAND_MAX as c_double) * 2.0) - 1.0) as JackDefaultAudioSample;
          let next_sample_ptr = core::ptr::addr_of(&next_sample);

          let write_result = jack_ringbuffer_write(rb, next_sample_ptr as *char, sys::size_of::<JackDefaultAudioSample>() as u64);

          if (write_result != 4) {
            fail!(~"ERROR: while writing to ring buffer");
          }
          available_write -= write_result;
        }
        //sleep(1 as c_uint);
      } //loop

    }
  }
}


