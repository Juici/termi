use core::str;
use std::collections::VecDeque;

use vtparse::{CsiParam, VTActor, VTParser};

use crate::event::Event;

use super::{DesktopNotificationsSupport, KeyboardEnhancementFlags};

pub struct Parser {
    engine: VTParser,
    actor: EventVTActor,
}

impl Parser {
    pub fn new() -> Self {
        Parser { engine: VTParser::new(), actor: EventVTActor::new() }
    }

    pub fn advance(&mut self, buffer: &[u8]) {
        self.engine.parse(buffer, &mut self.actor);
    }

    pub fn next(&mut self) -> Option<Event> {
        self.actor.events.pop_front()
    }
}

struct EventVTActor {
    events: VecDeque<Event>,
}

impl EventVTActor {
    fn new() -> Self {
        Self { events: VecDeque::with_capacity(32) }
    }

    fn primary_device_attributes(&mut self, _params: &[CsiParam]) {
        self.events.push_back(Event::PrimaryDeviceAttributes);
    }

    fn keyboard_enhancement_flags(&mut self, bits: i64) {
        if bits < 0 {
            return;
        }

        self.events.push_back(Event::KeyboardEnhancmentFlags(
            KeyboardEnhancementFlags::from_bits_truncate(bits as u8),
        ));
    }

    fn desktop_notifications_support(&mut self, param1: &[u8], _param2: &[u8]) {
        let identifier = match param1
            .strip_prefix(b"i=")
            .and_then(|s| s.strip_suffix(b":p=?"))
            .and_then(|s| str::from_utf8(s).ok())
        {
            Some(identifier) => identifier,
            None => return,
        };

        self.events.push_back(Event::DesktopNotifications(DesktopNotificationsSupport {
            identifier: identifier.to_owned(),
        }));
    }
}

impl VTActor for EventVTActor {
    fn print(&mut self, _b: char) {}

    fn execute_c0_or_c1(&mut self, _control: u8) {}

    fn dcs_hook(
        &mut self,
        _mode: u8,
        _params: &[i64],
        _intermediates: &[u8],
        _ignored_excess_intermediates: bool,
    ) {
    }

    fn dcs_put(&mut self, _byte: u8) {}

    fn dcs_unhook(&mut self) {}

    fn esc_dispatch(
        &mut self,
        _params: &[i64],
        _intermediates: &[u8],
        _ignored_excess_intermediates: bool,
        _byte: u8,
    ) {
    }

    fn csi_dispatch(&mut self, params: &[CsiParam], _parameters_truncated: bool, control: u8) {
        use CsiParam::*;

        // print!("csi: params={params:?} control={control}\r\n");

        match (control, params) {
            (b'c', [P(b'?'), params @ ..]) => self.primary_device_attributes(params),
            (b'u', [P(b'?'), Integer(bits)]) => self.keyboard_enhancement_flags(*bits),
            _ => {
                // TODO: Add more.
            }
        }
    }

    fn osc_dispatch(&mut self, params: &[&[u8]]) {
        // print!("osc: {params:?}\r\n");

        match params {
            [[b'9', b'9'], param1, param2] => self.desktop_notifications_support(param1, param2),
            _ => {
                // TODO: Add more.
            }
        }
    }

    fn apc_dispatch(&mut self, _data: Vec<u8>) {}
}
