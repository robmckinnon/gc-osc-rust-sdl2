extern crate sdl2;
extern crate rosc;

use std::{env};
use std::net::{UdpSocket, SocketAddrV4};
use std::str::FromStr;
use rosc::{OscPacket, OscMessage, OscType};
use sdl2::controller::{Button, Axis};
use rosc::encoder;

fn get_addr_from_arg(arg: &str) -> SocketAddrV4 {
    SocketAddrV4::from_str(arg).unwrap()
}

fn new_msf_buf(addr: String, args: Option<Vec<OscType>>) -> Vec<u8> {
    encoder::encode(&OscPacket::Message(OscMessage {
          addr: addr,
          args: args,
    })).unwrap()
}

fn send_osc(sock: UdpSocket, to_addr: SocketAddrV4, addr: String, args: Option<Vec<OscType>>) -> UdpSocket {
    let msg_buf = new_msf_buf(addr, args);
    sock.send_to(&msg_buf, to_addr).unwrap();
    sock
}

// const OSC_BUTTON_INVALID: &str = "/bi";
const OSC_BUTTON_A: &str = "/b/a"; // X
const OSC_BUTTON_B: &str = "/b/b"; // CIRCLE
const OSC_BUTTON_X: &str = "/b/x"; // SQUARE
const OSC_BUTTON_Y: &str = "/b/y"; // TRIANGLE
const OSC_BUTTON_BACK: &str = "/b/back"; // SHARE
const OSC_BUTTON_GUIDE: &str = "/b/guide"; // ON_BUTTON
const OSC_BUTTON_START: &str = "/b/start"; // OPTIONS
const OSC_BUTTON_LEFTSTICK: &str = "/b/leftstick"; // LEFT_ANALOG_PRESS
const OSC_BUTTON_RIGHTSTICK: &str = "/b/rightstick"; // RIGHT_ANALOG_PRESS
const OSC_BUTTON_LEFTSHOULDER: &str = "/b/leftshoulder";
const OSC_BUTTON_RIGHTSHOULDER: &str = "/b/rightshoulder";
const OSC_BUTTON_DPAD_UP: &str = "/b/dpup";
const OSC_BUTTON_DPAD_DOWN: &str = "/b/dpdown";
const OSC_BUTTON_DPAD_LEFT: &str = "/b/dpleft";
const OSC_BUTTON_DPAD_RIGHT: &str = "/b/dpright";
// const OSC_BUTTON_MAX: &str = "/bm";

// const OSC_AXIS_INVALID: &str = "/ai";
const OSC_AXIS_LEFTX: &str = "/a/leftx";
const OSC_AXIS_LEFTY: &str = "/a/lefty";
const OSC_AXIS_RIGHTX: &str = "/a/rightx";
const OSC_AXIS_RIGHTY: &str = "/a/righty";
const OSC_AXIS_TRIGGERLEFT: &str = "/a/lefttrigger";
const OSC_AXIS_TRIGGERRIGHT: &str = "/a/righttrigger";
// const OSC_AXIS_MAX: &str = "/am";

fn axis_osc_msg(axis: Axis) -> String {
    let msg = match axis {
        Axis::LeftX        => OSC_AXIS_LEFTX,
        Axis::LeftY        => OSC_AXIS_LEFTY,
        Axis::RightX       => OSC_AXIS_RIGHTX,
        Axis::RightY       => OSC_AXIS_RIGHTY,
        Axis::TriggerLeft  => OSC_AXIS_TRIGGERLEFT,
        Axis::TriggerRight => OSC_AXIS_TRIGGERRIGHT,
    };
    msg.to_string()
}

fn button_osc_msg(button: Button) -> String {
    let msg = match button {
        Button::A             => OSC_BUTTON_A,
        Button::B             => OSC_BUTTON_B,
        Button::X             => OSC_BUTTON_X,
        Button::Y             => OSC_BUTTON_Y,
        Button::Back          => OSC_BUTTON_BACK,
        Button::Guide         => OSC_BUTTON_GUIDE,
        Button::Start         => OSC_BUTTON_START,
        Button::LeftStick     => OSC_BUTTON_LEFTSTICK,
        Button::RightStick    => OSC_BUTTON_RIGHTSTICK,
        Button::LeftShoulder  => OSC_BUTTON_LEFTSHOULDER,
        Button::RightShoulder => OSC_BUTTON_RIGHTSHOULDER,
        Button::DPadUp        => OSC_BUTTON_DPAD_UP,
        Button::DPadDown      => OSC_BUTTON_DPAD_DOWN,
        Button::DPadLeft      => OSC_BUTTON_DPAD_LEFT,
        Button::DPadRight     => OSC_BUTTON_DPAD_RIGHT,
    };
    msg.to_string()
}

fn main() {
    // OSC setup
    let args: Vec<String> = env::args().collect();
    let usage = format!("Usage: {} OSC_HOST_IP:HOST_PORT OSC_CLIENT_IP:CLIENT_PORT",
                        &args[0]);
    if args.len() < 3 {
        panic!(usage);
    }
    let host_addr = get_addr_from_arg(&args[1]);
    let to_addr = get_addr_from_arg(&args[2]);
    let mut sock = UdpSocket::bind(host_addr).unwrap();

    // GC setup
    let sdl_context = sdl2::init().unwrap();
    let game_controller_subsystem = sdl_context.game_controller().unwrap();

    let available =
        match game_controller_subsystem.num_joysticks() {
            Ok(n)  => n,
            Err(e) => panic!("can't enumerate joysticks: {}", e),
        };

    println!("{} joysticks available", available);

    let mut controller = None;

    // Iterate over all available joysticks and look for game
    // controllers.
    for id in 0..available {
        if game_controller_subsystem.is_game_controller(id) {
            println!("Attempting to open controller {}", id);

            match game_controller_subsystem.open(id) {
                Ok(c) => {
                    // We managed to find and open a game controller,
                    // exit the loop
                    println!("Success: opened \"{}\"", c.name());
                    controller = Some(c);
                    break;
                },
                Err(e) => println!("failed: {:?}", e),
            }

        } else {
             println!("{} is not a game controller", id);
        }
    }

    let controller =
        match controller {
            Some(c) => c,
            None     => panic!("Couldn't open any controller"),
        };

    println!("Controller mapping: {}", controller.mapping());

    for event in sdl_context.event_pump().unwrap().wait_iter() {
        use sdl2::event::Event;

        match event {
            Event::ControllerAxisMotion{ axis, value: val, .. } => {
                // Axis motion is an absolute value in the range
                // [-32768, 32767]. Let's simulate a very rough dead
                // zone to ignore spurious events.
                let dead_zone = 10_000;
                if val > dead_zone || val < -dead_zone {
                    let msg = axis_osc_msg(axis);
                    // println!("Axis {:?} moved to {}", axis, val);
                    sock = send_osc(sock, to_addr, msg.to_string(), Some(vec![OscType::Int(val.into())]));
                }
            }
            Event::ControllerButtonDown{ button, .. } => {
                let msg = button_osc_msg(button);
                // println!("Button {:?} down", button);
                sock = send_osc(sock, to_addr, msg, Some(vec![OscType::Int(1)]));
            }
            Event::ControllerButtonUp{ button, .. } => {
                let msg = button_osc_msg(button);
                // println!("Button {:?} up", button);
                sock = send_osc(sock, to_addr, msg, None);
            }
            Event::Quit{..} => break,
            _ => (),
        }
    }
}
