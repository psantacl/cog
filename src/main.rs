use core::libc::{c_void, c_int, c_uint, c_ulong};
use core::libc::funcs::posix88::unistd::{sleep};

pub type c_str = *libc::c_char;
pub type c_str_array = *c_str;

pub enum JackOptions {
  JackNullOption = 0x00,
  JackNoStartServer = 0x01,
  JackUseExactName = 0x02,
  JackServerName = 0x04,
  JackLoadName = 0x08,
  JackLoadInit = 0x10,
  JackSessionID = 0x20
} 

type jack_options_t = JackOptions; 

enum JackStatus {
   JackFailure = 0x01,
   JackInvalidOption = 0x02,
   JackNameNotUnique = 0x04,
   JackServerStarted = 0x08,
   JackServerFailed = 0x10,
   JackServerError = 0x20,
   JackNoSuchClient = 0x40,
   JackLoadFailure = 0x80,
   JackInitFailure = 0x100,
   JackShmFailure = 0x200,
   JackVersionError = 0x400,
   JackBackendError = 0x800,
   JackClientZombie = 0x1000
} 

type jack_status_t = JackStatus;

enum JackPortFlags {
  JackPortIsInput = 0x1,
  JackPortIsOutput = 0x2,
  JackPortIsPhysical = 0x4,
  JackPortCanMonitor = 0x8,
  JackPortIsTerminal = 0x10
}

//Opaque Types
type jack_client_t = c_void;
type jack_port_t = c_void;


#[link_args = "-ljack"]
extern {
  //fn jack_client_open( client_name : c_str,  options : jack_options_t, status: *jack_status_t) -> *jack_client_t;
  //fn jack_client_open( client_name : c_str,  options : jack_options_t, status: ()) -> *jack_client_t;
  fn jack_client_open( client_name : c_str,  options : c_int, status: *jack_status_t) -> *jack_client_t;

  fn jack_activate( client : *jack_client_t) -> c_int;

  fn jack_port_register(client: *jack_client_t, 
                        port_name : c_str, 
                        port_type: c_str, 
                        flags : c_ulong, 
                        buffer_size : c_ulong) -> *jack_port_t;

  fn jack_get_ports(client : *jack_client_t, 
                    port_name_pattern : c_str, 
                    type_name_pattern: c_str, 
                    flags : c_ulong) -> c_str_array;
}


unsafe fn from_c_str_array(str_array: c_str_array, results : & mut ~[~str]) -> () {
  let mut curr_ptr = str_array;
   
  while (!ptr::is_null(*curr_ptr)) {
    let next = str::raw::from_c_str(*curr_ptr);
    results.push(next);
    curr_ptr = ptr::offset(curr_ptr, 1); 
  }
}

fn register_output_port(client : * jack_port_t) -> () {
  do str::as_c_str(~"32 bit float mono audio") |default_audio| {
    do str::as_c_str(~"secretshit") |port_name| {
      let port_type : JackPortFlags = JackPortIsOutput;
      let out_port =  jack_port_register(client, port_name, default_audio, port_type as c_ulong, 0 as c_ulong);
    }
  }
	//output_port = jack_port_register (client, bpm_string, JACK_DEFAULT_AUDIO_TYPE, JackPortIsOutput, 0);
}

#[allow(non_implicitly_copyable_typarams)]
fn main() {
  do str::as_c_str(~"chicken") |client_name| {
    let options : JackOptions = JackNullOption; 
    let status : JackStatus = JackFailure;

    unsafe { 
      let client : *jack_client_t = jack_client_open(client_name, 0, &status ); 

      if (ptr::is_null(client)) {
        fail(fmt!("error connecting to server: %d" status as int)); 
      } 

      if (jack_activate(client) != 0) {
        fail(~"could not activate client"); 
      }

      do str::as_c_str(~"") |empty_string| {
        let ports : c_str_array = jack_get_ports(client, empty_string ,empty_string, 0 as c_ulong);
        let mut port_names :  ~[~str] = ~[];
        from_c_str_array(ports, & mut port_names);
        for port_names.each |port| {
          io::println(*port);
        }
      }

      register_output_port(client);

      while true {
        sleep(1 as c_uint);
      }
    }

  }
  
}
