use core::libc::{c_void, c_int, c_char, c_ulong, c_uint};
use core::libc::funcs::posix88::unistd::{sleep};
use core::hashmap::linear;

type JackClient   = c_void;
type JackPort     = c_void;
type c_str        = *c_char;
type c_str_array  = *c_str;

enum JackStatus {
  JackFailure = 0x01,
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
  JackPortIsInput = 0x1,
  JackPortIsOutput = 0x2,
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


}

unsafe fn from_c_str_array(str_array: c_str_array, results : & mut ~[~str]) -> () {
  let mut curr_ptr = str_array;

  while (!ptr::is_null(*curr_ptr)) {
    let next = str::raw::from_c_str(*curr_ptr);
    results.push(next);
    curr_ptr = ptr::offset(curr_ptr, 1); 
  }
}


fn register_output_port(client : * JackPort) -> () {
  unsafe { 
    do str::as_c_str(~"32 bit float mono audio") |default_audio| {
      do str::as_c_str(~"secretshit") |port_name| {
        let port_type : JackPortFlags = JackPortIsOutput;
        //let out_port =  jack_port_register(client, port_name, default_audio, port_type as c_ulong, 0 as c_ulong);
        jack_port_register(client, port_name, default_audio, port_type as c_ulong, 0 as c_ulong);
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



fn parse_jack_status (status: &c_int, results: & mut ~[JackStatus]) -> () {
  let mut all_statuses = linear::LinearMap::new();
  let mut remaining = *status;

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

  if (*status == 0) {
    return; 
  }

  for uint::range_rev(6,0) |i| {
    let bit_val = float::pow_with_uint(2, i) as int;
    if remaining as int >= bit_val {
      results.push(*all_statuses.get(&bit_val));
      remaining = remaining - bit_val as i32;
    }
    if (remaining == 0) {
      break;  
    }
  }
  return;
}


fn main() -> () {
  do str::as_c_str(~"chicken") |client_name| {
    let options = 0 as c_int;
    let status_ptr : &c_int = &0;
    let mut status_results :  ~[JackStatus] = ~[];

    unsafe {
      let client : *JackClient = jack_client_open(client_name, options, status_ptr); 

      if (ptr::is_null(client)) {
        parse_jack_status(status_ptr, & mut status_results);
        fail!(fmt!("ERROR: connecting to server: %?", status_results)); 
      } else {
        io::println("client was FINE");
      }


      if (jack_activate(client) != 0) {
        fail!(~"could not activate client"); 
      }

      register_output_port(client);

      list_ports(client);

      loop {
        sleep(1 as c_uint);
      }

    }
  }
}


